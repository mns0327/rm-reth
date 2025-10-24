use crate::api::DISPATCHER;
use crate::error::HostApiError;
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
    net::{TcpListener, TcpStream},
    sync::{Mutex, RwLock},
};
use tokio_rustls::TlsAcceptor;
use types::P2pPoints;

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

        tracing::info!("Server running on {}", self.addr);

        loop {
            tokio::select! {
                res = listener.accept() => {
                    match res {
                        Ok((stream, addr)) => {
                            let acceptor = acceptor.clone();
                            let points = self.points.clone();

                            tokio::spawn(async move {
                                tracing::info!("connected: {}", addr);
                                if let Err(e) = handle_client(acceptor, stream, addr, points).await {
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
                    tracing::info!("Closing host server...");
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
            let stream = Arc::new(Mutex::new(types::stream::Stream::new(stream, addr)));

            DISPATCHER.dispatch_loop(stream, points).await?;
        }
        Err(e) => {
            tracing::error!("TLS accept error from {}: {:?}", addr, e);
            return Err(HostApiError::ConnectionError(e.to_string()));
        }
    }
    Ok(())
}
