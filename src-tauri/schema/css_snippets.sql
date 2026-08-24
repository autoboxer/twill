CREATE TABLE css_snippets (
    entity_id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL CHECK (
        length(trim(name)) BETWEEN 1 AND 80
    ),
    schema_version INTEGER NOT NULL CHECK (schema_version = 1),
    source TEXT NOT NULL CHECK (
        length(CAST(source AS BLOB)) BETWEEN 1 AND 100000
        AND instr(source, char(0)) = 0
    ),
    last_change_id TEXT NOT NULL,
    FOREIGN KEY (entity_id) REFERENCES entities(id),
    FOREIGN KEY (last_change_id) REFERENCES change_log(id)
) STRICT;

CREATE TABLE device_css_snippet_preferences (
    snippet_id TEXT PRIMARY KEY NOT NULL,
    enabled INTEGER NOT NULL CHECK (enabled IN (0, 1)),
    FOREIGN KEY (snippet_id) REFERENCES css_snippets(entity_id)
) STRICT;

CREATE TRIGGER validate_css_snippet_insert
BEFORE INSERT ON css_snippets
FOR EACH ROW
WHEN NOT EXISTS (
    SELECT 1
    FROM entities
    WHERE id = NEW.entity_id
        AND kind = 'css_snippet'
        AND deleted_at IS NULL
        AND last_change_id = NEW.last_change_id
)
BEGIN
    SELECT RAISE(ABORT, 'CSS snippet requires a matching active entity');
END;

CREATE TRIGGER validate_css_snippet_update
BEFORE UPDATE ON css_snippets
FOR EACH ROW
WHEN NEW.entity_id != OLD.entity_id
    OR NEW.last_change_id = OLD.last_change_id
    OR NOT EXISTS (
        SELECT 1
        FROM entities
        WHERE id = NEW.entity_id
            AND kind = 'css_snippet'
            AND deleted_at IS NULL
            AND last_change_id = NEW.last_change_id
    )
BEGIN
    SELECT RAISE(ABORT, 'CSS snippet update requires a matching entity change');
END;

CREATE TRIGGER prevent_css_snippet_delete
BEFORE DELETE ON css_snippets
FOR EACH ROW
BEGIN
    SELECT RAISE(ABORT, 'CSS snippets must be deleted with an entity tombstone');
END;

CREATE TRIGGER validate_device_css_snippet_preference_update
BEFORE UPDATE ON device_css_snippet_preferences
FOR EACH ROW
WHEN NEW.snippet_id != OLD.snippet_id
BEGIN
    SELECT RAISE(ABORT, 'CSS snippet preference identity is immutable');
END;
