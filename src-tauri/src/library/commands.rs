use serde::Serialize;
use tauri::State;

use crate::data::LocalDataStore;
use crate::library::{
    ConceptDetail, ConceptLibrary, CreateConceptInput, CreateNamedItemInput, EntityIdInput,
    LibraryError, LibrarySnapshot, OrganizationSummary, RenameNamedItemInput,
    SetConceptArchivedInput, UpdateConceptInput,
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
            LibraryError::EmptyValue { .. } | LibraryError::ValueTooLong { .. } => "validation",
            LibraryError::DuplicateName { .. } => "conflict",
            LibraryError::ConceptNotFound(_)
            | LibraryError::OrganizationNotFound { .. }
            | LibraryError::InvalidSelection { .. } => "notFound",
            LibraryError::Data(_) | LibraryError::Database(_) => "storage",
        };
        let message = match &error {
            LibraryError::Data(_) | LibraryError::Database(_) => {
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
