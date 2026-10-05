use std::io;

use thiserror::Error;

use crate::data::DataError;

pub type BackupResult<T> = Result<T, BackupError>;

#[derive(Debug, Error)]
pub enum BackupError {
    #[error("Choose a file outside Twill's data directory")]
    InvalidDestination,

    #[error("That file already exists. Choose a new filename")]
    DestinationExists,

    #[error("Local data failed its integrity check: {0}")]
    Integrity(&'static str),

    #[error("Local data could not be accessed: {0}")]
    Data(#[from] DataError),

    #[error("The database could not be read: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("The file could not be read or written: {0}")]
    Io(#[from] io::Error),

    #[error("The archive could not be written: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("Local content could not be exported: {0}")]
    Json(#[from] serde_json::Error),
}
