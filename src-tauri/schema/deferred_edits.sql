CREATE TABLE deferred_concept_edits (
    position INTEGER PRIMARY KEY AUTOINCREMENT,
    concept_id TEXT NOT NULL UNIQUE CHECK (length(concept_id) = 36),
    base_change_id TEXT NOT NULL CHECK (length(base_change_id) = 36),
    queued_at INTEGER NOT NULL CHECK (queued_at >= 0),
    FOREIGN KEY (concept_id) REFERENCES concepts(entity_id),
    FOREIGN KEY (base_change_id) REFERENCES change_log(id)
) STRICT;

CREATE INDEX deferred_concept_edits_queued_idx
    ON deferred_concept_edits(queued_at, position);
