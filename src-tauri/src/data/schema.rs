use std::sync::OnceLock;

use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

use crate::data::DataResult;

pub fn ensure_current(connection: &mut Connection) -> DataResult<()> {
    schema_migrations().to_latest(connection)?;

    Ok(())
}

fn schema_migrations() -> Migrations<'static> {
    Migrations::new(vec![M::up(initial_schema())])
}

fn initial_schema() -> &'static str {
    static INITIAL_SCHEMA: OnceLock<String> = OnceLock::new();

    INITIAL_SCHEMA.get_or_init(|| {
        [
            include_str!("../../schema/local_data.sql"),
            include_str!("../../schema/library.sql"),
            include_str!("../../schema/rich_content.sql"),
            include_str!("../../schema/scheduling.sql"),
            include_str!("../../schema/preferences.sql"),
            include_str!("../../schema/css_snippets.sql"),
            include_str!("../../schema/authoring_drafts.sql"),
        ]
        .join("\n")
    })
}

#[cfg(test)]
mod tests {
    use rusqlite::{Connection, OptionalExtension};
    use rusqlite_migration::{Migrations, M};

    use super::{ensure_current, schema_migrations};

    #[test]
    fn initial_schema_is_valid_and_reapplying_is_safe() {
        let mut connection = Connection::open_in_memory().unwrap();

        schema_migrations().validate().unwrap();
        ensure_current(&mut connection).unwrap();
        ensure_current(&mut connection).unwrap();

        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();

        assert_eq!(version, 1);
    }

    #[test]
    fn initial_schema_contains_current_study_defaults() {
        let mut connection = Connection::open_in_memory().unwrap();

        ensure_current(&mut connection).unwrap();

        let configuration: (String, String, String, f64, i64) = connection
            .query_row(
                "SELECT
                    scheduler_configurations.id,
                    scheduler_configurations.algorithm,
                    scheduler_configurations.algorithm_version,
                    scheduler_configurations.desired_retention,
                    scheduler_configurations.maximum_interval_days
                FROM active_scheduler_configuration
                INNER JOIN scheduler_configurations
                    ON scheduler_configurations.id = configuration_id
                WHERE singleton = 1",
                [],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )
            .unwrap();
        let parameters_json: String = connection
            .query_row(
                "SELECT parameters_json
                FROM scheduler_configurations
                WHERE id = ?1",
                [&configuration.0],
                |row| row.get(0),
            )
            .unwrap();
        let parameters: Vec<f32> = serde_json::from_str(&parameters_json).unwrap();
        let preferences: (String, String, String, String, String, String) = connection
            .query_row(
                "SELECT
                    grading_mode,
                    startup_destination,
                    theme,
                    reading_font,
                    reading_text_size,
                    motion_preference
                FROM device_preferences
                WHERE singleton = 1",
                [],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                    ))
                },
            )
            .unwrap();

        assert_eq!(configuration.0, "fsrs-6.6.1-default-0.90");
        assert_eq!(configuration.1, "fsrs");
        assert_eq!(configuration.2, "6.6.1");
        assert_eq!(configuration.3, 0.9);
        assert_eq!(configuration.4, 36_500);
        assert_eq!(parameters, fsrs::DEFAULT_PARAMETERS);
        assert_eq!(
            preferences,
            (
                "simple".to_owned(),
                "study".to_owned(),
                "aubergine".to_owned(),
                "inter".to_owned(),
                "medium".to_owned(),
                "system".to_owned(),
            )
        );
    }

    #[test]
    fn a_failed_schema_installation_rolls_back_the_whole_update() {
        let invalid_schema = Migrations::new(vec![
            M::up("CREATE TABLE survives_only_on_success (id INTEGER);"),
            M::up("CREATE TABLE this is not valid SQL;"),
        ]);
        let mut connection = Connection::open_in_memory().unwrap();

        assert!(invalid_schema.to_latest(&mut connection).is_err());

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
            .pragma_update(None, "user_version", 2)
            .unwrap();

        assert!(ensure_current(&mut connection).is_err());

        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();

        assert_eq!(version, 2);
    }
}
