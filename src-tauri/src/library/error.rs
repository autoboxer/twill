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

    #[error("CSS snippet {0} was not found")]
    CssSnippetNotFound(String),

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

    #[error("CSS {message}")]
    InvalidCss { message: String },

    #[error("authoring draft {message}")]
    InvalidAuthoringDraft { message: String },

    #[error("the local draft changed; reopen the editor before saving")]
    AuthoringDraftChanged,

    #[error("the saved item changed while you were editing")]
    AuthoringTargetChanged,

    #[error("the saved item was removed while you were editing")]
    AuthoringTargetMissing,

    #[error("the queued edit changed; reopen queued editing before saving")]
    DeferredEditChanged,

    #[error("the image editing session has ended; reopen the editor before importing")]
    AuthoringMediaSessionClosed,

    #[error("deferred edit {message}")]
    InvalidDeferredEdit { message: String },

    #[error("card quality concern {message}")]
    InvalidCardQualityConcern { message: String },

    #[error("card quality concern {0} was not found")]
    CardQualityConcernNotFound(String),

    #[error("card quality concern {0} is already closed")]
    CardQualityConcernClosed(String),

    #[error("an open concern of this kind already exists with a different note; view it in the card quality queue")]
    CardQualityConcernConflict,

    #[error("the review evidence has changed; refresh the card quality queue before dismissing")]
    CardQualityEvidenceChanged,

    #[error("card {0} does not have that active quality signal")]
    CardQualitySignalNotActive(String),

    #[error("stored card quality kind is not valid: {0}")]
    InvalidCardQualityKind(String),

    #[error("stored card quality source is not valid: {0}")]
    InvalidCardQualitySource(String),

    #[error("stored card quality status is not valid: {0}")]
    InvalidCardQualityStatus(String),

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

    #[error("review {0} was not found")]
    ReviewNotFound(String),

    #[error("only the latest grade for a card can be undone")]
    ReviewNotReversible,

    #[error("card {0} is not eligible for pretesting")]
    PretestNotEligible(String),

    #[error("stored pretest outcome is not valid: {0}")]
    InvalidPretestOutcome(String),

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

    #[error("explain cards require at least one key point")]
    MissingExplainKeyPoint,

    #[error("explain cards cannot have more than {maximum} key points")]
    TooManyExplainKeyPoints { maximum: usize },

    #[error("explain card key points must be unique")]
    DuplicateExplainKeyPoint,

    #[error("problem cards require at least one checkpoint")]
    MissingProblemCheckpoint,

    #[error("problem cards require a problem in the Prompt")]
    MissingProblemPrompt,

    #[error("problem cards cannot have more than {maximum} checkpoints")]
    TooManyProblemCheckpoints { maximum: usize },

    #[error("problem card checkpoints must be unique")]
    DuplicateProblemCheckpoint,

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
