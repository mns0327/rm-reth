use thiserror::Error;

#[derive(Debug, Error)]
pub enum DispatchError {
    #[error("timeout")]
    Timeout,

    #[error("concurrency limit reached")]
    Overloaded,

    #[error(transparent)]
    Inner(#[from] anyhow::Error),
}
