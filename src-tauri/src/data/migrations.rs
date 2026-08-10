use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

use crate::data::DataResult;

pub fn apply(connection: &mut Connection) -> DataResult<()> {
    migrations().to_latest(connection)?;

    Ok(())
}

fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        M::up(include_str!(
            "../../migrations/0001_local_data_foundation.sql"
        )),
        M::up(include_str!(
            "../../migrations/0002_concept_library.sql"
        )),
        M::up(include_str!(
            "../../migrations/0003_rich_content_authoring.sql"
        )),
    ])
}

#[cfg(test)]
mod tests {
    use rusqlite::{Connection, OptionalExtension};
    use rusqlite_migration::{Migrations, M};

    use super::{apply, migrations};

    #[test]
    fn migrations_are_valid_and_reapplying_is_safe() {
        let mut connection = Connection::open_in_memory().unwrap();

        migrations().validate().unwrap();
        apply(&mut connection).unwrap();
        apply(&mut connection).unwrap();

        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();

        assert_eq!(version, 3);
    }

    #[test]
    fn the_concept_library_migrates_an_existing_foundation_database() {
        let mut connection = Connection::open_in_memory().unwrap();

        migrations().to_version(&mut connection, 1).unwrap();

        let transaction = connection.transaction().unwrap();

        transaction
            .execute(
                "INSERT INTO change_log (id, entity_id, operation, recorded_at)
                VALUES (?1, ?2, 'create', 1)",
                [
                    "018f1e2d-3c4b-7a69-8f10-123456789abc",
                    "018f1e2d-3c4b-7a69-8f10-123456789abd",
                ],
            )
            .unwrap();
        transaction
            .execute(
                "INSERT INTO entities (
                    id,
                    kind,
                    created_at,
                    updated_at,
                    deleted_at,
                    revision,
                    last_change_id
                ) VALUES (?1, 'concept', 1, 1, NULL, 1, ?2)",
                [
                    "018f1e2d-3c4b-7a69-8f10-123456789abd",
                    "018f1e2d-3c4b-7a69-8f10-123456789abc",
                ],
            )
            .unwrap();

        transaction.commit().unwrap();

        apply(&mut connection).unwrap();

        let entity_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM entities", [], |row| row.get(0))
            .unwrap();
        let concept_table: Option<String> = connection
            .query_row(
                "SELECT name FROM sqlite_master WHERE name = 'concepts'",
                [],
                |row| row.get(0),
            )
            .optional()
            .unwrap();

        assert_eq!(entity_count, 1);
        assert_eq!(concept_table.as_deref(), Some("concepts"));
    }

    #[test]
    fn rich_content_is_added_to_existing_concepts() {
        let mut connection = Connection::open_in_memory().unwrap();

        migrations().to_version(&mut connection, 2).unwrap();

        let transaction = connection.transaction().unwrap();

        transaction
            .execute(
                "INSERT INTO change_log (id, entity_id, operation, recorded_at)
                VALUES (?1, ?2, 'create', 1)",
                [
                    "018f1e2d-3c4b-7a69-8f10-123456789abc",
                    "018f1e2d-3c4b-7a69-8f10-123456789abd",
                ],
            )
            .unwrap();
        transaction
            .execute(
                "INSERT INTO entities (
                    id,
                    kind,
                    created_at,
                    updated_at,
                    deleted_at,
                    revision,
                    last_change_id
                ) VALUES (?1, 'concept', 1, 1, NULL, 1, ?2)",
                [
                    "018f1e2d-3c4b-7a69-8f10-123456789abd",
                    "018f1e2d-3c4b-7a69-8f10-123456789abc",
                ],
            )
            .unwrap();
        transaction
            .execute(
                "INSERT INTO concepts (entity_id, title, archived_at, last_change_id)
                VALUES (?1, 'Existing concept', NULL, ?2)",
                [
                    "018f1e2d-3c4b-7a69-8f10-123456789abd",
                    "018f1e2d-3c4b-7a69-8f10-123456789abc",
                ],
            )
            .unwrap();

        transaction.commit().unwrap();

        apply(&mut connection).unwrap();

        let content: String = connection
            .query_row("SELECT content_json FROM concepts", [], |row| row.get(0))
            .unwrap();
        let media_table: Option<String> = connection
            .query_row(
                "SELECT name FROM sqlite_master WHERE name = 'media'",
                [],
                |row| row.get(0),
            )
            .optional()
            .unwrap();

        assert_eq!(serde_json::from_str::<serde_json::Value>(&content).unwrap()["schemaVersion"], 1);
        assert_eq!(media_table.as_deref(), Some("media"));
    }

    #[test]
    fn a_failed_migration_rolls_back_the_whole_schema_update() {
        let migrations = Migrations::new(vec![
            M::up("CREATE TABLE survives_only_on_success (id INTEGER);"),
            M::up("CREATE TABLE this is not valid SQL;"),
        ]);
        let mut connection = Connection::open_in_memory().unwrap();

        assert!(migrations.to_latest(&mut connection).is_err());

        let table: Option<String> = connection
            .query_row(
                "SELECT name FROM sqlite_master WHERE name = 'survives_only_on_success'",
                [],
                |row| row.get(0),
            )
            .optional()
            .unwrap();
        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();

        assert_eq!(table, None);
        assert_eq!(version, 0);
    }

    #[test]
    fn a_newer_schema_is_never_downgraded() {
        let mut connection = Connection::open_in_memory().unwrap();

        connection
            .pragma_update(None, "user_version", 4)
            .unwrap();

        assert!(apply(&mut connection).is_err());

        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();

        assert_eq!(version, 4);
    }
}
