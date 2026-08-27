CREATE TABLE pretests (
    entity_id TEXT PRIMARY KEY NOT NULL,
    concept_id TEXT NOT NULL,
    card_id TEXT NOT NULL,
    outcome TEXT NOT NULL CHECK (outcome IN ('attempted', 'skipped')),
    occurred_at INTEGER NOT NULL CHECK (occurred_at >= 0),
    last_change_id TEXT NOT NULL UNIQUE,
    FOREIGN KEY (entity_id) REFERENCES entities(id),
    FOREIGN KEY (concept_id) REFERENCES concepts(entity_id),
    FOREIGN KEY (card_id) REFERENCES cards(entity_id),
    FOREIGN KEY (last_change_id) REFERENCES change_log(id)
) STRICT;

CREATE INDEX pretests_concept_occurred_idx
    ON pretests(concept_id, occurred_at, entity_id);

CREATE INDEX pretests_card_occurred_idx
    ON pretests(card_id, occurred_at, entity_id);

CREATE TRIGGER validate_pretest_insert
BEFORE INSERT ON pretests
FOR EACH ROW
WHEN NOT EXISTS (
        SELECT 1
        FROM entities
        WHERE id = NEW.entity_id
            AND kind = 'pretest'
            AND deleted_at IS NULL
            AND created_at = NEW.occurred_at
            AND last_change_id = NEW.last_change_id
    )
    OR NOT EXISTS (
        SELECT 1
        FROM cards
        WHERE entity_id = NEW.card_id
            AND concept_id = NEW.concept_id
    )
BEGIN
    SELECT RAISE(ABORT, 'pretest requires a matching entity and retrieval form');
END;

CREATE TRIGGER prevent_pretest_update
BEFORE UPDATE ON pretests
FOR EACH ROW
BEGIN
    SELECT RAISE(ABORT, 'pretests are immutable');
END;

CREATE TRIGGER prevent_pretest_delete
BEFORE DELETE ON pretests
FOR EACH ROW
BEGIN
    SELECT RAISE(ABORT, 'pretests are immutable');
END;

CREATE TRIGGER prevent_pretest_entity_update
BEFORE UPDATE ON entities
FOR EACH ROW
WHEN OLD.kind = 'pretest'
BEGIN
    SELECT RAISE(ABORT, 'pretest history entities are immutable');
END;
