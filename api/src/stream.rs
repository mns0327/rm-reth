use hex;
use std::{
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    io::{self, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf},
    net::TcpStream,
};
use tracing::debug;

pub struct Stream {
    inner: InnerStream,
    pub addr: SocketAddr,
}

impl Stream {
    pub fn new(stream: impl Into<InnerStream>, addr: SocketAddr) -> Self {
        Stream {
            inner: stream.into(),
            addr,
        }
    }

    #[inline]
    pub async fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        AsyncWriteExt::write_all(self, buf).await
    }

    #[inline]
    pub async fn write_u8(&mut self, n: u8) -> io::Result<()> {
        AsyncWriteExt::write_u8(self, n).await
    }

    #[inline]
    pub async fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        AsyncReadExt::read_exact(self, buf).await
    }

    #[inline]
    pub async fn read_u8(&mut self) -> io::Result<u8> {
        AsyncReadExt::read_u8(self).await
    }
}

pub enum InnerStream {
    Tcp(TcpStream),
    ClientTls(Box<tokio_rustls::client::TlsStream<TcpStream>>),
    ServerTls(Box<tokio_rustls::server::TlsStream<TcpStream>>),
}

impl Into<InnerStream> for TcpStream {
    #[inline]
    fn into(self) -> InnerStream {
        InnerStream::Tcp(self)
    }
}

impl Into<InnerStream> for tokio_rustls::server::TlsStream<TcpStream> {
    #[inline]
    fn into(self) -> InnerStream {
        InnerStream::ServerTls(Box::new(self))
    }
}

impl Into<InnerStream> for tokio_rustls::client::TlsStream<TcpStream> {
    #[inline]
    fn into(self) -> InnerStream {
        InnerStream::ClientTls(Box::new(self))
    }
}

impl AsyncRead for Stream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let poll = match &mut self.inner {
            InnerStream::Tcp(s) => Pin::new(s).poll_read(cx, buf),
            InnerStream::ClientTls(s) => Pin::new(s.as_mut()).poll_read(cx, buf),
            InnerStream::ServerTls(s) => Pin::new(s.as_mut()).poll_read(cx, buf),
        };

        if tracing::enabled!(tracing::Level::DEBUG) {
            if let std::task::Poll::Ready(Ok(())) = &poll {
                let before = buf.filled().len();
                let after = buf.filled().len();
                let n = after - before;
                if n > 0 {
                    let data = &buf.filled()[after - n..after];
                    let preview = hex::encode_upper(&data[..n.min(64)]);
                    debug!(target: "tcp_rx",
                        conn=%self.addr,
                        "recv {} bytes: {}{}",
                        n,
                        preview,
                        if n > 64 { " ..." } else { "" }
                    );
                }
            }
        }
        poll
    }
}

impl AsyncWrite for Stream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let poll = match &mut self.inner {
            InnerStream::Tcp(s) => Pin::new(s).poll_write(cx, buf),
            InnerStream::ClientTls(s) => Pin::new(s.as_mut()).poll_write(cx, buf),
            InnerStream::ServerTls(s) => Pin::new(s.as_mut()).poll_write(cx, buf),
        };

        if tracing::enabled!(tracing::Level::DEBUG) {
            if let std::task::Poll::Ready(Ok(n)) = &poll {
                if *n > 0 {
                    let preview = hex::encode_upper(&buf[..(*n).min(64)]);
                    debug!(target: "tcp_tx",
                        conn=%self.addr,
                        "sent {} bytes: {}{}",
                        n,
                        preview,
                        if *n > 64 { " ..." } else { "" }
                    );
                }
            }
        }
        poll
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match &mut self.inner {
            InnerStream::Tcp(s) => Pin::new(s).poll_flush(cx),
            InnerStream::ClientTls(s) => Pin::new(s.as_mut()).poll_flush(cx),
            InnerStream::ServerTls(s) => Pin::new(s.as_mut()).poll_flush(cx),
        }
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match &mut self.inner {
            InnerStream::Tcp(s) => Pin::new(s).poll_shutdown(cx),
            InnerStream::ClientTls(s) => Pin::new(s.as_mut()).poll_shutdown(cx),
            InnerStream::ServerTls(s) => Pin::new(s.as_mut()).poll_shutdown(cx),
        }
    }
}
