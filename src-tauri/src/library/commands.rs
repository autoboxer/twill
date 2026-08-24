use serde::Serialize;
use tauri::ipc::{InvokeBody, Request, Response};
use tauri::State;

use crate::data::LocalDataStore;
use crate::library::{
    AuthoringDraft, AuthoringDraftLibrary, AuthoringDraftLocator, ConceptDetail,
    ConceptLibrary, CreateConceptInput, CreateCssSnippetInput, CreateNamedItemInput,
    CreateTemplateInput, CssSnippet, CssSnippetCatalog, CssSnippetLibrary,
    DevicePreferences, EntityIdInput, LibraryError, LibrarySnapshot,
    OrganizationSummary, RecordReviewInput, RenameNamedItemInput, ReverseReviewInput,
    ReviewOutcome, ReviewReversalOutcome, SchedulingSettings,
    SetAppearancePreferencesInput, SetConceptArchivedInput, SetCssSnippetEnabledInput,
    SetGradingModeInput, SetStartupDestinationInput, StudyQueue, TemplateCatalog,
    TemplateContent, TemplateDetail, TemplateLibrary, UpdateConceptInput,
    UpdateCssSnippetInput, UpdateSchedulingSettingsInput, UpdateTemplateInput,
    UpsertAuthoringDraftInput,
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
            | LibraryError::InvalidTemplate { .. }
            | LibraryError::InvalidCss { .. }
            | LibraryError::InvalidAuthoringDraft { .. }
            | LibraryError::ImageTooLarge { .. }
            | LibraryError::UnsupportedImage
            | LibraryError::ImageDimensionsTooLarge
            | LibraryError::InvalidDesiredRetention { .. }
            | LibraryError::InvalidMaximumInterval { .. }
            | LibraryError::MissingRetrievalForm
            | LibraryError::MissingAcceptedAnswer
            | LibraryError::TooManyAcceptedAnswers { .. }
            | LibraryError::DuplicateAcceptedAnswer => "validation",
            LibraryError::DuplicateName { .. }
            | LibraryError::CardNotDue { .. }
            | LibraryError::ReviewNotReversible
            | LibraryError::TemplateInUse { .. } => "conflict",
            LibraryError::ConceptNotFound(_)
            | LibraryError::OrganizationNotFound { .. }
            | LibraryError::TemplateNotFound(_)
            | LibraryError::CssSnippetNotFound(_)
            | LibraryError::InvalidSelection { .. }
            | LibraryError::MediaNotFound(_)
            | LibraryError::CardNotFound(_)
            | LibraryError::ReviewNotFound(_) => "notFound",
            LibraryError::Data(_)
            | LibraryError::Database(_)
            | LibraryError::Json(_)
            | LibraryError::MediaIntegrity { .. }
            | LibraryError::UnsupportedSchedulerConfiguration(_)
            | LibraryError::InvalidSchedulingState(_)
            | LibraryError::InvalidRetrievalFormKind(_)
            | LibraryError::InvalidRetrievalForm
            | LibraryError::InvalidGradingMode(_)
            | LibraryError::InvalidStartupDestination(_)
            | LibraryError::InvalidDevicePreference { .. }
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
            | LibraryError::InvalidRetrievalFormKind(_)
            | LibraryError::InvalidRetrievalForm
            | LibraryError::InvalidGradingMode(_)
            | LibraryError::InvalidStartupDestination(_)
            | LibraryError::InvalidDevicePreference { .. }
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
pub(crate) fn reverse_review(
    local_data: State<'_, LocalDataStore>,
    input: ReverseReviewInput,
) -> CommandResult<ReviewReversalOutcome> {
    ConceptLibrary::new(local_data.inner())
        .reverse_review(input)
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn get_device_preferences(
    local_data: State<'_, LocalDataStore>,
) -> CommandResult<DevicePreferences> {
    ConceptLibrary::new(local_data.inner())
        .device_preferences()
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn set_grading_mode(
    local_data: State<'_, LocalDataStore>,
    input: SetGradingModeInput,
) -> CommandResult<DevicePreferences> {
    ConceptLibrary::new(local_data.inner())
        .set_grading_mode(input.grading_mode)
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn set_startup_destination(
    local_data: State<'_, LocalDataStore>,
    input: SetStartupDestinationInput,
) -> CommandResult<DevicePreferences> {
    ConceptLibrary::new(local_data.inner())
        .set_startup_destination(input.startup_destination)
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn set_appearance_preferences(
    local_data: State<'_, LocalDataStore>,
    input: SetAppearancePreferencesInput,
) -> CommandResult<DevicePreferences> {
    ConceptLibrary::new(local_data.inner())
        .set_appearance_preferences(input.appearance)
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
pub(crate) fn get_css_snippets(
    local_data: State<'_, LocalDataStore>,
) -> CommandResult<CssSnippetCatalog> {
    CssSnippetLibrary::new(local_data.inner())
        .catalog()
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn create_css_snippet(
    local_data: State<'_, LocalDataStore>,
    input: CreateCssSnippetInput,
) -> CommandResult<CssSnippet> {
    CssSnippetLibrary::new(local_data.inner())
        .create_snippet(input)
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn update_css_snippet(
    local_data: State<'_, LocalDataStore>,
    input: UpdateCssSnippetInput,
) -> CommandResult<CssSnippet> {
    CssSnippetLibrary::new(local_data.inner())
        .update_snippet(input)
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn set_css_snippet_enabled(
    local_data: State<'_, LocalDataStore>,
    input: SetCssSnippetEnabledInput,
) -> CommandResult<CssSnippet> {
    CssSnippetLibrary::new(local_data.inner())
        .set_enabled(&input.id, input.enabled)
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn disable_all_css_snippets(
    local_data: State<'_, LocalDataStore>,
) -> CommandResult<()> {
    CssSnippetLibrary::new(local_data.inner())
        .disable_all()
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn delete_css_snippet(
    local_data: State<'_, LocalDataStore>,
    input: EntityIdInput,
) -> CommandResult<()> {
    CssSnippetLibrary::new(local_data.inner())
        .delete_snippet(&input.id)
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn get_authoring_draft(
    local_data: State<'_, LocalDataStore>,
    input: AuthoringDraftLocator,
) -> CommandResult<Option<AuthoringDraft>> {
    AuthoringDraftLibrary::new(local_data.inner())
        .draft(input)
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn upsert_authoring_draft(
    local_data: State<'_, LocalDataStore>,
    input: UpsertAuthoringDraftInput,
) -> CommandResult<AuthoringDraft> {
    AuthoringDraftLibrary::new(local_data.inner())
        .upsert_draft(input)
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn delete_authoring_draft(
    local_data: State<'_, LocalDataStore>,
    input: AuthoringDraftLocator,
) -> CommandResult<()> {
    AuthoringDraftLibrary::new(local_data.inner())
        .delete_draft(input)
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn get_templates(
    local_data: State<'_, LocalDataStore>,
) -> CommandResult<TemplateCatalog> {
    TemplateLibrary::new(local_data.inner())
        .catalog()
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn get_template(
    local_data: State<'_, LocalDataStore>,
    template_id: String,
) -> CommandResult<TemplateDetail> {
    TemplateLibrary::new(local_data.inner())
        .template(&template_id)
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn create_template(
    local_data: State<'_, LocalDataStore>,
    input: CreateTemplateInput,
) -> CommandResult<TemplateDetail> {
    TemplateLibrary::new(local_data.inner())
        .create_template(input)
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn update_template(
    local_data: State<'_, LocalDataStore>,
    input: UpdateTemplateInput,
) -> CommandResult<TemplateDetail> {
    TemplateLibrary::new(local_data.inner())
        .update_template(input)
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn delete_template(
    local_data: State<'_, LocalDataStore>,
    input: EntityIdInput,
) -> CommandResult<()> {
    TemplateLibrary::new(local_data.inner())
        .delete_template(&input.id)
        .map_err(Into::into)
}

#[tauri::command(async)]
pub(crate) fn prepare_template_preview(
    content: TemplateContent,
) -> CommandResult<TemplateContent> {
    TemplateLibrary::prepare_content(content).map_err(Into::into)
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
