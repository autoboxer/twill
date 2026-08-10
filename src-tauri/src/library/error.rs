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

    #[error("{field} {message}")]
    InvalidContent {
        field: &'static str,
        message: String,
    },

    #[error("the selected image cannot be larger than {maximum_megabytes} MB")]
    ImageTooLarge { maximum_megabytes: usize },

    #[error("the selected image format is not supported")]
    UnsupportedImage,

    #[error("the selected image dimensions are too large")]
    ImageDimensionsTooLarge,

    #[error("media {0} was not found")]
    MediaNotFound(String),

    #[error("card {0} was not found")]
    CardNotFound(String),

    #[error("card {card_id} is not due until {due_at}")]
    CardNotDue { card_id: String, due_at: i64 },

    #[error("scheduler configuration {0} is not supported")]
    UnsupportedSchedulerConfiguration(String),

    #[error("stored card scheduling state is not valid: {0}")]
    InvalidSchedulingState(String),

    #[error("stored grading mode is not valid: {0}")]
    InvalidGradingMode(String),

    #[error("the scheduler produced an invalid interval")]
    InvalidSchedule,

    #[error("stored media file for digest {expected_digest} failed its integrity check")]
    MediaIntegrity { expected_digest: String },

    #[error(transparent)]
    Data(#[from] DataError),

    #[error("local library data could not be read or written: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("local rich content could not be encoded or decoded: {0}")]
    Json(#[from] serde_json::Error),

    #[error("the FSRS scheduler could not calculate the next review: {0}")]
    Scheduler(#[from] fsrs::FSRSError),
}
