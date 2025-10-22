use std::net::SocketAddr;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tracing::debug;

pub struct LoggedStream<T> {
    inner: T,
    addr: SocketAddr, 
}

impl<T> LoggedStream<T> {
    pub fn new(inner: T, addr: impl Into<SocketAddr>) -> Self {
        Self {
            inner,
            addr: addr.into(),
        }
    }
}

impl<T: AsyncRead + Unpin> AsyncRead for LoggedStream<T> {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let poll = std::pin::Pin::new(&mut self.inner).poll_read(cx, buf);

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

impl<T: AsyncWrite + Unpin> AsyncWrite for LoggedStream<T> {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        let poll = std::pin::Pin::new(&mut self.inner).poll_write(cx, buf);
        
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

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}
