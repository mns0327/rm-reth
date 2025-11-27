#[derive(thiserror::Error, Debug)]
pub enum HostApiError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("connection error: {0}")]
    ConnectionError(String),
    #[error("api error: {0}")]
    ApiError(#[from] api::error::ApiError),
}
