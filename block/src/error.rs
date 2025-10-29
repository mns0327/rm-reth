use rand::rand_core::OsError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlockError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] bincode::error::EncodeError),

    #[error("deserialization error: {0}")]
    Deserialization(#[from] bincode::error::DecodeError),

    #[error("generating rand error: {0}")]
    OsRandError(#[from] OsError),

    #[error("transaction out of size")]
    TxSingleSizeError,

    #[error("transaction pool out of size")]
    TxSizeError,

    #[error("unknown error")]
    Unknown,
}
