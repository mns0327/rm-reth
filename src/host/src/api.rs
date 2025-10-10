use figment::{providers::{Format, Yaml}, Figment};
use rustls::{pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer}, ServerConfig};
use serde::Deserialize;
use std::{path::{Path, PathBuf}, sync::Arc};
use tokio_rustls::TlsAcceptor;
use tokio::{net::TcpListener, io::AsyncWriteExt};

#[derive(Deserialize)]
pub struct Settings {
    pub host: String,
    pub port: u16,
    pub certificate: PathBuf,
    pub private_key: PathBuf
}

pub struct HostServerConfig {
    pub certs: Vec<CertificateDer<'static>>,
    pub key: Option<PrivateKeyDer<'static>>,
    pub host: String,
    pub port: u16
}

impl HostServerConfig {
    pub fn new() -> Self {
        Self {
            certs: vec![],
            key: None,
            host: String::new(),
            port: 4433
        }
    }

    pub fn with_port(&mut self, port: u16) -> &mut Self {
        self.port = port;
        self
    }

    pub fn with_host(&mut self, host: impl Into<String>) -> &mut Self {
        self.host = host.into();
        self
    }

    pub fn add_certificate_file(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.certs.push(CertificateDer::from_pem_file(path).unwrap());
        self
    }

    pub fn with_private_key(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.key = Some(PrivateKeyDer::from_pem_file(path).unwrap());
        self
    }

    pub fn from_yaml(path: impl AsRef<Path>) -> Self {
        let relative_path = path.as_ref().parent().unwrap_or_else(|| Path::new("."));

        let config: Settings = Figment::new()
            .merge(Yaml::file(&path))
            .extract()
            .unwrap();

        let mut result = HostServerConfig::new();

        result.with_port(config.port)
            .with_host(config.host)
            .add_certificate_file(relative_path.join(config.certificate))
            .with_private_key(relative_path.join(config.private_key));

        result
    }
}

pub struct HostServer {
    config: Arc<ServerConfig>,
    host: String,
    port: u16,
    _data: Vec<usize>
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
            _data: vec![]
        }
    }

    pub async fn serve(&self) -> anyhow::Result<()> {
        let listener = TcpListener::bind((self.host.clone(), self.port)).await?;
        let acceptor = TlsAcceptor::from(self.config.clone());

        loop {
            let (stream, _) = listener.accept().await?;
            let acceptor = acceptor.clone();

            tokio::spawn(async move {
                let mut stream = acceptor.accept(stream).await.unwrap();
                stream.write_all(b"Testing...").await.unwrap();
            });
        }
    }
}