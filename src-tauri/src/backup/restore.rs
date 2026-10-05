use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tempfile::Builder;

use super::validation::{extract_backup, validate_staged};
use super::{BackupError, BackupPreview, BackupResult};

const PENDING_DIRECTORY: &str = ".twill-restore";
const JOURNAL_FILENAME: &str = "journal.json";
const PROFILE_ITEMS: [&str; 4] = [
    "twill.sqlite3",
    "twill.sqlite3-wal",
    "twill.sqlite3-shm",
    "media",
];

#[derive(Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
enum Phase {
    Prepared,
    Installing,
    Committed,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Journal {
    version: u32,
    phase: Phase,
    originals: Vec<String>,
}

pub(super) fn prepare_restore(
    profile: &Path,
    source: &Path,
    expected_fingerprint: &str,
) -> BackupResult<BackupPreview> {
    let pending = profile.join(PENDING_DIRECTORY);

    if exists(&pending)? {
        return Err(BackupError::RestorePending);
    }

    let staging = Builder::new()
        .prefix(".twill-restore-")
        .tempdir_in(profile)?;
    let new = staging.path().join("new");

    fs::create_dir(&new)?;
    fs::create_dir(staging.path().join("old"))?;
    fs::create_dir(staging.path().join("discarded"))?;

    let preview = extract_backup(source, &new)?;

    if preview.fingerprint != expected_fingerprint {
        return Err(BackupError::InvalidArchive(
            "the backup changed after its preview",
        ));
    }

    sync_directory(&new.join("media"))?;
    sync_directory(&new)?;
    write_journal(
        staging.path(),
        &Journal {
            version: 1,
            phase: Phase::Prepared,
            originals: Vec::new(),
        },
    )?;
    fs::rename(staging.path(), &pending)?;
    sync_directory(profile)?;

    // The startup journal owns this directory after publication
    let _ = staging.keep();

    Ok(preview)
}

pub(super) fn restore_pending(profile: &Path) -> BackupResult<bool> {
    exists(&profile.join(PENDING_DIRECTORY))
}

pub(super) fn cancel_restore(profile: &Path) -> BackupResult<()> {
    let pending = profile.join(PENDING_DIRECTORY);

    if !exists(&pending)? {
        return Ok(());
    }

    let journal = read_journal(&pending)?;

    if journal.phase == Phase::Installing {
        rollback(profile, &pending, &journal)?;
    }

    remove_staging(&pending)?;
    sync_directory(profile)
}

pub(super) fn open_with_restore<T>(
    profile: &Path,
    open: impl FnOnce() -> BackupResult<T>,
) -> BackupResult<T> {
    let pending = profile.join(PENDING_DIRECTORY);

    if !exists(&pending)? {
        return open();
    }

    let mut journal = read_journal(&pending)?;

    match journal.phase {
        Phase::Installing => {
            rollback(profile, &pending, &journal)?;
            remove_staging(&pending)?;
            sync_directory(profile)?;

            return Err(BackupError::RestoreInterrupted);
        }
        Phase::Committed => {
            // A completed restore remains usable if deferred cleanup cannot finish
            let _ = remove_staging(&pending).and_then(|_| sync_directory(profile));

            return open();
        }
        Phase::Prepared => {}
    }

    validate_staged(&pending.join("new"))?;

    for name in PROFILE_ITEMS {
        let path = profile.join(name);

        if exists(&path)? {
            check_item(&path, name == "media")?;
            journal.originals.push(name.to_owned());
        }
    }

    journal.phase = Phase::Installing;
    write_journal(&pending, &journal)?;

    let result = (|| {
        for name in &journal.originals {
            move_item(&profile.join(name), &pending.join("old").join(name))?;
        }

        for name in ["twill.sqlite3", "media"] {
            move_item(&pending.join("new").join(name), &profile.join(name))?;
        }

        open()
    })();

    let opened = match result {
        Ok(opened) => opened,
        Err(error) => {
            rollback(profile, &pending, &journal)?;
            remove_staging(&pending)?;
            sync_directory(profile)?;

            return Err(error);
        }
    };

    journal.phase = Phase::Committed;

    // On an uncertain journal write, leave both generations for startup recovery
    write_journal(&pending, &journal)?;

    let _ = remove_staging(&pending).and_then(|_| sync_directory(profile));

    Ok(opened)
}

fn rollback(profile: &Path, pending: &Path, journal: &Journal) -> BackupResult<()> {
    for name in PROFILE_ITEMS {
        let original = pending.join("old").join(name);
        let current = profile.join(name);

        if exists(&original)? {
            if exists(&current)? {
                move_item(&current, &pending.join("discarded").join(name))?;
            }

            move_item(&original, &current)?;
        } else if !journal.originals.iter().any(|item| item == name) && exists(&current)? {
            move_item(&current, &pending.join("discarded").join(name))?;
        }
    }

    Ok(())
}

fn read_journal(pending: &Path) -> BackupResult<Journal> {
    check_item(pending, true)?;

    for directory in ["old", "new", "discarded"] {
        check_item(&pending.join(directory), true)?;
    }

    let path = pending.join(JOURNAL_FILENAME);

    check_item(&path, false)?;

    if fs::metadata(&path)?.len() > 4096 {
        return Err(BackupError::Integrity("the restore journal is too large"));
    }

    let journal: Journal = serde_json::from_reader(File::open(path)?)?;
    let unique = journal
        .originals
        .iter()
        .collect::<std::collections::BTreeSet<_>>();

    if journal.version != 1
        || unique.len() != journal.originals.len()
        || journal
            .originals
            .iter()
            .any(|name| !PROFILE_ITEMS.contains(&name.as_str()))
        || (journal.phase == Phase::Prepared && !journal.originals.is_empty())
    {
        return Err(BackupError::Integrity("the restore journal is invalid"));
    }

    Ok(journal)
}

fn write_journal(pending: &Path, journal: &Journal) -> BackupResult<()> {
    let mut temporary = Builder::new().prefix(".journal-").tempfile_in(pending)?;

    serde_json::to_writer(temporary.as_file_mut(), journal)?;
    temporary.as_file().sync_all()?;
    temporary
        .persist(pending.join(JOURNAL_FILENAME))
        .map_err(|error| error.error)?;

    sync_directory(pending)
}

fn move_item(source: &Path, destination: &Path) -> BackupResult<()> {
    check_item(
        source,
        source.file_name().is_some_and(|name| name == "media"),
    )?;

    if exists(destination)? {
        return Err(BackupError::Integrity("a restore destination is occupied"));
    }

    fs::rename(source, destination)?;
    sync_directory(source.parent().ok_or(BackupError::InvalidDestination)?)?;
    sync_directory(
        destination
            .parent()
            .ok_or(BackupError::InvalidDestination)?,
    )
}

fn check_item(path: &Path, directory: bool) -> BackupResult<()> {
    let metadata = fs::symlink_metadata(path)?;

    if (directory && !metadata.is_dir()) || (!directory && !metadata.is_file()) {
        return Err(BackupError::Integrity(
            "a restore path has an unexpected type",
        ));
    }

    Ok(())
}

fn exists(path: &Path) -> BackupResult<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

fn remove_staging(path: &Path) -> BackupResult<()> {
    let parent = path.parent().ok_or(BackupError::InvalidDestination)?;
    let discarded = parent.join(format!(".twill-restore-cleanup-{}", uuid::Uuid::now_v7()));

    if exists(&discarded)? {
        return Err(BackupError::Integrity(
            "the restore cleanup destination is occupied",
        ));
    }

    // Retire the journal atomically before cleanup can be interrupted
    fs::rename(path, &discarded)?;
    sync_directory(parent)?;

    remove_tree(&discarded)
}

fn remove_tree(path: &Path) -> BackupResult<()> {
    // Inspect the exact owned tree before recursive cleanup; never follow links
    let mut directories = vec![path.to_path_buf()];
    let mut files = Vec::<PathBuf>::new();
    let mut index = 0;

    while index < directories.len() {
        check_item(&directories[index], true)?;

        for entry in fs::read_dir(&directories[index])? {
            let entry = entry?;
            let kind = entry.file_type()?;

            if kind.is_dir() {
                directories.push(entry.path());
            } else if kind.is_file() {
                files.push(entry.path());
            } else {
                return Err(BackupError::Integrity(
                    "a restore cleanup path has an unexpected type",
                ));
            }
        }

        index += 1;
    }

    for file in files {
        fs::remove_file(file)?;
    }

    for directory in directories.into_iter().rev() {
        fs::remove_dir(directory)?;
    }

    Ok(())
}

fn sync_directory(path: &Path) -> BackupResult<()> {
    #[cfg(unix)]
    File::open(path)?.sync_all()?;

    #[cfg(not(unix))]
    let _ = path;

    Ok(())
}

#[cfg(test)]
mod tests;
