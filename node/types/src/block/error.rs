use rand::rand_core::OsError;
use thiserror::Error;
use crate::tx::error::TransactionError;

#[derive(Debug, Error)]
pub enum BlockError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SCALE encoding failed: {0}")]
    ScaleError(#[from] parity_scale_codec::Error),

    #[error("generating rand error: {0}")]
    OsRandError(#[from] OsError),

    #[error("tx error")]
    TransactionError(#[from] TransactionError),

    #[error("type error")]
    TypeError(#[from] crate::error::TypeError),

    #[error("transaction out of size")]
    TxSingleSizeError,

    #[error("transaction pool out of size")]
    TxSizeError,

    #[error("invalid state: ({0})")]
    InvalidState(String),

    #[error("unknown error: ({0})")]
    Unknown(String),
}
