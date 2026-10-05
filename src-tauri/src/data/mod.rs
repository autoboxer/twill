mod entity;
mod error;
mod schema;
mod store;

pub use entity::{ChangeOperation, ChangeRecord, EntityKind, EntityMetadata};
pub use error::{DataError, DataResult};
pub use store::{
    LocalDataStore, WriteTransaction, DATABASE_FILENAME, MEDIA_DIRECTORY_NAME,
};
pub(crate) use store::current_timestamp;

pub(crate) fn current_schema_connection() -> DataResult<rusqlite::Connection> {
    let mut connection = rusqlite::Connection::open_in_memory()?;

    schema::ensure_current(&mut connection)?;

    Ok(connection)
}
