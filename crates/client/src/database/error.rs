use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("no home directory found for the data folder")]
    NoHome,
    #[error("creating the data folder failed: {0}")]
    Folder(#[from] std::io::Error),
    #[error("database error: {0}")]
    Store(#[from] redb::Error),
    #[error("stored data is unreadable: {0}")]
    Data(#[from] serde_json::Error),
}
