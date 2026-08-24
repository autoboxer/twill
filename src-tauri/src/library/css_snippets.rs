use std::fmt;

use cssparser::{
    AtRuleParser, BasicParseErrorKind, CowRcStr, DeclarationParser, ParseError,
    ParseErrorKind, Parser, ParserInput, ParserState, QualifiedRuleParser,
    RuleBodyItemParser, RuleBodyParser, StyleSheetParser, Token,
};
use rusqlite::{params, Connection, OptionalExtension};

use crate::data::{EntityKind, LocalDataStore};
use crate::library::{
    CreateCssSnippetInput, CssSnippet, CssSnippetCatalog, CssSnippetContent,
    LibraryError, LibraryResult, UpdateCssSnippetInput, CSS_SNIPPET_SCHEMA_VERSION,
};

const MAXIMUM_CSS_NESTING_DEPTH: usize = 32;
const MAXIMUM_CSS_SNIPPET_NAME_LENGTH: usize = 80;
const MAXIMUM_CSS_SNIPPET_SOURCE_BYTES: usize = 100_000;

pub struct CssSnippetLibrary<'store> {
    store: &'store LocalDataStore,
}

impl<'store> CssSnippetLibrary<'store> {
    pub fn new(store: &'store LocalDataStore) -> Self {
        Self { store }
    }

    pub fn catalog(&self) -> LibraryResult<CssSnippetCatalog> {
        self.store.read_result(query_catalog)
    }

    pub fn create_snippet(
        &self,
        input: CreateCssSnippetInput,
    ) -> LibraryResult<CssSnippet> {
        let name = normalize_name(input.name)?;
        let content = validate_content(input.content)?;

        self.store.write_result(|transaction| {
            ensure_unique_name(transaction, &name, None)?;

            let entity = transaction.create_entity(EntityKind::CssSnippet)?;

            transaction.execute(
                "INSERT INTO css_snippets (
                    entity_id,
                    name,
                    schema_version,
                    source,
                    last_change_id
                ) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    entity.id,
                    name,
                    content.schema_version,
                    content.source,
                    entity.last_change_id,
                ],
            )?;
            transaction.execute(
                "INSERT INTO device_css_snippet_preferences (snippet_id, enabled)
                VALUES (?1, 0)",
                [&entity.id],
            )?;

            query_snippet(transaction, &entity.id)
        })
    }

    pub fn update_snippet(
        &self,
        input: UpdateCssSnippetInput,
    ) -> LibraryResult<CssSnippet> {
        let id = input.id.trim().to_owned();
        let name = normalize_name(input.name)?;
        let content = validate_content(input.content)?;

        self.store.write_result(|transaction| {
            let current = query_snippet(transaction, &id)?;

            if current.name == name && current.content == content {
                return Ok(current);
            }

            ensure_unique_name(transaction, &name, Some(&id))?;

            let entity = transaction.touch_entity(&id)?;

            transaction.execute(
                "UPDATE css_snippets
                SET name = ?1,
                    schema_version = ?2,
                    source = ?3,
                    last_change_id = ?4
                WHERE entity_id = ?5",
                params![
                    name,
                    content.schema_version,
                    content.source,
                    entity.last_change_id,
                    id,
                ],
            )?;

            query_snippet(transaction, &id)
        })
    }

    pub fn set_enabled(&self, id: &str, enabled: bool) -> LibraryResult<CssSnippet> {
        let id = id.trim().to_owned();

        self.store.write_result(|transaction| {
            query_snippet(transaction, &id)?;
            transaction.execute(
                "INSERT INTO device_css_snippet_preferences (snippet_id, enabled)
                VALUES (?1, ?2)
                ON CONFLICT(snippet_id) DO UPDATE SET enabled = excluded.enabled",
                params![id, enabled],
            )?;

            query_snippet(transaction, &id)
        })
    }

    pub fn disable_all(&self) -> LibraryResult<()> {
        self.store.write_result(|transaction| {
            transaction.execute(
                "UPDATE device_css_snippet_preferences SET enabled = 0 WHERE enabled = 1",
                [],
            )?;

            Ok(())
        })
    }

    pub fn delete_snippet(&self, id: &str) -> LibraryResult<()> {
        let id = id.trim().to_owned();

        self.store.write_result(|transaction| {
            if !snippet_record_exists(transaction, &id)? {
                return Err(LibraryError::CssSnippetNotFound(id));
            }

            transaction.soft_delete_entity(&id)?;

            Ok(())
        })
    }
}

fn normalize_name(name: String) -> LibraryResult<String> {
    let name = name.trim().to_owned();

    if name.is_empty() {
        return Err(LibraryError::EmptyValue {
            field: "CSS snippet name",
        });
    }

    if name.chars().count() > MAXIMUM_CSS_SNIPPET_NAME_LENGTH {
        return Err(LibraryError::ValueTooLong {
            field: "CSS snippet name",
            maximum: MAXIMUM_CSS_SNIPPET_NAME_LENGTH,
        });
    }

    Ok(name)
}

fn validate_content(mut content: CssSnippetContent) -> LibraryResult<CssSnippetContent> {
    if content.schema_version != CSS_SNIPPET_SCHEMA_VERSION {
        return Err(invalid_css("uses an unsupported schema version"));
    }

    if content.source.len() > MAXIMUM_CSS_SNIPPET_SOURCE_BYTES {
        return Err(invalid_css("cannot be larger than 100 KB"));
    }

    if content.source.contains('\0') {
        return Err(invalid_css("contains an invalid character"));
    }

    content.source = content.source.trim().to_owned();

    if content.source.is_empty() {
        return Err(invalid_css("cannot be empty"));
    }

    validate_stylesheet(&content.source)?;

    Ok(content)
}

fn validate_stylesheet(source: &str) -> LibraryResult<()> {
    if !has_balanced_css_delimiters(source) {
        return Err(invalid_css("contains unclosed or unmatched syntax"));
    }

    let mut input = ParserInput::new(source);
    let mut input = Parser::new(&mut input);
    let mut validator = CssValidator { nesting_depth: 0 };
    let mut rule_count = 0;

    for result in StyleSheetParser::new(&mut input, &mut validator) {
        result.map_err(|(error, _)| css_parse_error(error))?;
        rule_count += 1;
    }

    if rule_count == 0 {
        return Err(invalid_css("must contain at least one rule"));
    }

    Ok(())
}

fn has_balanced_css_delimiters(source: &str) -> bool {
    let bytes = source.as_bytes();
    let mut delimiters = Vec::new();
    let mut index = 0;
    let mut quote = None;
    let mut in_comment = false;

    while index < bytes.len() {
        if in_comment {
            if bytes[index..].starts_with(b"*/") {
                in_comment = false;
                index += 2;
            } else {
                index += 1;
            }

            continue;
        }

        if let Some(active_quote) = quote {
            match bytes[index] {
                b'\\' => index = css_escape_end(bytes, index),
                byte if byte == active_quote => {
                    quote = None;
                    index += 1;
                }
                b'\n' | b'\r' | b'\x0c' => return false,
                _ => index += 1,
            }

            continue;
        }

        if bytes[index..].starts_with(b"/*") {
            in_comment = true;
            index += 2;
            continue;
        }

        match bytes[index] {
            b'\'' | b'"' => {
                quote = Some(bytes[index]);
                index += 1;
            }
            b'\\' => index = css_escape_end(bytes, index),
            b'(' | b'[' | b'{' => {
                delimiters.push(bytes[index]);
                index += 1;
            }
            b')' => {
                if delimiters.pop() != Some(b'(') {
                    return false;
                }

                index += 1;
            }
            b']' => {
                if delimiters.pop() != Some(b'[') {
                    return false;
                }

                index += 1;
            }
            b'}' => {
                if delimiters.pop() != Some(b'{') {
                    return false;
                }

                index += 1;
            }
            _ => index += 1,
        }
    }

    !in_comment && quote.is_none() && delimiters.is_empty()
}

fn css_escape_end(source: &[u8], slash_index: usize) -> usize {
    let mut index = slash_index + 1;

    if index >= source.len() {
        return index;
    }

    if source[index].is_ascii_hexdigit() {
        let mut digits = 0;

        while index < source.len() && source[index].is_ascii_hexdigit() && digits < 6 {
            index += 1;
            digits += 1;
        }

        if index < source.len() && source[index].is_ascii_whitespace() {
            if source[index] == b'\r'
                && source.get(index + 1) == Some(&b'\n')
            {
                return index + 2;
            }

            return index + 1;
        }

        return index;
    }

    index + 1
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum CssParseIssue {
    Invalid(&'static str),
    Unsafe(&'static str),
}

impl fmt::Display for CssParseIssue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Invalid(message) | Self::Unsafe(message) => message,
        };

        formatter.write_str(message)
    }
}

struct CssAtRule {
    allows_no_block: bool,
}

struct CssValidator {
    nesting_depth: usize,
}

impl CssValidator {
    fn parse_rule_body<'i, 't>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> Result<(), ParseError<'i, CssParseIssue>> {
        if self.nesting_depth >= MAXIMUM_CSS_NESTING_DEPTH {
            return Err(input.new_custom_error(CssParseIssue::Invalid(
                "is nested too deeply",
            )));
        }

        self.nesting_depth += 1;

        let result = {
            let body = RuleBodyParser::new(input, self);

            body.collect::<Result<Vec<_>, _>>()
        };

        self.nesting_depth -= 1;

        result
            .map(|_| ())
            .map_err(|(error, _)| error)
    }
}

impl<'i> DeclarationParser<'i> for CssValidator {
    type Declaration = ();
    type Error = CssParseIssue;

    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
        _declaration_start: &ParserState,
    ) -> Result<Self::Declaration, ParseError<'i, Self::Error>> {
        if matches_ignore_ascii_case(&name, &["behavior", "-ms-behavior", "-moz-binding"])
        {
            return Err(input.new_custom_error(CssParseIssue::Unsafe(
                "contains an executable legacy property",
            )));
        }

        if !validate_component_values(input, 0)? {
            return Err(input.new_custom_error(CssParseIssue::Invalid(
                "contains an empty declaration",
            )));
        }

        Ok(())
    }
}

impl<'i> AtRuleParser<'i> for CssValidator {
    type Prelude = CssAtRule;
    type AtRule = ();
    type Error = CssParseIssue;

    fn parse_prelude<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, Self::Error>> {
        if matches_ignore_ascii_case(&name, &["import", "font-face", "charset", "namespace"])
        {
            return Err(input.new_custom_error(CssParseIssue::Unsafe(
                "contains a restricted at-rule",
            )));
        }

        validate_component_values(input, 0)?;

        Ok(CssAtRule {
            allows_no_block: name.eq_ignore_ascii_case("layer"),
        })
    }

    fn rule_without_block(
        &mut self,
        prelude: Self::Prelude,
        _start: &ParserState,
    ) -> Result<Self::AtRule, ()> {
        prelude.allows_no_block.then_some(()).ok_or(())
    }

    fn parse_block<'t>(
        &mut self,
        _prelude: Self::Prelude,
        _start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::AtRule, ParseError<'i, Self::Error>> {
        self.parse_rule_body(input)
    }
}

impl<'i> QualifiedRuleParser<'i> for CssValidator {
    type Prelude = ();
    type QualifiedRule = ();
    type Error = CssParseIssue;

    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, Self::Error>> {
        if !validate_component_values(input, 0)? {
            return Err(input.new_custom_error(CssParseIssue::Invalid(
                "contains a rule without a selector",
            )));
        }

        Ok(())
    }

    fn parse_block<'t>(
        &mut self,
        _prelude: Self::Prelude,
        _start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::QualifiedRule, ParseError<'i, Self::Error>> {
        self.parse_rule_body(input)
    }
}

impl RuleBodyItemParser<'_, (), CssParseIssue> for CssValidator {
    fn parse_declarations(&self) -> bool {
        true
    }

    fn parse_qualified(&self) -> bool {
        true
    }
}

fn validate_component_values<'i, 't>(
    input: &mut Parser<'i, 't>,
    depth: usize,
) -> Result<bool, ParseError<'i, CssParseIssue>> {
    if depth >= MAXIMUM_CSS_NESTING_DEPTH {
        return Err(input.new_custom_error(CssParseIssue::Invalid(
            "is nested too deeply",
        )));
    }

    let mut has_value = false;

    while let Ok(token) = input.next().cloned() {
        has_value = true;

        match token {
            Token::UnquotedUrl(_) => {
                return Err(input.new_custom_error(CssParseIssue::Unsafe(
                    "cannot load external resources",
                )));
            }
            Token::Function(name) => {
                if matches_ignore_ascii_case(
                    &name,
                    &[
                        "url",
                        "src",
                        "image",
                        "image-set",
                        "-webkit-image-set",
                    ],
                ) {
                    return Err(input.new_custom_error(CssParseIssue::Unsafe(
                        "cannot load external resources",
                    )));
                }

                if name.eq_ignore_ascii_case("expression") {
                    return Err(input.new_custom_error(CssParseIssue::Unsafe(
                        "contains an executable legacy function",
                    )));
                }

                input.parse_nested_block(|input| {
                    validate_component_values(input, depth + 1).map(|_| ())
                })?;
            }
            Token::ParenthesisBlock | Token::SquareBracketBlock | Token::CurlyBracketBlock => {
                input.parse_nested_block(|input| {
                    validate_component_values(input, depth + 1).map(|_| ())
                })?;
            }
            Token::BadUrl(_)
            | Token::BadString(_)
            | Token::CloseParenthesis
            | Token::CloseSquareBracket
            | Token::CloseCurlyBracket => {
                return Err(input.new_custom_error(CssParseIssue::Invalid(
                    "contains invalid syntax",
                )));
            }
            _ => {}
        }
    }

    Ok(has_value)
}

fn matches_ignore_ascii_case(value: &str, candidates: &[&str]) -> bool {
    candidates
        .iter()
        .any(|candidate| value.eq_ignore_ascii_case(candidate))
}

fn css_parse_error(error: ParseError<'_, CssParseIssue>) -> LibraryError {
    let message = match error.kind {
        ParseErrorKind::Custom(issue) => issue.to_string(),
        ParseErrorKind::Basic(BasicParseErrorKind::AtRuleInvalid(_)) => {
            "contains an invalid at-rule".to_owned()
        }
        ParseErrorKind::Basic(_) => "contains invalid syntax".to_owned(),
    };

    invalid_css(format!(
        "{message} near line {}, column {}",
        error.location.line + 1,
        error.location.column
    ))
}

fn query_catalog(connection: &Connection) -> LibraryResult<CssSnippetCatalog> {
    let mut statement = connection.prepare(
        "SELECT
            css_snippets.entity_id,
            css_snippets.name,
            entities.created_at,
            entities.updated_at,
            css_snippets.schema_version,
            css_snippets.source,
            COALESCE(device_css_snippet_preferences.enabled, 0)
        FROM css_snippets
        INNER JOIN entities ON entities.id = css_snippets.entity_id
        LEFT JOIN device_css_snippet_preferences
            ON device_css_snippet_preferences.snippet_id = css_snippets.entity_id
        WHERE entities.deleted_at IS NULL
        ORDER BY css_snippets.name COLLATE NOCASE, css_snippets.entity_id",
    )?;
    let snippets = statement.query_map([], snippet_from_row)?;

    Ok(CssSnippetCatalog {
        snippets: snippets.collect::<Result<_, _>>()?,
    })
}

fn query_snippet(connection: &Connection, id: &str) -> LibraryResult<CssSnippet> {
    connection
        .query_row(
            "SELECT
                css_snippets.entity_id,
                css_snippets.name,
                entities.created_at,
                entities.updated_at,
                css_snippets.schema_version,
                css_snippets.source,
                COALESCE(device_css_snippet_preferences.enabled, 0)
            FROM css_snippets
            INNER JOIN entities ON entities.id = css_snippets.entity_id
            LEFT JOIN device_css_snippet_preferences
                ON device_css_snippet_preferences.snippet_id = css_snippets.entity_id
            WHERE css_snippets.entity_id = ?1
                AND entities.deleted_at IS NULL",
            [id],
            snippet_from_row,
        )
        .optional()?
        .ok_or_else(|| LibraryError::CssSnippetNotFound(id.to_owned()))
}

fn snippet_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<CssSnippet> {
    Ok(CssSnippet {
        id: row.get(0)?,
        name: row.get(1)?,
        created_at: row.get(2)?,
        updated_at: row.get(3)?,
        content: CssSnippetContent {
            schema_version: row.get(4)?,
            source: row.get(5)?,
        },
        enabled: row.get(6)?,
    })
}

fn ensure_unique_name(
    connection: &Connection,
    name: &str,
    excluded_id: Option<&str>,
) -> LibraryResult<()> {
    let mut statement = connection.prepare(
        "SELECT css_snippets.entity_id, css_snippets.name
        FROM css_snippets
        INNER JOIN entities ON entities.id = css_snippets.entity_id
        WHERE entities.deleted_at IS NULL",
    )?;
    let names = statement.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let normalized_name = name.to_lowercase();

    for item in names {
        let (id, existing_name) = item?;

        if excluded_id == Some(id.as_str()) {
            continue;
        }

        if existing_name.to_lowercase() == normalized_name {
            return Err(LibraryError::DuplicateName {
                kind: "CSS snippet",
                name: name.to_owned(),
            });
        }
    }

    Ok(())
}

fn snippet_record_exists(connection: &Connection, id: &str) -> LibraryResult<bool> {
    Ok(connection.query_row(
        "SELECT EXISTS (SELECT 1 FROM css_snippets WHERE entity_id = ?1)",
        [id],
        |row| row.get(0),
    )?)
}

fn invalid_css(message: impl Into<String>) -> LibraryError {
    LibraryError::InvalidCss {
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::params;
    use tempfile::tempdir;

    use super::CssSnippetLibrary;
    use crate::data::{ChangeOperation, DataResult, EntityKind, LocalDataStore};
    use crate::library::{
        CreateCssSnippetInput, CssSnippetContent, LibraryError, UpdateCssSnippetInput,
        CSS_SNIPPET_SCHEMA_VERSION,
    };

    fn content(source: &str) -> CssSnippetContent {
        CssSnippetContent {
            schema_version: CSS_SNIPPET_SCHEMA_VERSION,
            source: source.to_owned(),
        }
    }

    #[test]
    fn snippets_are_sync_ready_while_enablement_stays_device_local() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let library = CssSnippetLibrary::new(&store);
        let created = library
            .create_snippet(CreateCssSnippetInput {
                name: "  Focused study  ".to_owned(),
                content: content(
                    "[data-twill-page='study'] { --ui-primary: rebeccapurple; }",
                ),
            })
            .unwrap();

        assert_eq!(created.name, "Focused study");
        assert!(!created.enabled);
        assert_eq!(
            store.entity(&created.id).unwrap().unwrap().kind,
            EntityKind::CssSnippet
        );
        assert_eq!(store.changes_after(0, 10).unwrap().len(), 1);

        let enabled = library.set_enabled(&created.id, true).unwrap();

        assert!(enabled.enabled);
        assert_eq!(store.changes_after(0, 10).unwrap().len(), 1);

        let updated = library
            .update_snippet(UpdateCssSnippetInput {
                id: created.id.clone(),
                name: "Focused recall".to_owned(),
                content: content(
                    "@media (width >= 40rem) {\n  [data-twill-page='study'] { opacity: .98; }\n}",
                ),
            })
            .unwrap();

        assert_eq!(updated.name, "Focused recall");
        assert!(updated.enabled);
        assert_eq!(store.entity(&created.id).unwrap().unwrap().revision, 2);

        let unchanged = library
            .update_snippet(UpdateCssSnippetInput {
                id: created.id.clone(),
                name: updated.name.clone(),
                content: updated.content.clone(),
            })
            .unwrap();

        assert_eq!(unchanged, updated);
        assert_eq!(store.entity(&created.id).unwrap().unwrap().revision, 2);

        library.disable_all().unwrap();
        assert!(!library.catalog().unwrap().snippets[0].enabled);
        assert_eq!(store.changes_after(0, 10).unwrap().len(), 2);

        library.delete_snippet(&created.id).unwrap();
        library.delete_snippet(&created.id).unwrap();

        assert!(library.catalog().unwrap().snippets.is_empty());
        assert!(matches!(
            library.set_enabled(&created.id, true),
            Err(LibraryError::CssSnippetNotFound(_))
        ));

        let operations: Vec<_> = store
            .changes_after(0, 10)
            .unwrap()
            .into_iter()
            .map(|change| change.operation)
            .collect();

        assert_eq!(
            operations,
            vec![
                ChangeOperation::Create,
                ChangeOperation::Update,
                ChangeOperation::Delete,
            ]
        );
    }

    #[test]
    fn snippet_names_versions_and_css_are_validated() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let library = CssSnippetLibrary::new(&store);
        let first = library
            .create_snippet(CreateCssSnippetInput {
                name: "Calm cards".to_owned(),
                content: content(
                    "@layer twill-personal {\n  [data-twill-study-card] { border-width: 2px; }\n}\n\n@keyframes settle { from { opacity: 0; } to { opacity: 1; } }",
                ),
            })
            .unwrap();

        assert!(matches!(
            library.create_snippet(CreateCssSnippetInput {
                name: " calm cards ".to_owned(),
                content: content("[data-twill-app] { color: inherit; }"),
            }),
            Err(LibraryError::DuplicateName { .. })
        ));

        let mut unsupported = content("[data-twill-app] { color: inherit; }");
        unsupported.schema_version = 2;

        assert!(matches!(
            library.update_snippet(UpdateCssSnippetInput {
                id: first.id.clone(),
                name: first.name.clone(),
                content: unsupported,
            }),
            Err(LibraryError::InvalidCss { .. })
        ));

        for invalid_source in [
            "",
            "@import 'https://example.com/styles.css';",
            "@i\\6dport 'https://example.com/escaped.css';",
            "@font-face { font-family: Unsafe; src: url(font.woff2); }",
            "@f\\6fnt-face { font-family: Unsafe; src: local(Unsafe); }",
            ".card { background: u\\72l(https://example.com/image.png); }",
            ".card { background: image-set('image.png' 1x); }",
            ".card { background: src('image.png'); }",
            ".card { width: ex\\70ression(alert(1)); }",
            ".card { b\\65havior: url(script.htc); }",
            ".card { color red; }",
            ".card { color: ; }",
            ".card { color: red;",
            ".card { content: 'unterminated; }",
            ".card { color: red; } /* unterminated",
        ] {
            assert!(
                matches!(
                    library.create_snippet(CreateCssSnippetInput {
                        name: format!("Invalid {}", invalid_source.len()),
                        content: content(invalid_source),
                    }),
                    Err(LibraryError::InvalidCss { .. })
                ),
                "accepted unsafe CSS: {invalid_source}"
            );
        }

        let oversized = "a".repeat(100_001);

        assert!(matches!(
            library.create_snippet(CreateCssSnippetInput {
                name: "Oversized".to_owned(),
                content: content(&oversized),
            }),
            Err(LibraryError::InvalidCss { .. })
        ));

        let location_error = library
            .create_snippet(CreateCssSnippetInput {
                name: "Location".to_owned(),
                content: content(".card { background: url(image.png); }"),
            })
            .unwrap_err();

        assert!(location_error.to_string().contains("near line 1, column"));
    }

    #[test]
    fn snippet_rows_cannot_bypass_change_tracking_or_tombstones() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let library = CssSnippetLibrary::new(&store);
        let snippet = library
            .create_snippet(CreateCssSnippetInput {
                name: "Protected".to_owned(),
                content: content("[data-twill-app] { color: inherit; }"),
            })
            .unwrap();

        let update = store.write(|transaction| {
            transaction.execute(
                "UPDATE css_snippets SET name = 'Untracked' WHERE entity_id = ?1",
                [&snippet.id],
            )?;

            Ok(())
        });
        let delete = store.write(|transaction| {
            transaction.execute(
                "DELETE FROM css_snippets WHERE entity_id = ?1",
                [&snippet.id],
            )?;

            Ok(())
        });

        assert!(matches!(update, Err(crate::data::DataError::Database(_))));
        assert!(matches!(delete, Err(crate::data::DataError::Database(_))));

        let preference_update: DataResult<()> = store.write(|transaction| {
            transaction.execute(
                "UPDATE device_css_snippet_preferences
                SET snippet_id = ?1
                WHERE snippet_id = ?2",
                params![
                    "018f1e2d-3c4b-7a69-8f10-123456789abc",
                    snippet.id,
                ],
            )?;

            Ok(())
        });

        assert!(matches!(
            preference_update,
            Err(crate::data::DataError::Database(_))
        ));
    }
}
