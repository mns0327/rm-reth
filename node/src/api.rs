use crate::cert::NoVerifier;
use crate::error::NodeError;
use figment::{
    Figment,
    providers::{Format, Yaml},
};
use network::LoggedStream;
use serde::Deserialize;
use tracing_subscriber::fmt::format::FmtSpan;
use std::{
    net::{IpAddr, SocketAddr}, path::Path, str::FromStr, sync::Arc, time::Duration
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::{RwLock, Mutex},
    time::timeout,
};
use tokio_rustls::{
    TlsConnector,
    client::TlsStream,
    rustls::{ClientConfig, pki_types::ServerName},
};
use types::api::{ApiErrorFrame, HostCommand, NodeCommand};
use types::{P2pPoints, api::ERRORCODE};

#[derive(Deserialize)]
pub struct Config {
    host: IpAddr,
    port: u16,
    #[serde(rename = "p2p-server-addr")]
    p2p_server_addr: SocketAddr,
    #[serde(rename = "trust-all-certs")]
    trust_all_certs: bool,
}

pub struct NodeConfig {
    addr: SocketAddr,
    p2p_server_addr: SocketAddr,
    trust_all_certs: bool,
}

impl NodeConfig {
    pub fn from_yaml(path: impl AsRef<Path>) -> Self {
        // let relative_path = path.as_ref().parent().unwrap_or_else(|| Path::new("."));

        let config: Config = Figment::new().merge(Yaml::file(&path)).extract().unwrap();

        NodeConfig {
            addr: SocketAddr::new(config.host, config.port),
            p2p_server_addr: config.p2p_server_addr,
            trust_all_certs: config.trust_all_certs,
        }
    }
}

pub enum Connect {
    P2PHost(),
}

pub struct Node {
    addr: SocketAddr,
    p2p_server_addr: SocketAddr,
    trust_all_certs: bool,
    points: Arc<RwLock<P2pPoints>>,
    _connection: Option<u8>,
}

impl Node {
    pub fn init_tracing(&self) -> Result<(), NodeError> {
        #[cfg(debug_assertions)]
        let level = "debug";
        #[cfg(not(debug_assertions))]
        let level = "info";

        let env_level = std::env::var("RUST_LOG").unwrap_or_else(|_| level.to_string());

        if tracing::subscriber::set_global_default(
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::from_str(&env_level).unwrap_or(tracing::Level::INFO))
                .with_target(true)
                .with_file(true)
                .with_line_number(true)
                .with_thread_names(true)
                .with_span_events(FmtSpan::NONE)
                .finish(),
        ).is_err() {
            tracing::info!("tracing skipped")
        }

        Ok(())
    }

    pub fn from_config(config: NodeConfig) -> Self {
        let NodeConfig {
            addr,
            p2p_server_addr,
            trust_all_certs,
        } = config;

        Self {
            addr,
            p2p_server_addr,
            trust_all_certs,
            points: Arc::new(RwLock::new(P2pPoints::new())),
            _connection: None,
        }
    }

    pub async fn serve(&mut self) -> Result<(), NodeError> {
        let listener = TcpListener::bind(&self.addr).await?;

        let points = self.points.clone();

        let stream = Node::connect_p2p_host(self.p2p_server_addr, self.trust_all_certs).await?;
        let stream = Arc::new(Mutex::new(stream));

        let addr = self.addr;

        let _handler = tokio::spawn(async move {
            if let Err(e) = async {
                Node::p2p_add_peer(stream.clone(), addr).await?;

                *points.write().await = Node::p2p_get_peers(stream.clone()).await?;

                tracing::info!("{:?}", points.read().await);

                stream.lock().await.write_u8(HostCommand::Bye.as_byte()).await?;

                Ok::<(), NodeError>(())
            }
            .await
            {
                tracing::error!("Node peer update failed ({:?})", e);
            }
        });

        loop {
            tokio::select! {
                res = listener.accept() => {
                    match res {
                        Ok((stream, addr)) => {
                            let points = self.points.clone();

                            tokio::spawn(async move {
                                tracing::info!("connected: {}", addr);
                                if let Err(e) = handle_client(stream, addr, points).await {
                                    tracing::error!("Client error ({}): {:?}", addr, e);
                                }
                            });
                        },
                        Err(e) => {
                            tracing::error!("Accept error: {:?}", e);
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        }
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    tracing::info!("Closing node server...");
                    break;
                }
            }
        }

        Ok(())
    }

    pub async fn connect_p2p_host(
        addr: SocketAddr,
        trust_all_certs: bool,
    ) -> Result<LoggedStream<TlsStream<TcpStream>>, NodeError> {
        let stream = connect_with_tls(addr, trust_all_certs).await?;

        let mut stream = LoggedStream::new(stream, addr);

        stream.write_u8(HostCommand::Hello.as_byte()).await?;

        match timeout(Duration::from_secs(5), stream.read_u8()).await {
            Ok(Ok(cmd)) if cmd == NodeCommand::Hello.as_byte() => Ok(stream),
            Ok(_) => Err(NodeError::ConnectionError(
                "invalid return from host".into(),
            )),
            Err(_) => Err(NodeError::Timeout("no response from host".into())),
        }
    }

    pub async fn p2p_add_peer<S>(
        stream: Arc<Mutex<S>>,
        addr: SocketAddr
    ) -> Result<(), NodeError>
    where
        S: AsyncRead + AsyncWrite + Send + Unpin + 'static,
    {
        let mut stream = stream.lock().await;
        stream.write_u8(HostCommand::AddPeer.as_byte()).await?;

        let mut buf = [0u8; 19];

        match addr {
            SocketAddr::V4(addr) => {
                buf[0..4].copy_from_slice(&addr.ip().octets());
                buf[4..6].copy_from_slice(&addr.port().to_be_bytes());
            }
            SocketAddr::V6(addr) => {
                buf[0..16].copy_from_slice(&addr.ip().octets());
                buf[16..18].copy_from_slice(&addr.port().to_be_bytes());
            }
        }

        stream.write_all(&buf).await?;

        match timeout(Duration::from_secs(5), stream.read_u8()).await {
            Ok(Ok(cmd)) if cmd == NodeCommand::Done.as_byte() => Ok(()),
            Ok(_) => Err(NodeError::ConnectionError(
                "invalid return from host".into(),
            )),
            Err(_) => Err(NodeError::Timeout("no response from host".into())),
        }
    }

    pub async fn p2p_get_peers<S>(stream: Arc<Mutex<S>>) -> Result<P2pPoints, NodeError>
    where
        S: AsyncRead + AsyncWrite + Send + Unpin + 'static,
    {
        let mut stream = stream.lock().await;

        stream.write_u8(HostCommand::Peer.as_byte()).await?;

        let len = stream.read_u32().await?;

        let mut buf = vec![0u8; len as usize];

        match timeout(Duration::from_secs(5), stream.read_exact(&mut buf)).await {
            Ok(_) => Ok(P2pPoints::from_bytes(&buf)?),
            Err(_) => Err(NodeError::Timeout("no response from host".into())),
        }
    }
}

pub async fn handle_client(
    stream: TcpStream,
    addr: SocketAddr,
    points: Arc<RwLock<P2pPoints>>,
) -> Result<(), NodeError> {
    let (reader, mut writer) = tokio::io::split(stream);
    let mut reader = BufReader::new(reader);

    loop {
        let cmd = match reader.read_u8().await {
            Ok(v) => v,
            Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                tracing::info!("client disconnected");
                break;
            }
            Err(e) => return Err(e.into()),
        };

        match NodeCommand::from_byte(cmd) {
            NodeCommand::Hello => {
                writer.write_u8(HostCommand::Hello.as_byte()).await?;
            }
            NodeCommand::AddPeer => {
                points.write().await.insert(addr);
                tracing::info!("peer added: {}", addr);
                writer.write_u8(HostCommand::Done.as_byte()).await?;
            }
            NodeCommand::CheckPeer => {
                // TODO: add Verification
                tracing::info!("peer checking requested: {}", addr);
                writer.write_u8(HostCommand::Done.as_byte()).await?;
            }
            NodeCommand::Peer => {
                let bytes = {
                    let guard = points.read().await;
                    guard.to_bytes()?
                };

                writer.write_u32(bytes.len() as u32).await?;
                writer.write_all(&bytes).await?
            }
            NodeCommand::Bye => {
                writer.write_u8(HostCommand::Bye.as_byte()).await?;
                break;
            }
            NodeCommand::Done => {
                writer.write_u8(HostCommand::Bye.as_byte()).await?;
                break;
            }
            NodeCommand::Error => {
                if cmd != ERRORCODE {
                    let mut msg = [0u8; 127];
                    let n = reader.read(&mut msg).await?;

                    if n == 0 {
                        tracing::info!("Server {} disconnected", addr);
                        break;
                    }

                    let frame = ApiErrorFrame {
                        len: n as u8,
                        cmd,
                        data: &msg,
                    };

                    let buf = frame.to_bytes();

                    tracing::error!("client error: \"{}\"", String::from_utf8_lossy(&buf[2..]));

                    writer.write_all(&buf).await?;
                } else {
                    let n = reader.read_u8().await? as usize;

                    let mut msg = vec![0u8; n];

                    reader.read(&mut msg).await?;

                    tracing::error!("server error: \"{}\"", String::from_utf8_lossy(&msg));

                    writer.write_u8(HostCommand::Bye.as_byte()).await?;
                    break;
                }
            }
        }
    }
    Ok(())
}

pub async fn connect_with_tls(
    addr: SocketAddr,
    trust_all_certs: bool,
) -> Result<TlsStream<TcpStream>, NodeError> {
    let root_store =
        rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let mut config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    if trust_all_certs {
        tracing::info!("[!] trust_all_certs = true (skipping certificate validation)");
        config
            .dangerous()
            .set_certificate_verifier(Arc::new(NoVerifier));
    }
    let config = Arc::new(config);

    let connector = TlsConnector::from(config);
    let domain = ServerName::IpAddress(addr.ip().into());

    let tcp = TcpStream::connect(addr).await?;

    let stream = connector.connect(domain, tcp).await?;

    Ok(stream)
}
