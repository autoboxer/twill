CREATE TABLE concepts (
    entity_id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL CHECK (
        length(trim(title)) BETWEEN 1 AND 200
    ),
    content_json TEXT NOT NULL
        DEFAULT '{"schemaVersion":1,"prompt":{"type":"doc","content":[{"type":"paragraph"}]},"answer":{"type":"doc","content":[{"type":"paragraph"}]}}'
        CHECK (
            json_valid(content_json)
            AND json_type(content_json) = 'object'
        ),
    archived_at INTEGER CHECK (archived_at IS NULL OR archived_at >= 0),
    last_change_id TEXT NOT NULL,
    FOREIGN KEY (entity_id) REFERENCES entities(id),
    FOREIGN KEY (last_change_id) REFERENCES change_log(id)
) STRICT;

CREATE TABLE decks (
    entity_id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL CHECK (
        length(trim(name)) BETWEEN 1 AND 80
    ),
    last_change_id TEXT NOT NULL,
    FOREIGN KEY (entity_id) REFERENCES entities(id),
    FOREIGN KEY (last_change_id) REFERENCES change_log(id)
) STRICT;

CREATE TABLE tags (
    entity_id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL CHECK (
        length(trim(name)) BETWEEN 1 AND 80
    ),
    last_change_id TEXT NOT NULL,
    FOREIGN KEY (entity_id) REFERENCES entities(id),
    FOREIGN KEY (last_change_id) REFERENCES change_log(id)
) STRICT;

CREATE TABLE templates (
    entity_id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL CHECK (
        length(trim(name)) BETWEEN 1 AND 80
    ),
    content_json TEXT NOT NULL CHECK (
        json_valid(content_json)
        AND json_type(content_json) = 'object'
    ),
    last_change_id TEXT NOT NULL,
    FOREIGN KEY (entity_id) REFERENCES entities(id),
    FOREIGN KEY (last_change_id) REFERENCES change_log(id)
) STRICT;

CREATE TABLE cards (
    entity_id TEXT PRIMARY KEY NOT NULL,
    concept_id TEXT NOT NULL,
    retrieval_kind TEXT NOT NULL CHECK (
        retrieval_kind IN ('recall', 'type_answer')
    ),
    configuration_json TEXT NOT NULL DEFAULT '{}' CHECK (
        json_valid(configuration_json)
        AND json_type(configuration_json) = 'object'
    ),
    template_id TEXT,
    last_change_id TEXT NOT NULL,
    CHECK (retrieval_kind = 'recall' OR template_id IS NULL),
    FOREIGN KEY (entity_id) REFERENCES entities(id),
    FOREIGN KEY (concept_id) REFERENCES concepts(entity_id),
    FOREIGN KEY (template_id) REFERENCES templates(entity_id),
    FOREIGN KEY (last_change_id) REFERENCES change_log(id)
) STRICT;

CREATE TABLE concept_decks (
    concept_id TEXT NOT NULL,
    deck_id TEXT NOT NULL,
    created_at INTEGER NOT NULL CHECK (created_at >= 0),
    updated_at INTEGER NOT NULL CHECK (updated_at >= created_at),
    removed_at INTEGER CHECK (
        removed_at IS NULL
        OR (removed_at >= created_at AND removed_at <= updated_at)
    ),
    last_change_id TEXT NOT NULL,
    PRIMARY KEY (concept_id, deck_id),
    FOREIGN KEY (concept_id) REFERENCES concepts(entity_id),
    FOREIGN KEY (deck_id) REFERENCES decks(entity_id),
    FOREIGN KEY (last_change_id) REFERENCES change_log(id)
) STRICT;

CREATE TABLE concept_tags (
    concept_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    created_at INTEGER NOT NULL CHECK (created_at >= 0),
    updated_at INTEGER NOT NULL CHECK (updated_at >= created_at),
    removed_at INTEGER CHECK (
        removed_at IS NULL
        OR (removed_at >= created_at AND removed_at <= updated_at)
    ),
    last_change_id TEXT NOT NULL,
    PRIMARY KEY (concept_id, tag_id),
    FOREIGN KEY (concept_id) REFERENCES concepts(entity_id),
    FOREIGN KEY (tag_id) REFERENCES tags(entity_id),
    FOREIGN KEY (last_change_id) REFERENCES change_log(id)
) STRICT;

CREATE INDEX cards_concept_idx
    ON cards(concept_id);

CREATE INDEX cards_template_idx
    ON cards(template_id);

CREATE INDEX concept_decks_deck_active_idx
    ON concept_decks(deck_id, removed_at);

CREATE INDEX concept_tags_tag_active_idx
    ON concept_tags(tag_id, removed_at);

CREATE TRIGGER validate_concept_insert
BEFORE INSERT ON concepts
FOR EACH ROW
WHEN NOT EXISTS (
    SELECT 1
    FROM entities
    WHERE id = NEW.entity_id
        AND kind = 'concept'
        AND deleted_at IS NULL
        AND last_change_id = NEW.last_change_id
)
BEGIN
    SELECT RAISE(ABORT, 'concept requires a matching active entity');
END;

CREATE TRIGGER validate_concept_update
BEFORE UPDATE ON concepts
FOR EACH ROW
WHEN NEW.entity_id != OLD.entity_id
    OR NEW.last_change_id = OLD.last_change_id
    OR NOT EXISTS (
        SELECT 1
        FROM entities
        WHERE id = NEW.entity_id
            AND kind = 'concept'
            AND deleted_at IS NULL
            AND last_change_id = NEW.last_change_id
    )
BEGIN
    SELECT RAISE(ABORT, 'concept update requires a matching entity change');
END;

CREATE TRIGGER prevent_concept_delete
BEFORE DELETE ON concepts
FOR EACH ROW
BEGIN
    SELECT RAISE(ABORT, 'concepts must be deleted with an entity tombstone');
END;

CREATE TRIGGER validate_deck_insert
BEFORE INSERT ON decks
FOR EACH ROW
WHEN NOT EXISTS (
    SELECT 1
    FROM entities
    WHERE id = NEW.entity_id
        AND kind = 'deck'
        AND deleted_at IS NULL
        AND last_change_id = NEW.last_change_id
)
BEGIN
    SELECT RAISE(ABORT, 'deck requires a matching active entity');
END;

CREATE TRIGGER validate_deck_update
BEFORE UPDATE ON decks
FOR EACH ROW
WHEN NEW.entity_id != OLD.entity_id
    OR NEW.last_change_id = OLD.last_change_id
    OR NOT EXISTS (
        SELECT 1
        FROM entities
        WHERE id = NEW.entity_id
            AND kind = 'deck'
            AND deleted_at IS NULL
            AND last_change_id = NEW.last_change_id
    )
BEGIN
    SELECT RAISE(ABORT, 'deck update requires a matching entity change');
END;

CREATE TRIGGER prevent_deck_delete
BEFORE DELETE ON decks
FOR EACH ROW
BEGIN
    SELECT RAISE(ABORT, 'decks must be deleted with an entity tombstone');
END;

CREATE TRIGGER validate_tag_insert
BEFORE INSERT ON tags
FOR EACH ROW
WHEN NOT EXISTS (
    SELECT 1
    FROM entities
    WHERE id = NEW.entity_id
        AND kind = 'tag'
        AND deleted_at IS NULL
        AND last_change_id = NEW.last_change_id
)
BEGIN
    SELECT RAISE(ABORT, 'tag requires a matching active entity');
END;

CREATE TRIGGER validate_tag_update
BEFORE UPDATE ON tags
FOR EACH ROW
WHEN NEW.entity_id != OLD.entity_id
    OR NEW.last_change_id = OLD.last_change_id
    OR NOT EXISTS (
        SELECT 1
        FROM entities
        WHERE id = NEW.entity_id
            AND kind = 'tag'
            AND deleted_at IS NULL
            AND last_change_id = NEW.last_change_id
    )
BEGIN
    SELECT RAISE(ABORT, 'tag update requires a matching entity change');
END;

CREATE TRIGGER prevent_tag_delete
BEFORE DELETE ON tags
FOR EACH ROW
BEGIN
    SELECT RAISE(ABORT, 'tags must be deleted with an entity tombstone');
END;

CREATE TRIGGER validate_template_insert
BEFORE INSERT ON templates
FOR EACH ROW
WHEN NOT EXISTS (
    SELECT 1
    FROM entities
    WHERE id = NEW.entity_id
        AND kind = 'template'
        AND deleted_at IS NULL
        AND last_change_id = NEW.last_change_id
)
BEGIN
    SELECT RAISE(ABORT, 'template requires a matching active entity');
END;

CREATE TRIGGER validate_template_update
BEFORE UPDATE ON templates
FOR EACH ROW
WHEN NEW.entity_id != OLD.entity_id
    OR NEW.last_change_id = OLD.last_change_id
    OR NOT EXISTS (
        SELECT 1
        FROM entities
        WHERE id = NEW.entity_id
            AND kind = 'template'
            AND deleted_at IS NULL
            AND last_change_id = NEW.last_change_id
    )
BEGIN
    SELECT RAISE(ABORT, 'template update requires a matching entity change');
END;

CREATE TRIGGER prevent_template_delete
BEFORE DELETE ON templates
FOR EACH ROW
BEGIN
    SELECT RAISE(ABORT, 'templates must be deleted with an entity tombstone');
END;

CREATE TRIGGER validate_card_insert
BEFORE INSERT ON cards
FOR EACH ROW
WHEN NOT EXISTS (
    SELECT 1
    FROM entities
    WHERE id = NEW.entity_id
        AND kind = 'card'
        AND deleted_at IS NULL
        AND last_change_id = NEW.last_change_id
)
    OR NOT EXISTS (
        SELECT 1
        FROM entities
        WHERE id = NEW.concept_id
            AND kind = 'concept'
            AND deleted_at IS NULL
    )
    OR (
        NEW.template_id IS NOT NULL
        AND NOT EXISTS (
            SELECT 1
            FROM entities
            WHERE id = NEW.template_id
                AND kind = 'template'
                AND deleted_at IS NULL
        )
    )
    OR EXISTS (
        SELECT 1
        FROM cards AS existing_cards
        INNER JOIN entities AS existing_entities
            ON existing_entities.id = existing_cards.entity_id
        WHERE existing_cards.concept_id = NEW.concept_id
            AND existing_cards.retrieval_kind = NEW.retrieval_kind
            AND existing_cards.template_id IS NEW.template_id
            AND existing_entities.deleted_at IS NULL
    )
BEGIN
    SELECT RAISE(ABORT, 'card requires a unique active retrieval form');
END;

CREATE TRIGGER validate_card_update
BEFORE UPDATE ON cards
FOR EACH ROW
WHEN NEW.entity_id != OLD.entity_id
    OR NEW.concept_id != OLD.concept_id
    OR NEW.retrieval_kind != OLD.retrieval_kind
    OR NEW.template_id IS NOT OLD.template_id
    OR NEW.last_change_id = OLD.last_change_id
    OR NOT EXISTS (
        SELECT 1
        FROM entities
        WHERE id = NEW.entity_id
            AND kind = 'card'
            AND deleted_at IS NULL
            AND last_change_id = NEW.last_change_id
    )
BEGIN
    SELECT RAISE(ABORT, 'card update requires a matching entity change');
END;

CREATE TRIGGER prevent_card_delete
BEFORE DELETE ON cards
FOR EACH ROW
BEGIN
    SELECT RAISE(ABORT, 'cards must be deleted with an entity tombstone');
END;

CREATE TRIGGER prevent_active_template_delete
BEFORE UPDATE OF deleted_at ON entities
FOR EACH ROW
WHEN OLD.kind = 'template'
    AND OLD.deleted_at IS NULL
    AND NEW.deleted_at IS NOT NULL
    AND EXISTS (
        SELECT 1
        FROM cards
        INNER JOIN entities AS card_entities
            ON card_entities.id = cards.entity_id
        WHERE cards.template_id = OLD.id
            AND card_entities.deleted_at IS NULL
    )
BEGIN
    SELECT RAISE(ABORT, 'templates in use by active cards cannot be deleted');
END;

CREATE TRIGGER validate_concept_deck_insert
BEFORE INSERT ON concept_decks
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
    SELECT RAISE(ABORT, 'deck assignment requires a matching concept change');
END;

CREATE TRIGGER validate_concept_deck_update
BEFORE UPDATE ON concept_decks
FOR EACH ROW
WHEN NEW.concept_id != OLD.concept_id
    OR NEW.deck_id != OLD.deck_id
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
    SELECT RAISE(ABORT, 'deck assignment update requires a matching concept change');
END;

CREATE TRIGGER prevent_concept_deck_delete
BEFORE DELETE ON concept_decks
FOR EACH ROW
BEGIN
    SELECT RAISE(ABORT, 'deck assignments must retain removal markers');
END;

CREATE TRIGGER validate_concept_tag_insert
BEFORE INSERT ON concept_tags
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
    SELECT RAISE(ABORT, 'tag assignment requires a matching concept change');
END;

CREATE TRIGGER validate_concept_tag_update
BEFORE UPDATE ON concept_tags
FOR EACH ROW
WHEN NEW.concept_id != OLD.concept_id
    OR NEW.tag_id != OLD.tag_id
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
    SELECT RAISE(ABORT, 'tag assignment update requires a matching concept change');
END;

CREATE TRIGGER prevent_concept_tag_delete
BEFORE DELETE ON concept_tags
FOR EACH ROW
BEGIN
    SELECT RAISE(ABORT, 'tag assignments must retain removal markers');
END;
