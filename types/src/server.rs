use std::{marker::PhantomData, sync::Arc};

use crate::stream::Stream;
use async_trait::async_trait;
use tokio::sync::Mutex;
use tracing::{debug, error};

#[async_trait]
pub trait Handler<T, E>: Send + Sync
where
    T: Send + 'static,
{
    async fn handle(&self, stream: Arc<Mutex<Stream>>, cmd: u8, value: T) -> Result<bool, E>;
}

pub struct ApiHandlers<T, E, const N: usize = 32>(
    [Option<Box<dyn Handler<T, E> + Send + Sync>>; N],
);

impl<T, E> ApiHandlers<T, E>
where
    T: Send + 'static,
{
    pub fn insert_api<H>(&mut self, key: u8, value: H)
    where
        H: Handler<T, E> + 'static,
    {
        self.0[key as usize] = Some(Box::new(value));
    }
}

pub struct Dispatcher<T, U, E, const N: usize = 32> {
    handlers: ApiHandlers<T, E, N>,
    _unknown: PhantomData<U>,
}

impl<T, U, E> Dispatcher<T, U, E>
where
    U: Handler<T, E> + Default + Send + Sync + 'static,
    T: Send + Clone + 'static,
    E: From<std::io::Error>,
{
    pub fn new() -> Self {
        Self {
            handlers: ApiHandlers(std::array::from_fn(|_| None)),
            _unknown: PhantomData,
        }
    }

    pub fn with_handler<H>(mut self, key: u8, handler: H) -> Self
    where
        H: Handler<T, E> + 'static,
    {
        self.handlers.insert_api(key, handler);
        self
    }

    pub async fn dispatch_loop(&self, stream: Arc<Mutex<Stream>>, value: T) -> Result<(), E> {
        while self
            .process_connection(stream.clone(), value.clone())
            .await?
        {}
        Ok(())
    }

    pub async fn process_connection(
        &self,
        stream: Arc<Mutex<Stream>>,
        value: T,
    ) -> Result<bool, E> {
        let mut guard = stream.lock().await;
        let cmd = match guard.read_u8().await {
            Ok(v) => v,
            Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                tracing::info!("client disconnected");
                return Ok(false);
            }
            Err(e) => return Err(e.into()),
        };
        drop(guard);

        let should_continue = self.dispatch(cmd, stream.clone(), value).await?;
        Ok(should_continue)
    }

    pub async fn dispatch(&self, key: u8, stream: Arc<Mutex<Stream>>, value: T) -> Result<bool, E> {
        debug!(target: "dispatcher", ?key, "dispatching command");

        let should_continue = if let Some(handler) = &self.handlers.0[key as usize] {
            handler.handle(stream, key, value).await?
        } else {
            error!(target: "dispatcher", ?key, "unknown handler key");
            let unknown = U::default();
            unknown.handle(stream, key, value).await?
        };
        Ok(should_continue)
    }
}
