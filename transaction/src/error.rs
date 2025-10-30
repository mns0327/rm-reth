use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransactionError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] bincode::error::EncodeError),

    #[error("deserialization error: {0}")]
    Deserialization(#[from] bincode::error::DecodeError),

    #[error("transaction out of size")]
    TxSingleSizeError,

    #[error("transaction pool out of size")]
    TxSizeError,

    #[error("TxPool already sealed")]
    TxPoolFinalized,

    #[error("TxPool reached limit")]
    TxPoolReachedLimit,

    #[error("unknown error")]
    Unknown,
}
