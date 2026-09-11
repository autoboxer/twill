use crate::library::{LibraryError, LibraryResult};

const MAXIMUM_QUERY_LENGTH: usize = 250;

pub fn search_expression(query: &str) -> LibraryResult<String> {
    let query = query.trim();

    if query.chars().count() > MAXIMUM_QUERY_LENGTH {
        return Err(LibraryError::ValueTooLong {
            field: "Search",
            maximum: MAXIMUM_QUERY_LENGTH,
        });
    }

    // Quote user input so FTS operators and punctuation cannot change the query grammar
    Ok(query
        .split(|character: char| character.is_whitespace() || character == '\0')
        .filter(|term| !term.is_empty())
        .map(|term| format!("\"{}\"*", term.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" AND "))
}
