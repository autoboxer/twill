use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::data::LocalDataStore;

use super::recovery::{StorageRecovery, StorageStatus};
use super::BackupPreview;
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
            BackupError::InvalidArchive(_) => {
                ("validation", "This is not a valid Twill backup".to_owned())
            }
            BackupError::ArchiveLimit => (
                "validation",
                "This library exceeds the supported archive size".to_owned(),
            ),
            BackupError::IncompatibleArchive => (
                "compatibility",
                "This backup does not match this version of Twill".to_owned(),
            ),
            BackupError::RestorePending => (
                "conflict",
                "A restore is already pending. Cancel it before choosing another backup".to_owned(),
            ),
            BackupError::RestoreInterrupted => (
                "storage",
                "The interrupted restore was rolled back. Retry opening your library".to_owned(),
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
pub(crate) async fn inspect_backup(source: String) -> Result<BackupPreview, CommandError> {
    tauri::async_runtime::spawn_blocking(move || {
        super::inspect_backup(source).map_err(restore_error)
    })
    .await
    .map_err(worker_error)?
}

#[tauri::command]
pub(crate) async fn prepare_restore(
    app: AppHandle,
    source: String,
    fingerprint: String,
) -> Result<BackupPreview, CommandError> {
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<StorageRecovery>()
            .prepare(std::path::Path::new(&source), &fingerprint)
            .map_err(restore_error)
    })
    .await
    .map_err(worker_error)?
}

#[tauri::command]
pub(crate) async fn cancel_restore(app: AppHandle) -> Result<(), CommandError> {
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<StorageRecovery>()
            .cancel()
            .map_err(restore_error)
    })
    .await
    .map_err(worker_error)?
}

#[tauri::command]
pub(crate) async fn get_storage_status(app: AppHandle) -> Result<StorageStatus, CommandError> {
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<StorageRecovery>()
            .status()
            .map_err(restore_error)
    })
    .await
    .map_err(worker_error)?
}

#[tauri::command]
pub(crate) async fn retry_storage(app: AppHandle) -> Result<StorageStatus, CommandError> {
    tauri::async_runtime::spawn_blocking(move || {
        let recovery = app.state::<StorageRecovery>();

        let _ = recovery.open(|store| {
            app.manage(store);
        });

        recovery.status().map_err(restore_error)
    })
    .await
    .map_err(worker_error)?
}

fn restore_error(error: BackupError) -> CommandError {
    match error {
        BackupError::Io(_) => CommandError {
            code: "file",
            message: "The backup could not be read or prepared. Check access and available space"
                .to_owned(),
        },
        BackupError::Zip(_) | BackupError::Json(_) | BackupError::Database(_) => CommandError {
            code: "validation",
            message: "This is not a valid Twill backup".to_owned(),
        },
        BackupError::Integrity(_) | BackupError::Data(_) => CommandError {
            code: "storage",
            message: "The restore could not finish. Your original library has been retained"
                .to_owned(),
        },
        other => other.into(),
    }
}

fn worker_error(_: tauri::Error) -> CommandError {
    CommandError {
        code: "storage",
        message: "The backup operation could not finish".to_owned(),
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
