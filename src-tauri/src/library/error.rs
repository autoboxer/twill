use thiserror::Error;

use crate::data::DataError;

pub type LibraryResult<T> = Result<T, LibraryError>;

#[derive(Debug, Error)]
pub enum LibraryError {
    #[error("{field} cannot be empty")]
    EmptyValue { field: &'static str },

    #[error("{field} cannot be longer than {maximum} characters")]
    ValueTooLong {
        field: &'static str,
        maximum: usize,
    },

    #[error("a {kind} named '{name}' already exists")]
    DuplicateName {
        kind: &'static str,
        name: String,
    },

    #[error("concept {0} was not found")]
    ConceptNotFound(String),

    #[error("{kind} {id} was not found")]
    OrganizationNotFound { kind: &'static str, id: String },

    #[error("selected {kind} {id} was not found")]
    InvalidSelection { kind: &'static str, id: String },

    #[error(transparent)]
    Data(#[from] DataError),

    #[error("local library data could not be read or written: {0}")]
    Database(#[from] rusqlite::Error),
}
