use figment::{
    Figment,
    providers::{Format, Yaml},
};
use rustls::{
    ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
};
use serde::Deserialize;
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::RwLock,
};
use tokio_rustls::TlsAcceptor;
use types::{
    P2pPoints,
    api::{ApiErrorFrame, ERRORCODE, HostCommand, NodeCommand},
};

use crate::error::HostApiError;

#[derive(Deserialize)]
pub struct Config {
    pub host: IpAddr,
    pub port: u16,
    pub certificate: PathBuf,
    pub private_key: PathBuf,
}

pub struct HostServerConfig {
    pub certs: Vec<CertificateDer<'static>>,
    pub key: Option<PrivateKeyDer<'static>>,
    pub addr: SocketAddr,
}

impl HostServerConfig {
    pub fn new() -> Self {
        Self {
            certs: vec![],
            key: None,
            addr: SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 4433),
        }
    }

    pub fn with_addr(&mut self, addr: impl Into<SocketAddr>) -> &mut Self {
        self.addr = addr.into();
        self
    }

    pub fn add_certificate_file(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.certs
            .push(CertificateDer::from_pem_file(path).unwrap());
        self
    }

    pub fn with_private_key(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.key = Some(PrivateKeyDer::from_pem_file(path).unwrap());
        self
    }

    pub fn from_yaml(path: impl AsRef<Path>) -> Self {
        let relative_path = path.as_ref().parent().unwrap_or_else(|| Path::new("."));

        let config: Config = Figment::new().merge(Yaml::file(&path)).extract().unwrap();

        let mut result = HostServerConfig::new();

        result
            .with_addr(SocketAddr::new(config.host, config.port))
            .add_certificate_file(relative_path.join(config.certificate))
            .with_private_key(relative_path.join(config.private_key));

        result
    }
}

pub struct HostServer {
    config: Arc<ServerConfig>,
    addr: SocketAddr,
    points: Arc<RwLock<P2pPoints>>,
}

impl HostServer {
    pub fn from_config(config: HostServerConfig) -> Self {
        let HostServerConfig { certs, key, addr } = config;

        let tls_config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key.unwrap())
            .unwrap();

        HostServer {
            config: Arc::new(tls_config),
            addr,
            points: Arc::new(RwLock::new(P2pPoints::new())),
        }
    }

    pub async fn serve(&self) -> Result<(), HostApiError> {
        let listener = TcpListener::bind(self.addr).await?;
        let acceptor = TlsAcceptor::from(self.config.clone());

        println!("Server running on {}", self.addr);

        loop {
            tokio::select! {
                res = listener.accept() => {
                    match res {
                        Ok((stream, addr)) => {
                            let acceptor = acceptor.clone();
                            let points = self.points.clone();

                            tokio::spawn(async move {
                                println!("connected: {}", addr);
                                if let Err(e) = handle_client(acceptor, stream, addr, points).await {
                                    eprintln!("Client error ({}): {:?}", addr, e);
                                }
                            });
                        },
                        Err(e) => {
                            eprintln!("Accept error: {:?}", e);
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        }
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    println!("Closing host server...");
                    break;
                }
            }
        }
        Ok(())
    }
}

async fn handle_client(
    acceptor: TlsAcceptor,
    stream: TcpStream,
    addr: SocketAddr,
    points: Arc<RwLock<P2pPoints>>,
) -> Result<(), HostApiError> {
    match acceptor.accept(stream).await {
        Ok(stream) => {
            let (reader, mut writer) = tokio::io::split(stream);
            let mut reader = BufReader::new(reader);

            loop {
                let cmd = match reader.read_u8().await {
                    Ok(v) => v,
                    Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                        println!("client disconnected");
                        break;
                    }
                    Err(e) => return Err(e.into()),
                };

                match HostCommand::from_byte(cmd) {
                    HostCommand::Hello => {
                        writer.write_u8(NodeCommand::Hello.as_byte()).await?;
                    }
                    HostCommand::AddPeer => {
                        let mut buf = [0u8; 19];

                        reader.read(&mut buf).await?;

                        let new_peer_addr = if buf[16..18] == [0u8; 2] {
                            let ip = Ipv4Addr::from(<[u8; 4]>::try_from(&buf[0..4]).unwrap());
                            let port = u16::from_be_bytes(<[u8; 2]>::try_from(&buf[4..6]).unwrap());
                            SocketAddr::new(IpAddr::V4(ip), port)
                        } else {
                            let ip = Ipv6Addr::from(<[u8; 16]>::try_from(&buf[0..16]).unwrap());
                            let port =
                                u16::from_be_bytes(<[u8; 2]>::try_from(&buf[16..18]).unwrap());
                            SocketAddr::new(IpAddr::V6(ip), port)
                        };

                        let stream = TcpStream::connect(new_peer_addr).await?;

                        let (mut peer_reader, mut peer_writer) = tokio::io::split(stream);

                        peer_writer
                            .write_u8(NodeCommand::CheckPeer.as_byte())
                            .await?;

                        let cmd = peer_reader.read_u8().await?;

                        if cmd == NodeCommand::Done.as_byte() {
                            points.write().await.insert(new_peer_addr);
                            println!("peer added: {}", new_peer_addr);
                            writer.write_u8(NodeCommand::Done.as_byte()).await?;
                        } else {
                            eprintln!("peer adding failed");
                        }
                    }
                    HostCommand::Peer => {
                        let bytes = {
                            let guard = points.read().await;
                            guard.to_bytes()
                        };

                        writer.write_all(&bytes).await?
                    }
                    HostCommand::Bye => {
                        writer.write_u8(NodeCommand::Bye.as_byte()).await?;
                        break;
                    }
                    HostCommand::Done => {
                        writer.write_u8(NodeCommand::Bye.as_byte()).await?;
                        break;
                    }
                    HostCommand::Error => {
                        if cmd != ERRORCODE {
                            let mut msg = [0u8; 127];
                            let n = reader.read(&mut msg).await?;

                            if n == 0 {
                                println!("Server {} disconnected", addr);
                                break;
                            }

                            let frame = ApiErrorFrame {
                                len: n as u8,
                                cmd,
                                data: &msg,
                            };

                            let buf = frame.to_bytes();

                            eprintln!("client error: \"{}\"", String::from_utf8_lossy(&buf[2..]));

                            writer.write_all(&buf).await?;
                        } else {
                            let n = reader.read_u8().await? as usize;

                            let mut msg = vec![0u8; n];

                            reader.read(&mut msg).await?;

                            eprintln!("server error: \"{}\"", String::from_utf8_lossy(&msg));

                            writer.write_u8(HostCommand::Bye.as_byte()).await?;
                            break;
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("TLS accept error from {}: {:?}", addr, e);
            return Err(HostApiError::ConnectionError(e.to_string()));
        }
    }
    Ok(())
}
