use std::fs;
use std::ops::Deref;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, OptionalExtension, Transaction, TransactionBehavior};
use uuid::Uuid;

use crate::data::migrations;
use crate::data::{
    ChangeOperation, ChangeRecord, DataError, DataResult, EntityKind, EntityMetadata,
};

pub const DATABASE_FILENAME: &str = "twill.sqlite3";

const WRITE_TIMEOUT: Duration = Duration::from_secs(5);

pub struct LocalDataStore {
    connection: Mutex<Connection>,
}

pub struct WriteTransaction<'connection> {
    transaction: Transaction<'connection>,
}

impl LocalDataStore {
    pub fn open(data_directory: impl AsRef<Path>) -> DataResult<Self> {
        fs::create_dir_all(data_directory.as_ref())?;

        let database_path = data_directory.as_ref().join(DATABASE_FILENAME);
        let connection = Connection::open(database_path)?;

        Self::from_connection(connection)
    }

    pub fn write<T>(
        &self,
        operation: impl FnOnce(&WriteTransaction<'_>) -> DataResult<T>,
    ) -> DataResult<T> {
        self.write_result(operation)
    }

    pub fn write_result<T, E>(
        &self,
        operation: impl FnOnce(&WriteTransaction<'_>) -> Result<T, E>,
    ) -> Result<T, E>
    where
        E: From<DataError>,
    {
        let mut connection = self.connection().map_err(E::from)?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(DataError::from)
            .map_err(E::from)?;
        let transaction = WriteTransaction { transaction };

        let result = operation(&transaction)?;

        transaction
            .transaction
            .commit()
            .map_err(DataError::from)
            .map_err(E::from)?;

        Ok(result)
    }

    pub fn read_result<T, E>(
        &self,
        operation: impl FnOnce(&Connection) -> Result<T, E>,
    ) -> Result<T, E>
    where
        E: From<DataError>,
    {
        let connection = self.connection().map_err(E::from)?;

        operation(&connection)
    }

    pub fn entity(&self, id: &str) -> DataResult<Option<EntityMetadata>> {
        let connection = self.connection()?;

        query_entity(&connection, id)
    }

    pub fn changes_after(&self, sequence: i64, limit: usize) -> DataResult<Vec<ChangeRecord>> {
        let connection = self.connection()?;
        let limit = i64::try_from(limit).unwrap_or(i64::MAX);
        let mut statement = connection.prepare(
            "SELECT
                change_log.sequence,
                change_log.id,
                change_log.entity_id,
                entities.kind,
                change_log.operation,
                change_log.recorded_at
            FROM change_log
            INNER JOIN entities ON entities.id = change_log.entity_id
            WHERE change_log.sequence > ?1
            ORDER BY change_log.sequence
            LIMIT ?2",
        )?;
        let rows = statement.query_map(params![sequence, limit], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
            ))
        })?;

        rows.map(|row| {
            let (sequence, id, entity_id, entity_kind, operation, recorded_at) = row?;

            Ok(ChangeRecord {
                sequence,
                id,
                entity_id,
                entity_kind: EntityKind::try_from(entity_kind.as_str())?,
                operation: ChangeOperation::try_from(operation.as_str())?,
                recorded_at,
            })
        })
        .collect()
    }

    pub fn schema_version(&self) -> DataResult<i64> {
        let connection = self.connection()?;
        let version = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;

        Ok(version)
    }

    fn from_connection(mut connection: Connection) -> DataResult<Self> {
        configure_connection(&connection)?;
        migrations::apply(&mut connection)?;

        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    fn connection(&self) -> DataResult<MutexGuard<'_, Connection>> {
        self.connection
            .lock()
            .map_err(|_| DataError::ConnectionUnavailable)
    }

    #[cfg(test)]
    fn open_in_memory() -> DataResult<Self> {
        Self::from_connection(Connection::open_in_memory()?)
    }
}

impl WriteTransaction<'_> {
    pub fn create_entity(&self, kind: EntityKind) -> DataResult<EntityMetadata> {
        let timestamp = current_timestamp()?;
        let entity_id = new_id();
        let change_id = record_change(
            self,
            &entity_id,
            ChangeOperation::Create,
            timestamp,
        )?;

        self.execute(
            "INSERT INTO entities (
                id,
                kind,
                created_at,
                updated_at,
                deleted_at,
                revision,
                last_change_id
            ) VALUES (?1, ?2, ?3, ?3, NULL, 1, ?4)",
            params![entity_id, kind.as_str(), timestamp, change_id],
        )?;

        query_entity(self, &entity_id)?.ok_or(DataError::EntityNotFound(entity_id))
    }

    pub fn touch_entity(&self, id: &str) -> DataResult<EntityMetadata> {
        let entity = required_entity(self, id)?;

        if entity.deleted_at.is_some() {
            return Err(DataError::EntityDeleted(id.to_owned()));
        }

        let timestamp = current_timestamp()?.max(entity.updated_at);
        let change_id = record_change(self, id, ChangeOperation::Update, timestamp)?;

        self.execute(
            "UPDATE entities
            SET updated_at = ?1,
                revision = revision + 1,
                last_change_id = ?2
            WHERE id = ?3",
            params![timestamp, change_id, id],
        )?;

        required_entity(self, id)
    }

    pub fn soft_delete_entity(&self, id: &str) -> DataResult<EntityMetadata> {
        let entity = required_entity(self, id)?;

        if entity.deleted_at.is_some() {
            return Ok(entity);
        }

        let timestamp = current_timestamp()?.max(entity.updated_at);
        let change_id = record_change(self, id, ChangeOperation::Delete, timestamp)?;

        self.execute(
            "UPDATE entities
            SET updated_at = ?1,
                deleted_at = ?1,
                revision = revision + 1,
                last_change_id = ?2
            WHERE id = ?3",
            params![timestamp, change_id, id],
        )?;

        required_entity(self, id)
    }
}

impl Deref for WriteTransaction<'_> {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.transaction
    }
}

fn configure_connection(connection: &Connection) -> DataResult<()> {
    connection.busy_timeout(WRITE_TIMEOUT)?;
    connection.pragma_update(None, "foreign_keys", true)?;
    connection.pragma_update(None, "journal_mode", "WAL")?;
    connection.pragma_update(None, "synchronous", "FULL")?;

    Ok(())
}

fn record_change(
    transaction: &WriteTransaction<'_>,
    entity_id: &str,
    operation: ChangeOperation,
    recorded_at: i64,
) -> DataResult<String> {
    let change_id = new_id();

    transaction.execute(
        "INSERT INTO change_log (id, entity_id, operation, recorded_at)
        VALUES (?1, ?2, ?3, ?4)",
        params![change_id, entity_id, operation.as_str(), recorded_at],
    )?;

    Ok(change_id)
}

fn required_entity(connection: &Connection, id: &str) -> DataResult<EntityMetadata> {
    query_entity(connection, id)?.ok_or_else(|| DataError::EntityNotFound(id.to_owned()))
}

fn query_entity(connection: &Connection, id: &str) -> DataResult<Option<EntityMetadata>> {
    let row = connection
        .query_row(
            "SELECT
                id,
                kind,
                created_at,
                updated_at,
                deleted_at,
                revision,
                last_change_id
            FROM entities
            WHERE id = ?1",
            [id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, Option<i64>>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, String>(6)?,
                ))
            },
        )
        .optional()?;

    row.map(
        |(id, kind, created_at, updated_at, deleted_at, revision, last_change_id)| {
            Ok(EntityMetadata {
                id,
                kind: EntityKind::try_from(kind.as_str())?,
                created_at,
                updated_at,
                deleted_at,
                revision,
                last_change_id,
            })
        },
    )
    .transpose()
}

fn current_timestamp() -> DataResult<i64> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| DataError::InvalidSystemTime)?
        .as_millis();

    i64::try_from(timestamp).map_err(|_| DataError::InvalidSystemTime)
}

fn new_id() -> String {
    Uuid::now_v7().hyphenated().to_string()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use tempfile::tempdir;
    use uuid::Version;

    use super::{LocalDataStore, DATABASE_FILENAME};
    use crate::data::{ChangeOperation, DataError, DataResult, EntityKind};

    #[test]
    fn opening_a_store_creates_and_reuses_the_latest_database() {
        let temporary_directory = tempdir().unwrap();
        let data_directory = temporary_directory.path().join("nested").join("data");

        let store = LocalDataStore::open(&data_directory).unwrap();

        assert_eq!(store.schema_version().unwrap(), 2);
        assert!(data_directory.join(DATABASE_FILENAME).is_file());

        let connection = store.connection().unwrap();
        let foreign_keys: i64 = connection
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        let journal_mode: String = connection
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        let synchronous: i64 = connection
            .query_row("PRAGMA synchronous", [], |row| row.get(0))
            .unwrap();

        assert_eq!(foreign_keys, 1);
        assert_eq!(journal_mode, "wal");
        assert_eq!(synchronous, 2);

        drop(connection);

        drop(store);

        let reopened_store = LocalDataStore::open(&data_directory).unwrap();

        assert_eq!(reopened_store.schema_version().unwrap(), 2);
    }

    #[test]
    fn every_syncable_entity_kind_gets_a_durable_uuid_v7_identity() {
        let store = LocalDataStore::open_in_memory().unwrap();
        let mut ids = HashSet::new();

        for kind in EntityKind::ALL {
            let entity = store.write(|transaction| transaction.create_entity(kind)).unwrap();
            let parsed_id = uuid::Uuid::parse_str(&entity.id).unwrap();

            assert_eq!(parsed_id.get_version(), Some(Version::SortRand));
            assert_eq!(entity.kind, kind);
            assert_eq!(entity.revision, 1);
            assert_eq!(entity.deleted_at, None);
            assert!(ids.insert(entity.id));
        }

        let changes = store.changes_after(0, 100).unwrap();

        assert_eq!(changes.len(), EntityKind::ALL.len());
        assert!(changes
            .iter()
            .all(|change| change.operation == ChangeOperation::Create));
    }

    #[test]
    fn updates_and_deletions_advance_metadata_and_append_changes() {
        let store = LocalDataStore::open_in_memory().unwrap();
        let created = store
            .write(|transaction| transaction.create_entity(EntityKind::Concept))
            .unwrap();
        let updated = store
            .write(|transaction| transaction.touch_entity(&created.id))
            .unwrap();
        let deleted = store
            .write(|transaction| transaction.soft_delete_entity(&created.id))
            .unwrap();
        let deleted_again = store
            .write(|transaction| transaction.soft_delete_entity(&created.id))
            .unwrap();

        assert_eq!(updated.revision, 2);
        assert!(updated.updated_at >= created.updated_at);
        assert_eq!(deleted.revision, 3);
        assert_eq!(deleted.deleted_at, Some(deleted.updated_at));
        assert_eq!(deleted_again, deleted);

        let stored = store.entity(&created.id).unwrap().unwrap();
        let changes = store.changes_after(0, 100).unwrap();
        let operations: Vec<_> = changes.iter().map(|change| change.operation).collect();

        assert_eq!(stored, deleted);
        assert_eq!(
            operations,
            vec![
                ChangeOperation::Create,
                ChangeOperation::Update,
                ChangeOperation::Delete,
            ]
        );

        let update_result = store.write(|transaction| transaction.touch_entity(&created.id));

        assert!(matches!(update_result, Err(DataError::EntityDeleted(_))));
    }

    #[test]
    fn failed_writes_roll_back_entities_and_their_changes() {
        let store = LocalDataStore::open_in_memory().unwrap();

        let result: DataResult<()> = store.write(|transaction| {
            transaction.create_entity(EntityKind::Concept)?;

            Err(DataError::EntityNotFound("forced failure".to_owned()))
        });

        assert!(result.is_err());
        assert!(store.changes_after(0, 100).unwrap().is_empty());

        let entity_count: i64 = store
            .connection()
            .unwrap()
            .query_row("SELECT COUNT(*) FROM entities", [], |row| row.get(0))
            .unwrap();

        assert_eq!(entity_count, 0);
    }

    #[test]
    fn change_history_cannot_be_mutated_or_removed() {
        let store = LocalDataStore::open_in_memory().unwrap();
        let entity = store
            .write(|transaction| transaction.create_entity(EntityKind::Review))
            .unwrap();

        let update = store.write(|transaction| {
            transaction.execute(
                "UPDATE change_log SET recorded_at = recorded_at + 1 WHERE entity_id = ?1",
                [&entity.id],
            )?;

            Ok(())
        });
        let delete = store.write(|transaction| {
            transaction.execute("DELETE FROM change_log WHERE entity_id = ?1", [&entity.id])?;

            Ok(())
        });

        assert!(update.is_err());
        assert!(delete.is_err());
        assert_eq!(store.changes_after(0, 100).unwrap().len(), 1);
    }
}
