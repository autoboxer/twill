CREATE TABLE practice_exposures (
    entity_id TEXT PRIMARY KEY NOT NULL REFERENCES entities(id),
    concept_id TEXT NOT NULL REFERENCES concepts(entity_id),
    -- A card without a recall kind records shared prompt context, not that card's hidden answer
    card_id TEXT REFERENCES cards(entity_id),
    content_version TEXT,
    recall_kind TEXT CHECK (recall_kind IN ('first', 'recent', 'delayed')),
    occurred_at INTEGER NOT NULL CHECK (occurred_at >= 0),
    last_change_id TEXT NOT NULL UNIQUE REFERENCES change_log(id),
    CHECK (recall_kind IS NULL OR (card_id IS NOT NULL AND content_version IS NOT NULL))
) STRICT;

CREATE INDEX practice_exposures_concept_time_idx
    ON practice_exposures(concept_id, occurred_at);

CREATE TRIGGER validate_practice_exposure_insert
BEFORE INSERT ON practice_exposures
FOR EACH ROW
WHEN NOT EXISTS (
    SELECT 1 FROM entities
    WHERE id = NEW.entity_id AND kind = 'practice_exposure'
        AND created_at = NEW.occurred_at AND deleted_at IS NULL
        AND last_change_id = NEW.last_change_id
)
OR (NEW.card_id IS NOT NULL AND NOT EXISTS (
    SELECT 1 FROM cards WHERE entity_id = NEW.card_id AND concept_id = NEW.concept_id
))
BEGIN
    SELECT RAISE(ABORT, 'practice exposure requires matching entities');
END;

CREATE TRIGGER prevent_practice_exposure_update
BEFORE UPDATE ON practice_exposures
BEGIN
    SELECT RAISE(ABORT, 'practice exposures are immutable');
END;

CREATE TRIGGER prevent_practice_exposure_delete
BEFORE DELETE ON practice_exposures
BEGIN
    SELECT RAISE(ABORT, 'practice exposures are immutable');
END;

CREATE TRIGGER prevent_practice_exposure_entity_update
BEFORE UPDATE ON entities
WHEN OLD.kind = 'practice_exposure'
BEGIN
    SELECT RAISE(ABORT, 'practice exposure entities are immutable');
END;
