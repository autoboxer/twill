use std::collections::VecDeque;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use serde::Serialize;

use crate::data::{DataError, LocalDataStore};
use crate::library::{AuthoringMediaLibrary, LibraryError};

use super::{restore, BackupError, BackupPreview, BackupResult};

const MAX_DIAGNOSTICS: usize = 12;

pub(crate) struct StorageRecovery(Mutex<RecoveryState>);

struct RecoveryState {
    profile: PathBuf,
    lock: Option<Arc<File>>,
    ready: bool,
    reason: Option<&'static str>,
    diagnostics: VecDeque<StorageDiagnostic>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StorageStatus {
    pub ready: bool,
    pub reason: Option<&'static str>,
    pub restore_pending: bool,
    pub diagnostics: Vec<StorageDiagnostic>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StorageDiagnostic {
    operation: &'static str,
    category: &'static str,
    code: Option<i32>,
}

impl StorageRecovery {
    pub(crate) fn new(profile: PathBuf) -> Self {
        Self(Mutex::new(RecoveryState {
            profile,
            lock: None,
            ready: false,
            reason: None,
            diagnostics: VecDeque::new(),
        }))
    }

    pub(crate) fn open(&self, publish: impl FnOnce(LocalDataStore)) -> BackupResult<()> {
        let mut state = self.state()?;

        if state.ready {
            return Ok(());
        }

        let result = (|| {
            if state.lock.is_none() {
                state.lock = Some(LocalDataStore::lock_profile(&state.profile)?);
            }

            let lock = state.lock.as_ref().unwrap().clone();

            restore::open_with_restore(&state.profile, || {
                let store = LocalDataStore::open_locked(&state.profile, lock)?;

                AuthoringMediaLibrary::new(&store)
                    .release_abandoned_sessions()
                    .map_err(|error| match error {
                        LibraryError::Data(error) => BackupError::Data(error),
                        LibraryError::Database(error) => BackupError::Database(error),
                        _ => BackupError::Integrity("authoring media recovery failed"),
                    })?;

                Ok(store)
            })
        })();

        match result {
            Ok(store) => {
                publish(store);
                state.ready = true;
                state.reason = None;

                Ok(())
            }
            Err(error) => {
                state.reason = Some(recovery_reason(&error));
                state.record("startup", &error);

                Err(error)
            }
        }
    }

    pub(crate) fn status(&self) -> BackupResult<StorageStatus> {
        let state = self.state()?;

        Ok(StorageStatus {
            ready: state.ready,
            reason: state.reason,
            restore_pending: restore::restore_pending(&state.profile).unwrap_or(false),
            diagnostics: state.diagnostics.iter().cloned().collect(),
        })
    }

    pub(crate) fn prepare(&self, source: &Path, fingerprint: &str) -> BackupResult<BackupPreview> {
        let mut state = self.state()?;

        if state.lock.is_none() {
            state.lock = Some(LocalDataStore::lock_profile(&state.profile)?);
        }

        let result = restore::prepare_restore(&state.profile, source, fingerprint);

        if let Err(error) = &result {
            state.record("restore", error);
        }

        result
    }

    pub(crate) fn cancel(&self) -> BackupResult<()> {
        let mut state = self.state()?;

        if state.lock.is_none() {
            return Err(BackupError::Integrity("the profile is not locked"));
        }

        let result = restore::cancel_restore(&state.profile);

        if let Err(error) = &result {
            state.record("cancelRestore", error);
        }

        result
    }

    fn state(&self) -> BackupResult<MutexGuard<'_, RecoveryState>> {
        self.0
            .lock()
            .map_err(|_| DataError::ConnectionUnavailable.into())
    }
}

impl RecoveryState {
    fn record(&mut self, operation: &'static str, error: &BackupError) {
        let (category, code) = match error {
            BackupError::Database(rusqlite::Error::SqliteFailure(error, _))
            | BackupError::Data(DataError::Database(rusqlite::Error::SqliteFailure(error, _))) => {
                ("sqlite", Some(error.extended_code))
            }
            BackupError::Io(error) | BackupError::Data(DataError::Io(error)) => {
                ("filesystem", error.raw_os_error())
            }
            BackupError::Data(DataError::Schema(_)) | BackupError::IncompatibleArchive => {
                ("schema", None)
            }
            BackupError::InvalidArchive(_) => ("archive", None),
            BackupError::RestoreInterrupted => ("interruptedRestore", None),
            BackupError::Zip(_) => ("zip", None),
            _ => ("storage", None),
        };

        if self.diagnostics.len() == MAX_DIAGNOSTICS {
            self.diagnostics.pop_front();
        }

        self.diagnostics.push_back(StorageDiagnostic {
            operation,
            category,
            code,
        });
    }
}

fn recovery_reason(error: &BackupError) -> &'static str {
    match error {
        BackupError::RestoreInterrupted => "An interrupted restore was rolled back. Retry opening your library",
        BackupError::IncompatibleArchive => "The backup does not match this version of Twill",
        BackupError::InvalidArchive(_) => "The pending backup could not be verified",
        _ => "Your library could not be opened. Close other Twill windows and retry, or restore a backup",
    }
}

#[cfg(test)]
mod tests;
