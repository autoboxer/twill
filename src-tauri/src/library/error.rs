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

    #[error("template {0} was not found")]
    TemplateNotFound(String),

    #[error("selected {kind} {id} was not found")]
    InvalidSelection { kind: &'static str, id: String },

    #[error("{field} {message}")]
    InvalidContent {
        field: &'static str,
        message: String,
    },

    #[error("{field} {message}")]
    InvalidTemplate {
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

    #[error("stored retrieval form kind is not valid: {0}")]
    InvalidRetrievalFormKind(String),

    #[error("stored retrieval form is not valid")]
    InvalidRetrievalForm,

    #[error("a concept must have at least one retrieval form")]
    MissingRetrievalForm,

    #[error("type answer cards require at least one accepted answer")]
    MissingAcceptedAnswer,

    #[error("type answer cards cannot have more than {maximum} accepted answers")]
    TooManyAcceptedAnswers { maximum: usize },

    #[error("accepted answers must be unique")]
    DuplicateAcceptedAnswer,

    #[error("this template is used by {retrieval_form_count} active retrieval forms")]
    TemplateInUse { retrieval_form_count: i64 },

    #[error("stored grading mode is not valid: {0}")]
    InvalidGradingMode(String),

    #[error("stored startup destination is not valid: {0}")]
    InvalidStartupDestination(String),

    #[error("stored {field} device preference is not valid: {value}")]
    InvalidDevicePreference {
        field: &'static str,
        value: String,
    },

    #[error("target retention must be between {minimum}% and {maximum}%")]
    InvalidDesiredRetention { minimum: i64, maximum: i64 },

    #[error("maximum interval must be between {minimum} and {maximum} days")]
    InvalidMaximumInterval { minimum: i64, maximum: i64 },

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
