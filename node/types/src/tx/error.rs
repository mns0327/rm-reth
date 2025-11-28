use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransactionError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SCALE encoding failed: {0}")]
    ScaleError(#[from] parity_scale_codec::Error),

    #[error("transaction out of size")]
    TxSingleSizeError,

    #[error("transaction pool out of size")]
    TxSizeError,

    #[error("TxPool already sealed")]
    TxPoolFinalized,

    #[error("TxPool reached limit")]
    TxPoolReachedLimit,

    #[error("unknown error: ({0})")]
    Unknown(String),
}
