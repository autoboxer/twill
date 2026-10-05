use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::data::LocalDataStore;

use super::{ArchiveKind, ArchiveSummary, BackupError};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CommandError {
    code: &'static str,
    message: String,
}

impl From<BackupError> for CommandError {
    fn from(error: BackupError) -> Self {
        let (code, message) = match error {
            BackupError::InvalidDestination => (
                "validation",
                "Choose a file outside Twill's data directory".to_owned(),
            ),
            BackupError::DestinationExists => (
                "conflict",
                "That file already exists. Choose a new filename".to_owned(),
            ),
            BackupError::Integrity(_)
            | BackupError::Json(_)
            | BackupError::Database(_)
            | BackupError::Data(_) => (
                "storage",
                "Local data could not be exported. Your library has not been changed".to_owned(),
            ),
            BackupError::Io(_) | BackupError::Zip(_) => (
                "file",
                "The archive could not be saved. Check the folder and available space".to_owned(),
            ),
        };

        Self { code, message }
    }
}

#[tauri::command]
pub(crate) async fn create_backup(
    app: AppHandle,
    destination: String,
) -> Result<ArchiveSummary, CommandError> {
    write_archive(app, destination, ArchiveKind::Backup).await
}

#[tauri::command]
pub(crate) async fn export_library(
    app: AppHandle,
    destination: String,
) -> Result<ArchiveSummary, CommandError> {
    write_archive(app, destination, ArchiveKind::Export).await
}

async fn write_archive(
    app: AppHandle,
    destination: String,
    kind: ArchiveKind,
) -> Result<ArchiveSummary, CommandError> {
    tauri::async_runtime::spawn_blocking(move || {
        let store = app
            .try_state::<LocalDataStore>()
            .ok_or_else(|| CommandError {
                code: "storage",
                message: "Open or restore your library before creating an archive".to_owned(),
            })?;
        let result = match kind {
            ArchiveKind::Backup => super::create_backup(&store, destination),
            ArchiveKind::Export => super::export_library(&store, destination),
        };

        result.map_err(CommandError::from)
    })
    .await
    .map_err(|_| CommandError {
        code: "storage",
        message: "The archive operation could not finish".to_owned(),
    })?
}
