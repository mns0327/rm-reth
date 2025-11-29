use crate::error::HostApiError;
use types::api::{
    points::P2pPoints,
    command::{ApiErrorFrame, ERRORCODE, HostCommand, NodeCommand},
    handler::{Dispatcher, Handler},
    stream::Stream,
};
use async_trait::async_trait;
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::{Arc, LazyLock},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{Mutex, RwLock},
};

pub const DISPATCHER: LazyLock<Dispatcher<Arc<RwLock<P2pPoints>>, ApiError, HostApiError>> =
    LazyLock::new(|| {
        Dispatcher::new()
            .with_handler(HostCommand::Hello.as_byte(), ApiHello)
            .with_handler(HostCommand::AddPeer.as_byte(), ApiAddPeer)
            .with_handler(HostCommand::Peer.as_byte(), ApiPeer)
            .with_handler(HostCommand::Done.as_byte(), ApiDone)
            .with_handler(HostCommand::Bye.as_byte(), ApiBye)
    });

struct ApiHello;
struct ApiAddPeer;
struct ApiPeer;
struct ApiBye;
struct ApiDone;
#[derive(Default)]
pub struct ApiError;

#[async_trait]
impl Handler<Arc<RwLock<P2pPoints>>, HostApiError> for ApiHello {
    async fn handle(
        &self,
        stream: Arc<Mutex<Stream>>,
        _cmd: u8,
        _value: Arc<RwLock<P2pPoints>>,
    ) -> Result<bool, HostApiError> {
        let mut stream = stream.lock().await;

        stream.write_u8(NodeCommand::Hello.as_byte()).await?;

        Ok(true)
    }
}

#[async_trait]
impl Handler<Arc<RwLock<P2pPoints>>, HostApiError> for ApiAddPeer {
    async fn handle(
        &self,
        stream: Arc<Mutex<Stream>>,
        _cmd: u8,
        value: Arc<RwLock<P2pPoints>>,
    ) -> Result<bool, HostApiError> {
        let mut stream = stream.lock().await;
        let points = value;

        let mut buf = [0u8; 19];

        stream.read(&mut buf).await?;

        let new_peer_addr = if buf[16..18] == [0u8; 2] {
            let ip = Ipv4Addr::from(<[u8; 4]>::try_from(&buf[0..4]).unwrap());
            let port = u16::from_be_bytes(<[u8; 2]>::try_from(&buf[4..6]).unwrap());
            SocketAddr::new(IpAddr::V4(ip), port)
        } else {
            let ip = Ipv6Addr::from(<[u8; 16]>::try_from(&buf[0..16]).unwrap());
            let port = u16::from_be_bytes(<[u8; 2]>::try_from(&buf[16..18]).unwrap());
            SocketAddr::new(IpAddr::V6(ip), port)
        };

        let mut peer_stream = TcpStream::connect(new_peer_addr).await?;

        peer_stream
            .write_u8(NodeCommand::CheckPeer.as_byte())
            .await?;

        let cmd = peer_stream.read_u8().await?;

        if cmd == NodeCommand::Done.as_byte() {
            points.write().await.insert(new_peer_addr);
            tracing::info!("peer added: {}", new_peer_addr);
            stream.write_u8(NodeCommand::Done.as_byte()).await?;
        } else {
            tracing::error!("peer adding failed");
        }

        Ok(true)
    }
}

#[async_trait]
impl Handler<Arc<RwLock<P2pPoints>>, HostApiError> for ApiPeer {
    async fn handle(
        &self,
        stream: Arc<Mutex<Stream>>,
        _cmd: u8,
        value: Arc<RwLock<P2pPoints>>,
    ) -> Result<bool, HostApiError> {
        let mut stream = stream.lock().await;
        let points = value;

        let bytes = {
            let guard = points.read().await;
            guard.to_bytes()
        };

        stream.write_u32(bytes.len() as u32).await?;
        stream.write_all(&bytes).await?;

        Ok(true)
    }
}

#[async_trait]
impl Handler<Arc<RwLock<P2pPoints>>, HostApiError> for ApiDone {
    async fn handle(
        &self,
        _stream: Arc<Mutex<Stream>>,
        _cmd: u8,
        _value: Arc<RwLock<P2pPoints>>,
    ) -> Result<bool, HostApiError> {
        Ok(true)
    }
}

#[async_trait]
impl Handler<Arc<RwLock<P2pPoints>>, HostApiError> for ApiBye {
    async fn handle(
        &self,
        stream: Arc<Mutex<Stream>>,
        _cmd: u8,
        _value: Arc<RwLock<P2pPoints>>,
    ) -> Result<bool, HostApiError> {
        let mut stream = stream.lock().await;

        stream.write_u8(NodeCommand::Bye.as_byte()).await?;

        Ok(false)
    }
}

#[async_trait]
impl Handler<Arc<RwLock<P2pPoints>>, HostApiError> for ApiError {
    async fn handle(
        &self,
        stream: Arc<Mutex<Stream>>,
        cmd: u8,
        _value: Arc<RwLock<P2pPoints>>,
    ) -> Result<bool, HostApiError> {
        let mut stream = stream.lock().await;

        if cmd != ERRORCODE {
            let mut msg = [0u8; 127];
            let n = stream.read(&mut msg).await?;

            if n == 0 {
                tracing::info!("Server {} disconnected", stream.addr);
                return Ok(false);
            }

            let frame = ApiErrorFrame {
                len: n as u8,
                cmd,
                data: &msg.as_slice(),
            };

            let buf = frame.to_bytes();

            tracing::error!("client error: \"{}\"", String::from_utf8_lossy(&buf[2..]));

            stream.write_all(&buf).await?;
            Ok(false)
        } else {
            let n = stream.read_u8().await? as usize;

            let mut msg = vec![0u8; n];

            stream.read(&mut msg).await?;

            tracing::error!("server error: \"{}\"", String::from_utf8_lossy(&msg));

            stream.write_u8(HostCommand::Bye.as_byte()).await?;
            Ok(false)
        }
    }
}
