use std::io;

use thiserror::Error;

pub type DataResult<T> = Result<T, DataError>;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("local data could not be read or written: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("the local data schema could not be updated: {0}")]
    Schema(#[from] rusqlite_migration::Error),

    #[error("the local data directory could not be prepared: {0}")]
    Io(#[from] io::Error),

    #[error("the local data connection is unavailable")]
    ConnectionUnavailable,

    #[error("the system clock cannot produce a valid local timestamp")]
    InvalidSystemTime,

    #[error("entity {0} was not found")]
    EntityNotFound(String),

    #[error("entity {0} has been deleted")]
    EntityDeleted(String),

    #[error("the database contains an unknown entity kind: {0}")]
    UnknownEntityKind(String),

    #[error("the database contains an unknown change operation: {0}")]
    UnknownChangeOperation(String),
}
