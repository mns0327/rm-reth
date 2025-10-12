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
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::RwLock,
};
use tokio_rustls::TlsAcceptor;
use types::P2pPoints;

use crate::error::HostApiError;

#[derive(Clone, Copy)]
enum Command {
    Hello,
    AddPeer,
    Peer,
    Bye,
    Unknown,
}

static CMD_TABLE: phf::Map<&'static str, Command> = phf::phf_map! {
    "HELLO" => Command::Hello,
    "ADDPEER" => Command::AddPeer,
    "PEER" => Command::Peer,
    "BYE" => Command::Bye
};

fn parse_command(line: &str) -> Command {
    let cmd = line.split_whitespace().next().unwrap_or("");
    *CMD_TABLE
        .get(cmd.to_uppercase().as_str())
        .unwrap_or(&Command::Unknown)
}

#[derive(Deserialize)]
pub struct Settings {
    pub host: IpAddr,
    pub port: u16,
    pub certificate: PathBuf,
    pub private_key: PathBuf,
}

pub struct HostServerConfig {
    pub certs: Vec<CertificateDer<'static>>,
    pub key: Option<PrivateKeyDer<'static>>,
    pub host: IpAddr,
    pub port: u16,
}

impl HostServerConfig {
    pub fn new() -> Self {
        Self {
            certs: vec![],
            key: None,
            host: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port: 4433,
        }
    }

    pub fn with_port(&mut self, port: u16) -> &mut Self {
        self.port = port;
        self
    }

    pub fn with_host(&mut self, host: impl Into<IpAddr>) -> &mut Self {
        self.host = host.into();
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

        let config: Settings = Figment::new().merge(Yaml::file(&path)).extract().unwrap();

        let mut result = HostServerConfig::new();

        result
            .with_port(config.port)
            .with_host(config.host)
            .add_certificate_file(relative_path.join(config.certificate))
            .with_private_key(relative_path.join(config.private_key));

        result
    }
}

pub struct HostServer {
    config: Arc<ServerConfig>,
    host: IpAddr,
    port: u16,
    points: Arc<RwLock<P2pPoints>>,
}

impl HostServer {
    pub fn from_config(config: HostServerConfig) -> Self {
        let HostServerConfig {
            certs,
            key,
            host,
            port,
        } = config;

        let tls_config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key.unwrap())
            .unwrap();

        HostServer {
            config: Arc::new(tls_config),
            host,
            port,
            points: Arc::new(RwLock::new(P2pPoints::new())),
        }
    }

    pub async fn serve(&self) -> Result<(), HostApiError> {
        let listener = TcpListener::bind((self.host.clone(), self.port)).await?;
        let acceptor = TlsAcceptor::from(self.config.clone());

        println!("Server running on {}:{}", self.host, self.port);

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

            let mut line = String::new();

            loop {
                line.clear();

                let n = match reader.read_line(&mut line).await {
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("Read error from {}: {:?}", addr, e);
                        break;
                    }
                };
                if n == 0 {
                    println!("Client {} disconnected", addr);
                    break;
                }

                let cmd = parse_command(&line);

                match cmd {
                    Command::Hello => {
                        writer.write_all(b"Hello\n").await?;
                        continue;
                    }
                    Command::AddPeer => {
                        points.write().await.insert(addr);
                    }
                    Command::Peer => {
                        let bytes = {
                            let guard = points.read().await;
                            guard.to_bytes()
                        };

                        writer.write_all(&bytes).await?
                    }
                    Command::Bye => {
                        writer.write_all(b"Bye\n").await?;
                        break;
                    }
                    Command::Unknown => {
                        let msg = format!("Unknown: {}\n", line.trim());
                        writer.write_all(msg.as_bytes()).await?;
                    }
                }
                writer.write_all(b"Done\n").await?;
            }
        }
        Err(e) => {
            eprintln!("TLS accept error from {}: {:?}", addr, e);
            return Err(HostApiError::ConnectionError(e.to_string()));
        }
    }
    Ok(())
}
