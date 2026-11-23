use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("no home directory found for the data folder")]
    NoHome,
    #[error("opening the database failed: {0}")]
    Open(surrealdb::Error),
    #[error("selecting the namespace failed: {0}")]
    Namespace(surrealdb::Error),
}
