#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("slice error: {0}")]
    SliceError(#[from] std::array::TryFromSliceError),
    #[error("decode error: {0}")]
    DecodeError(#[from] bincode::error::DecodeError),
    #[error("slice error: {0}")]
    EncodeError(#[from] bincode::error::EncodeError),
}
