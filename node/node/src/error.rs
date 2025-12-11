#[derive(Debug, thiserror::Error)]
pub enum NodeError {
    #[error("initialize error: ({0})")]
    InitializeError(#[source] anyhow::Error),

    #[error("process block error: ({0})")]
    ProcessBlockError(#[source] anyhow::Error),

    #[error("insert block error: ({0})")]
    InsertBlockError(#[source] anyhow::Error),
}
