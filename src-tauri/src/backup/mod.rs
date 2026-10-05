mod archive;
mod checksum;
pub(crate) mod commands;
mod error;
mod models;
mod portable;

pub use archive::{create_backup, export_library};
pub use error::{BackupError, BackupResult};
pub use models::{ArchiveKind, ArchiveSummary, LibraryCounts};

#[cfg(test)]
mod tests;
