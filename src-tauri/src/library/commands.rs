use serde::Serialize;
use tauri::ipc::{InvokeBody, Request, Response};
use tauri::State;

use crate::data::LocalDataStore;
use crate::library::{
    ConceptDetail, ConceptLibrary, CreateConceptInput, CreateNamedItemInput, EntityIdInput,
    LibraryError, LibrarySnapshot, OrganizationSummary, RenameNamedItemInput,
    RecordReviewInput, ReviewOutcome, SchedulingSettings, SetConceptArchivedInput,
    SetGradingModeInput, StudyPreferences, StudyQueue, UpdateConceptInput,
    UpdateSchedulingSettingsInput,
};

type CommandResult<T> = Result<T, CommandError>;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CommandError {
    code: &'static str,
    message: String,
}

impl From<LibraryError> for CommandError {
    fn from(error: LibraryError) -> Self {
        let code = match &error {
            LibraryError::EmptyValue { .. }
            | LibraryError::ValueTooLong { .. }
            | LibraryError::InvalidContent { .. }
            | LibraryError::ImageTooLarge { .. }
            | LibraryError::UnsupportedImage
            | LibraryError::ImageDimensionsTooLarge
            | LibraryError::InvalidDesiredRetention { .. }
            | LibraryError::InvalidMaximumInterval { .. } => "validation",
            LibraryError::DuplicateName { .. } | LibraryError::CardNotDue { .. } => "conflict",
            LibraryError::ConceptNotFound(_)
            | LibraryError::OrganizationNotFound { .. }
            | LibraryError::InvalidSelection { .. }
            | LibraryError::MediaNotFound(_)
            | LibraryError::CardNotFound(_) => "notFound",
            LibraryError::Data(_)
            | LibraryError::Database(_)
            | LibraryError::Json(_)
            | LibraryError::MediaIntegrity { .. }
            | LibraryError::UnsupportedSchedulerConfiguration(_)
            | LibraryError::InvalidSchedulingState(_)
            | LibraryError::InvalidGradingMode(_)
            | LibraryError::InvalidSchedule
            | LibraryError::Scheduler(_) => "storage",
        };
        let message = match &error {
            LibraryError::Data(_)
            | LibraryError::Database(_)
            | LibraryError::Json(_)
            | LibraryError::MediaIntegrity { .. }
            | LibraryError::UnsupportedSchedulerConfiguration(_)
            | LibraryError::InvalidSchedulingState(_)
            | LibraryError::InvalidGradingMode(_)
            | LibraryError::InvalidSchedule
            | LibraryError::Scheduler(_) => {
                "Local data could not be accessed.".to_owned()
            }
            _ => error.to_string(),
        };

        Self { code, message }
    }
}

#[tauri::command(async)]
pub(crate) fn get_library(
    local_data: State<'_, LocalDataStore>,
    include_archived: bool,
) -> CommandResult<LibrarySnapshot> {
    ConceptLibrary::new(local_data.inner())
        .snapshot(include_archived)
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn get_concept(
    local_data: State<'_, LocalDataStore>,
    concept_id: String,
) -> CommandResult<ConceptDetail> {
    ConceptLibrary::new(local_data.inner())
        .concept(&concept_id)
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn get_study_queue(
    local_data: State<'_, LocalDataStore>,
) -> CommandResult<StudyQueue> {
    ConceptLibrary::new(local_data.inner())
        .study_queue()
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn record_review(
    local_data: State<'_, LocalDataStore>,
    input: RecordReviewInput,
) -> CommandResult<ReviewOutcome> {
    ConceptLibrary::new(local_data.inner())
        .record_review(input)
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn get_study_preferences(
    local_data: State<'_, LocalDataStore>,
) -> CommandResult<StudyPreferences> {
    ConceptLibrary::new(local_data.inner())
        .study_preferences()
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn set_grading_mode(
    local_data: State<'_, LocalDataStore>,
    input: SetGradingModeInput,
) -> CommandResult<StudyPreferences> {
    ConceptLibrary::new(local_data.inner())
        .set_grading_mode(input.grading_mode)
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn get_scheduling_settings(
    local_data: State<'_, LocalDataStore>,
) -> CommandResult<SchedulingSettings> {
    ConceptLibrary::new(local_data.inner())
        .scheduling_settings()
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn update_scheduling_settings(
    local_data: State<'_, LocalDataStore>,
    input: UpdateSchedulingSettingsInput,
) -> CommandResult<SchedulingSettings> {
    ConceptLibrary::new(local_data.inner())
        .update_scheduling_settings(input)
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn create_concept(
    local_data: State<'_, LocalDataStore>,
    input: CreateConceptInput,
) -> CommandResult<ConceptDetail> {
    ConceptLibrary::new(local_data.inner())
        .create_concept(input)
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn update_concept(
    local_data: State<'_, LocalDataStore>,
    input: UpdateConceptInput,
) -> CommandResult<ConceptDetail> {
    ConceptLibrary::new(local_data.inner())
        .update_concept(input)
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn set_concept_archived(
    local_data: State<'_, LocalDataStore>,
    input: SetConceptArchivedInput,
) -> CommandResult<ConceptDetail> {
    ConceptLibrary::new(local_data.inner())
        .set_concept_archived(&input.id, input.archived)
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn delete_concept(
    local_data: State<'_, LocalDataStore>,
    input: EntityIdInput,
) -> CommandResult<()> {
    ConceptLibrary::new(local_data.inner())
        .delete_concept(&input.id)
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn create_deck(
    local_data: State<'_, LocalDataStore>,
    input: CreateNamedItemInput,
) -> CommandResult<OrganizationSummary> {
    ConceptLibrary::new(local_data.inner())
        .create_deck(input.name)
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn rename_deck(
    local_data: State<'_, LocalDataStore>,
    input: RenameNamedItemInput,
) -> CommandResult<OrganizationSummary> {
    ConceptLibrary::new(local_data.inner())
        .rename_deck(&input.id, input.name)
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn delete_deck(
    local_data: State<'_, LocalDataStore>,
    input: EntityIdInput,
) -> CommandResult<()> {
    ConceptLibrary::new(local_data.inner())
        .delete_deck(&input.id)
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn create_tag(
    local_data: State<'_, LocalDataStore>,
    input: CreateNamedItemInput,
) -> CommandResult<OrganizationSummary> {
    ConceptLibrary::new(local_data.inner())
        .create_tag(input.name)
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn rename_tag(
    local_data: State<'_, LocalDataStore>,
    input: RenameNamedItemInput,
) -> CommandResult<OrganizationSummary> {
    ConceptLibrary::new(local_data.inner())
        .rename_tag(&input.id, input.name)
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn delete_tag(
    local_data: State<'_, LocalDataStore>,
    input: EntityIdInput,
) -> CommandResult<()> {
    ConceptLibrary::new(local_data.inner())
        .delete_tag(&input.id)
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn import_image(
    local_data: State<'_, LocalDataStore>,
    request: Request<'_>,
) -> CommandResult<crate::library::MediaSummary> {
    let owned_bytes;
    let bytes = match request.body() {
        InvokeBody::Raw(bytes) => bytes.as_slice(),
        InvokeBody::Json(serde_json::Value::Array(values)) => {
            owned_bytes = values
                .iter()
                .map(|value| {
                    value
                        .as_u64()
                        .and_then(|byte| u8::try_from(byte).ok())
                        .ok_or(LibraryError::UnsupportedImage)
                })
                .collect::<Result<Vec<_>, _>>()?;
            &owned_bytes
        }
        InvokeBody::Json(_) => return Err(LibraryError::UnsupportedImage.into()),
    };

    ConceptLibrary::new(local_data.inner())
        .import_image(bytes)
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn read_media(
    local_data: State<'_, LocalDataStore>,
    media_id: String,
) -> CommandResult<Response> {
    ConceptLibrary::new(local_data.inner())
        .media_bytes(&media_id)
        .map(Response::new)
        .map_err(Into::into)
}
