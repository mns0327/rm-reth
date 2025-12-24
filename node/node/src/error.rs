use storage::error::StorageError;

#[derive(Debug, thiserror::Error)]
pub enum NodeError {
    #[error("initialize error: ({0})")]
    InitializeError(#[source] anyhow::Error),

    #[error("process block error: ({0})")]
    ProcessBlockError(#[source] anyhow::Error),

    #[error("insert block error: ({0})")]
    InsertBlockError(#[source] anyhow::Error),

    #[error("block not exist error: (id: {0})")]
    BlockNotExist(u64),

    #[error("storage error: ({0})")]
    StorageError(#[from] StorageError),

    #[error("invaild extra data while mining block")]
    InvalidExtraData,

    #[error("mempool full error")]
    MempoolFull,
}
