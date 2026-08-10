use rusqlite::{params, Connection, Transaction};
use rusqlite_migration::{HookError, HookResult, Migrations, M};

use crate::data::DataResult;

use super::store::{current_timestamp, new_id};

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
        M::up_with_hook("SELECT 1;", backfill_recall_cards)
            .comment("Create a default recall card for existing concepts"),
        M::up(include_str!(
            "../../migrations/0005_fsrs_scheduling.sql"
        )),
    ])
}

fn backfill_recall_cards(transaction: &Transaction<'_>) -> HookResult {
    let mut statement = transaction.prepare(
        "SELECT concepts.entity_id
        FROM concepts
        INNER JOIN entities AS concept_entities
            ON concept_entities.id = concepts.entity_id
        WHERE concept_entities.deleted_at IS NULL
            AND NOT EXISTS (
                SELECT 1
                FROM cards
                WHERE cards.concept_id = concepts.entity_id
            )
        ORDER BY concept_entities.created_at, concepts.entity_id",
    )?;
    let concept_ids = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    let timestamp = current_timestamp().map_err(|error| HookError::Hook(error.to_string()))?;

    for concept_id in concept_ids {
        let card_id = new_id();
        let change_id = new_id();

        transaction.execute(
            "INSERT INTO change_log (id, entity_id, operation, recorded_at)
            VALUES (?1, ?2, 'create', ?3)",
            params![&change_id, &card_id, timestamp],
        )?;
        transaction.execute(
            "INSERT INTO entities (
                id,
                kind,
                created_at,
                updated_at,
                deleted_at,
                revision,
                last_change_id
            ) VALUES (?1, 'card', ?2, ?2, NULL, 1, ?3)",
            params![&card_id, timestamp, &change_id],
        )?;
        transaction.execute(
            "INSERT INTO cards (entity_id, concept_id, last_change_id)
            VALUES (?1, ?2, ?3)",
            params![card_id, concept_id, change_id],
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use rusqlite::{Connection, OptionalExtension};
    use rusqlite_migration::{Migrations, M};
    use uuid::Version;

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

        assert_eq!(version, 5);
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
    fn recall_cards_are_added_to_existing_concepts() {
        let mut connection = Connection::open_in_memory().unwrap();

        migrations().to_version(&mut connection, 3).unwrap();

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
                "INSERT INTO concepts (
                    entity_id,
                    title,
                    archived_at,
                    last_change_id,
                    content_json
                ) VALUES (?1, 'Existing concept', NULL, ?2, ?3)",
                (
                    "018f1e2d-3c4b-7a69-8f10-123456789abd",
                    "018f1e2d-3c4b-7a69-8f10-123456789abc",
                    r#"{"schemaVersion":1,"prompt":{"type":"doc","content":[{"type":"paragraph"}]},"answer":{"type":"doc","content":[{"type":"paragraph"}]}}"#,
                ),
            )
            .unwrap();

        transaction.commit().unwrap();

        apply(&mut connection).unwrap();
        apply(&mut connection).unwrap();

        let card_id: String = connection
            .query_row("SELECT entity_id FROM cards", [], |row| row.get(0))
            .unwrap();
        let (kind, operation): (String, String) = connection
            .query_row(
                "SELECT entities.kind, change_log.operation
                FROM entities
                INNER JOIN change_log
                    ON change_log.id = entities.last_change_id
                WHERE entities.id = ?1",
                [&card_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        let card_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM cards", [], |row| row.get(0))
            .unwrap();

        assert_eq!(
            uuid::Uuid::parse_str(&card_id).unwrap().get_version(),
            Some(Version::SortRand)
        );
        assert_eq!(kind, "card");
        assert_eq!(operation, "create");
        assert_eq!(card_count, 1);
    }

    #[test]
    fn scheduling_is_added_to_existing_recall_cards() {
        let mut connection = Connection::open_in_memory().unwrap();

        migrations().to_version(&mut connection, 3).unwrap();

        let transaction = connection.transaction().unwrap();

        transaction
            .execute(
                "INSERT INTO change_log (id, entity_id, operation, recorded_at)
                VALUES (?1, ?2, 'create', 1000)",
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
                ) VALUES (?1, 'concept', 1000, 1000, NULL, 1, ?2)",
                [
                    "018f1e2d-3c4b-7a69-8f10-123456789abd",
                    "018f1e2d-3c4b-7a69-8f10-123456789abc",
                ],
            )
            .unwrap();
        transaction
            .execute(
                "INSERT INTO concepts (
                    entity_id,
                    title,
                    archived_at,
                    last_change_id,
                    content_json
                ) VALUES (?1, 'Existing concept', NULL, ?2, ?3)",
                (
                    "018f1e2d-3c4b-7a69-8f10-123456789abd",
                    "018f1e2d-3c4b-7a69-8f10-123456789abc",
                    r#"{"schemaVersion":1,"prompt":{"type":"doc","content":[{"type":"paragraph"}]},"answer":{"type":"doc","content":[{"type":"paragraph"}]}}"#,
                ),
            )
            .unwrap();
        transaction.commit().unwrap();

        migrations().to_version(&mut connection, 4).unwrap();

        let card_id: String = connection
            .query_row("SELECT entity_id FROM cards", [], |row| row.get(0))
            .unwrap();
        let card_created_at: i64 = connection
            .query_row(
                "SELECT created_at FROM entities WHERE id = ?1",
                [&card_id],
                |row| row.get(0),
            )
            .unwrap();

        apply(&mut connection).unwrap();

        let configuration: (String, String, String, f64) = connection
            .query_row(
                "SELECT
                    scheduler_configurations.id,
                    scheduler_configurations.algorithm,
                    scheduler_configurations.algorithm_version,
                    scheduler_configurations.desired_retention
                FROM active_scheduler_configuration
                INNER JOIN scheduler_configurations
                    ON scheduler_configurations.id = configuration_id",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        let parameters_json: String = connection
            .query_row(
                "SELECT parameters_json
                FROM scheduler_configurations
                WHERE id = 'fsrs-6.6.1-default-0.90'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let parameters: Vec<f32> = serde_json::from_str(&parameters_json).unwrap();
        let schedule: (String, i64, i64) = connection
            .query_row(
                "SELECT state, due_at, review_count
                FROM card_scheduling
                WHERE card_id = ?1",
                [&card_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();

        assert_eq!(configuration.0, "fsrs-6.6.1-default-0.90");
        assert_eq!(configuration.1, "fsrs");
        assert_eq!(configuration.2, "6.6.1");
        assert_eq!(configuration.3, 0.9);
        assert_eq!(parameters, fsrs::DEFAULT_PARAMETERS);
        assert_eq!(schedule, ("new".to_owned(), card_created_at, 0));
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
            .pragma_update(None, "user_version", 6)
            .unwrap();

        assert!(apply(&mut connection).is_err());

        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();

        assert_eq!(version, 6);
    }
}
