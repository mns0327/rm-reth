use std::marker::PhantomData;

use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};

#[async_trait]
pub trait Handler<T, S>: Send + Sync
where
    S: AsyncRead + AsyncWrite + Send + Unpin + Send + 'static,
    T: Send + 'static,
{
    async fn handle(&self, mut stream: S, value: T);
}

pub struct ApiHandlers<T, S>([Option<Box<dyn Handler<T, S> + Send + Sync>>; 256]);

impl<T, S> ApiHandlers<T, S>
where
    S: AsyncRead + AsyncWrite + Send + Unpin + Send + 'static,
    T: Send + 'static,
{
    pub fn insert_api<H>(&mut self, key: u8, value: H)
    where
        H: Handler<T, S> + 'static,
    {
        self.0[key as usize] = Some(Box::new(value));
    }
}

pub struct Dispatcher<T, S, U> {
    handlers: ApiHandlers<T, S>,
    _unknown: PhantomData<U>,
}

impl<T, S, U> Dispatcher<T, S, U>
where
    U: Handler<T, S> + Default + Send + Sync + 'static,
    T: Send + 'static,
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    pub fn new() -> Self {
        Self {
            handlers: ApiHandlers(std::array::from_fn(|_| None)),
            _unknown: PhantomData,
        }
    }

    pub async fn dispatch(&self, key: u8, stream: S, value: T) {
        if let Some(handler) = &self.handlers.0[key as usize] {
            handler.handle(stream, value).await;
        } else {
            let unknown = U::default();
            unknown.handle(stream, value).await;
        }
    }
}
