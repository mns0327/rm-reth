#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("database error: {0}")]
    Database(#[from] redb::DatabaseError),

    #[error("transaction failed: {0}")]
    Transaction(#[from] redb::TransactionError),

    #[error("table error: {0}")]
    Table(#[from] redb::TableError),

    #[error("storage access error: {0}")]
    StorageError(#[from] redb::StorageError),

    #[error("commit error: {0}")]
    CommitError(#[from] redb::CommitError),

    #[error("other: {0}")]
    Other(String),
}
