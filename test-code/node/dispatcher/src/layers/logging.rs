use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Instant,
};

use tower::{Layer, Service};

use crate::command::{Command, CommandLog};

#[derive(Clone)]
pub struct LoggingLayer;

impl<S> Layer<S> for LoggingLayer {
    type Service = LoggingService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LoggingService { inner }
    }
}

#[derive(Clone)]
pub struct LoggingService<S> {
    inner: S,
}

impl<S> Service<Command> for LoggingService<S>
where
    S: Service<Command> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, cmd: Command) -> Self::Future {
        let mut inner = self.inner.clone();

        let span = tracing::info_span!("command", name = cmd.name(), summary = cmd.summary(),);

        Box::pin(async move {
            let _enter = span.enter();
            let start = Instant::now();

            let result = inner.call(cmd).await;

            tracing::info!(
                latency_ms = start.elapsed().as_millis(),
                success = result.is_ok(),
            );

            result
        })
    }
}
