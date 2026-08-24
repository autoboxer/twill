CREATE TABLE change_log (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    id TEXT NOT NULL UNIQUE CHECK (length(id) = 36),
    entity_id TEXT NOT NULL CHECK (length(entity_id) = 36),
    operation TEXT NOT NULL CHECK (operation IN ('create', 'update', 'delete')),
    recorded_at INTEGER NOT NULL CHECK (recorded_at >= 0),
    FOREIGN KEY (entity_id) REFERENCES entities(id)
        DEFERRABLE INITIALLY DEFERRED
) STRICT;

CREATE TABLE entities (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) = 36),
    kind TEXT NOT NULL CHECK (
        kind IN (
            'concept',
            'card',
            'deck',
            'tag',
            'template',
            'review',
            'review_reversal',
            'media',
            'css_snippet'
        )
    ),
    created_at INTEGER NOT NULL CHECK (created_at >= 0),
    updated_at INTEGER NOT NULL CHECK (updated_at >= created_at),
    deleted_at INTEGER CHECK (
        deleted_at IS NULL
        OR (deleted_at >= created_at AND deleted_at <= updated_at)
    ),
    revision INTEGER NOT NULL CHECK (revision > 0),
    last_change_id TEXT NOT NULL UNIQUE,
    FOREIGN KEY (last_change_id) REFERENCES change_log(id)
        DEFERRABLE INITIALLY DEFERRED
) STRICT;

CREATE INDEX entities_kind_active_idx
    ON entities(kind, deleted_at);

CREATE INDEX change_log_entity_sequence_idx
    ON change_log(entity_id, sequence);

CREATE TRIGGER validate_entity_insert
BEFORE INSERT ON entities
FOR EACH ROW
BEGIN
    SELECT CASE
        WHEN NOT EXISTS (
            SELECT 1
            FROM change_log
            WHERE id = NEW.last_change_id
                AND entity_id = NEW.id
                AND operation = 'create'
        )
        THEN RAISE(ABORT, 'entity creation requires a matching change')
    END;
END;

CREATE TRIGGER validate_entity_update
BEFORE UPDATE ON entities
FOR EACH ROW
BEGIN
    SELECT CASE
        WHEN NEW.id != OLD.id
            OR NEW.kind != OLD.kind
            OR NEW.created_at != OLD.created_at
        THEN RAISE(ABORT, 'entity identity is immutable')
    END;

    SELECT CASE
        WHEN OLD.deleted_at IS NOT NULL
        THEN RAISE(ABORT, 'deleted entities are immutable')
    END;

    SELECT CASE
        WHEN NEW.updated_at < OLD.updated_at
            OR NEW.revision != OLD.revision + 1
        THEN RAISE(ABORT, 'entity revisions must advance')
    END;

    SELECT CASE
        WHEN NOT EXISTS (
            SELECT 1
            FROM change_log
            WHERE id = NEW.last_change_id
                AND entity_id = NEW.id
                AND operation = CASE
                    WHEN NEW.deleted_at IS NULL THEN 'update'
                    ELSE 'delete'
                END
        )
        THEN RAISE(ABORT, 'entity update requires a matching change')
    END;
END;

CREATE TRIGGER prevent_change_log_update
BEFORE UPDATE ON change_log
FOR EACH ROW
BEGIN
    SELECT RAISE(ABORT, 'change log entries are immutable');
END;

CREATE TRIGGER prevent_change_log_delete
BEFORE DELETE ON change_log
FOR EACH ROW
BEGIN
    SELECT RAISE(ABORT, 'change log entries are immutable');
END;
