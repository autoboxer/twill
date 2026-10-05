mod archive;
mod checksum;
pub(crate) mod commands;
mod error;
mod models;
mod portable;
mod restore;
mod validation;
mod zip_directory;

pub(crate) mod recovery;

pub use archive::{create_backup, export_library};
pub use error::{BackupError, BackupResult};
pub use models::{ArchiveKind, ArchiveSummary, BackupPreview, LibraryCounts};
pub use validation::inspect_backup;

#[cfg(test)]
mod tests;
