CREATE TABLE authoring_drafts (
    kind TEXT NOT NULL CHECK (kind IN ('concept', 'template')),
    target_key TEXT NOT NULL CHECK (
        target_key = 'new'
        OR length(target_key) = 36
    ),
    target_id TEXT CHECK (
        target_id IS NULL
        OR (length(target_id) = 36 AND target_id = target_key)
    ),
    schema_version INTEGER NOT NULL CHECK (schema_version = 1),
    base_change_id TEXT CHECK (
        base_change_id IS NULL
        OR length(base_change_id) = 36
    ),
    payload_json TEXT NOT NULL CHECK (
        json_valid(payload_json)
        AND json_type(payload_json) = 'object'
        AND length(CAST(payload_json AS BLOB)) BETWEEN 2 AND 5000000
    ),
    created_at INTEGER NOT NULL CHECK (created_at >= 0),
    updated_at INTEGER NOT NULL CHECK (updated_at >= created_at),
    PRIMARY KEY (kind, target_key),
    CHECK (
        (target_key = 'new' AND target_id IS NULL AND base_change_id IS NULL)
        OR (
            target_key != 'new'
            AND target_id = target_key
            AND base_change_id IS NOT NULL
        )
    )
) STRICT;

CREATE TABLE authoring_draft_media (
    kind TEXT NOT NULL,
    target_key TEXT NOT NULL,
    media_id TEXT NOT NULL,
    PRIMARY KEY (kind, target_key, media_id),
    FOREIGN KEY (kind, target_key)
        REFERENCES authoring_drafts(kind, target_key)
        ON DELETE CASCADE,
    FOREIGN KEY (media_id) REFERENCES media(entity_id)
) STRICT;

CREATE TABLE device_media_cleanup (
    digest TEXT PRIMARY KEY NOT NULL CHECK (length(digest) = 64),
    file_extension TEXT NOT NULL CHECK (
        file_extension IN ('gif', 'jpg', 'png', 'webp')
    )
) STRICT;

CREATE INDEX authoring_draft_target_idx
    ON authoring_drafts(target_id);

CREATE INDEX authoring_draft_media_id_idx
    ON authoring_draft_media(media_id);

CREATE TRIGGER validate_authoring_draft_update
BEFORE UPDATE ON authoring_drafts
FOR EACH ROW
WHEN NEW.kind != OLD.kind
    OR NEW.target_key != OLD.target_key
    OR NEW.target_id IS NOT OLD.target_id
    OR NEW.created_at != OLD.created_at
BEGIN
    SELECT RAISE(ABORT, 'authoring draft identity is immutable');
END;
