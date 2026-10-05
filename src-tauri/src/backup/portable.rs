use std::io::Write;

use rusqlite::{types::ValueRef, Connection};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use super::checksum::finish_digest;
use super::models::{LibraryCounts, ARCHIVE_FORMAT_VERSION};
use super::{BackupError, BackupResult};

// Authoring drafts are the largest supported persisted text value
const MAXIMUM_TEXT_BYTES: usize = 5_000_000;

pub(super) fn write_library(
    connection: &Connection,
    writer: &mut impl Write,
    created_at: i64,
) -> BackupResult<()> {
    write!(
        writer,
        "{{\n  \"format\": \"twillLibrary\",\n  \"formatVersion\": {},\n  \
        \"appVersion\": {},\n  \"createdAt\": {},\n  \
        \"mediaDirectory\": \"media\",\n  \"tables\": {{",
        ARCHIVE_FORMAT_VERSION,
        serde_json::to_string(env!("CARGO_PKG_VERSION"))?,
        created_at,
    )?;

    let mut statement = connection.prepare(
        "SELECT name FROM sqlite_schema
        WHERE type = 'table'
            AND name NOT LIKE 'sqlite_%'
            AND name NOT LIKE 'concept_search%'
            AND name NOT IN (
                'device_media_cleanup',
                'authoring_media_sessions',
                'authoring_session_media'
            )
        ORDER BY name",
    )?;
    let tables = statement.query_map([], |row| row.get::<_, String>(0))?;

    for (index, table) in tables.enumerate() {
        let table = table?;

        if index > 0 {
            write!(writer, ",")?;
        }

        write!(writer, "\n    {}: [", serde_json::to_string(&table)?)?;
        write_table(connection, writer, &table)?;
        write!(writer, "\n    ]")?;
    }

    writeln!(writer, "\n  }}\n}}")?;

    Ok(())
}

fn write_table(connection: &Connection, writer: &mut impl Write, table: &str) -> BackupResult<()> {
    let mut statement = connection.prepare(&format!("SELECT * FROM {}", quote_name(table)))?;
    let columns = statement
        .column_names()
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let mut rows = statement.query([])?;
    let mut first = true;

    while let Some(row) = rows.next()? {
        let mut record = Map::new();

        for (index, column) in columns.iter().enumerate() {
            let value = match row.get_ref(index)? {
                ValueRef::Null => Value::Null,
                ValueRef::Integer(value) => Value::from(value),
                ValueRef::Real(value) => Value::Number(
                    serde_json::Number::from_f64(value)
                        .ok_or(BackupError::Integrity("a number is not finite"))?,
                ),
                ValueRef::Text(bytes) => {
                    if bytes.len() > MAXIMUM_TEXT_BYTES {
                        return Err(BackupError::Integrity(
                            "a text value exceeds the supported size",
                        ));
                    }

                    let text = std::str::from_utf8(bytes)
                        .map_err(|_| BackupError::Integrity("text is not valid UTF-8"))?;

                    if column.ends_with("_json") {
                        serde_json::from_str(text)?
                    } else {
                        Value::String(text.to_owned())
                    }
                }
                ValueRef::Blob(_) => {
                    return Err(BackupError::Integrity("unexpected binary database content"));
                }
            };

            record.insert(column.clone(), value);
        }

        if !first {
            write!(writer, ",")?;
        }

        write!(writer, "\n      ")?;
        serde_json::to_writer(&mut *writer, &record)?;
        first = false;
    }

    Ok(())
}

pub(super) fn library_counts(connection: &Connection) -> BackupResult<LibraryCounts> {
    let active_count = |kind| {
        connection.query_row(
            "SELECT count(*) FROM entities WHERE kind = ?1 AND deleted_at IS NULL",
            [kind],
            |row| row.get::<_, i64>(0),
        )
    };

    Ok(LibraryCounts {
        concepts: active_count("concept")? as u64,
        cards: active_count("card")? as u64,
        reviews: connection.query_row("SELECT count(*) FROM reviews", [], |row| {
            row.get::<_, i64>(0)
        })? as u64,
        media: active_count("media")? as u64,
    })
}

pub(super) fn schema_fingerprint(connection: &Connection) -> BackupResult<String> {
    let mut digest = Sha256::new();
    let mut statement = connection.prepare(
        "SELECT type, name, tbl_name, sql FROM sqlite_schema
        WHERE name NOT LIKE 'sqlite_%'
        ORDER BY type, name",
    )?;
    let definitions = statement.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<String>>(3)?,
        ))
    })?;

    for definition in definitions {
        digest.update(serde_json::to_vec(&definition?)?);
        digest.update(b"\n");
    }

    Ok(finish_digest(digest))
}

fn quote_name(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}
