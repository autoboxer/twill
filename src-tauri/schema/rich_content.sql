CREATE TABLE media (
    entity_id TEXT PRIMARY KEY NOT NULL,
    digest TEXT NOT NULL UNIQUE CHECK (length(digest) = 64),
    mime_type TEXT NOT NULL CHECK (
        mime_type IN ('image/gif', 'image/jpeg', 'image/png', 'image/webp')
    ),
    file_extension TEXT NOT NULL CHECK (
        file_extension IN ('gif', 'jpg', 'png', 'webp')
    ),
    byte_size INTEGER NOT NULL CHECK (byte_size BETWEEN 1 AND 20971520),
    width INTEGER NOT NULL CHECK (width > 0),
    height INTEGER NOT NULL CHECK (height > 0),
    last_change_id TEXT NOT NULL,
    FOREIGN KEY (entity_id) REFERENCES entities(id),
    FOREIGN KEY (last_change_id) REFERENCES change_log(id)
) STRICT;

CREATE TABLE concept_media (
    concept_id TEXT NOT NULL,
    media_id TEXT NOT NULL,
    created_at INTEGER NOT NULL CHECK (created_at >= 0),
    updated_at INTEGER NOT NULL CHECK (updated_at >= created_at),
    removed_at INTEGER CHECK (
        removed_at IS NULL
        OR (removed_at >= created_at AND removed_at <= updated_at)
    ),
    last_change_id TEXT NOT NULL,
    PRIMARY KEY (concept_id, media_id),
    FOREIGN KEY (concept_id) REFERENCES concepts(entity_id),
    FOREIGN KEY (media_id) REFERENCES media(entity_id),
    FOREIGN KEY (last_change_id) REFERENCES change_log(id)
) STRICT;

CREATE INDEX concept_media_media_active_idx
    ON concept_media(media_id, removed_at);

CREATE TRIGGER validate_media_insert
BEFORE INSERT ON media
FOR EACH ROW
WHEN NOT EXISTS (
    SELECT 1
    FROM entities
    WHERE id = NEW.entity_id
        AND kind = 'media'
        AND deleted_at IS NULL
        AND last_change_id = NEW.last_change_id
)
BEGIN
    SELECT RAISE(ABORT, 'media requires a matching active entity');
END;

CREATE TRIGGER prevent_media_update
BEFORE UPDATE ON media
FOR EACH ROW
BEGIN
    SELECT RAISE(ABORT, 'media metadata is immutable');
END;

CREATE TRIGGER prevent_media_delete
BEFORE DELETE ON media
FOR EACH ROW
BEGIN
    SELECT RAISE(ABORT, 'media must be deleted with an entity tombstone');
END;

CREATE TRIGGER validate_concept_media_insert
BEFORE INSERT ON concept_media
FOR EACH ROW
WHEN NOT EXISTS (
    SELECT 1
    FROM entities
    WHERE id = NEW.concept_id
        AND kind = 'concept'
        AND deleted_at IS NULL
        AND last_change_id = NEW.last_change_id
)
BEGIN
    SELECT RAISE(ABORT, 'media assignment requires a matching concept change');
END;

CREATE TRIGGER validate_concept_media_update
BEFORE UPDATE ON concept_media
FOR EACH ROW
WHEN NEW.concept_id != OLD.concept_id
    OR NEW.media_id != OLD.media_id
    OR NEW.created_at != OLD.created_at
    OR NEW.last_change_id = OLD.last_change_id
    OR NOT EXISTS (
        SELECT 1
        FROM entities
        WHERE id = NEW.concept_id
            AND kind = 'concept'
            AND deleted_at IS NULL
            AND last_change_id = NEW.last_change_id
    )
BEGIN
    SELECT RAISE(ABORT, 'media assignment update requires a matching concept change');
END;

CREATE TRIGGER prevent_concept_media_delete
BEFORE DELETE ON concept_media
FOR EACH ROW
BEGIN
    SELECT RAISE(ABORT, 'media assignments must retain removal markers');
END;
