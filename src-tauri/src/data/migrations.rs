use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

use crate::data::DataResult;

pub fn apply(connection: &mut Connection) -> DataResult<()> {
    migrations().to_latest(connection)?;

    Ok(())
}

fn migrations() -> Migrations<'static> {
    Migrations::new(vec![M::up(include_str!(
        "../../migrations/0001_local_data_foundation.sql"
    ))])
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

        assert_eq!(version, 1);
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
            .pragma_update(None, "user_version", 2)
            .unwrap();

        assert!(apply(&mut connection).is_err());

        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();

        assert_eq!(version, 2);
    }
}
