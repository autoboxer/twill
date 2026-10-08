CREATE TABLE practice_links (
    entity_id TEXT PRIMARY KEY NOT NULL,
    first_concept_id TEXT NOT NULL,
    second_concept_id TEXT NOT NULL,
    objective TEXT NOT NULL CHECK (
        length(trim(objective)) BETWEEN 1 AND 300
        AND objective = trim(objective)
    ),
    last_change_id TEXT NOT NULL,
    CHECK (first_concept_id < second_concept_id),
    FOREIGN KEY (entity_id) REFERENCES entities(id),
    FOREIGN KEY (first_concept_id) REFERENCES concepts(entity_id),
    FOREIGN KEY (second_concept_id) REFERENCES concepts(entity_id),
    FOREIGN KEY (last_change_id) REFERENCES change_log(id)
) STRICT;

CREATE INDEX practice_links_first_idx ON practice_links(first_concept_id);
CREATE INDEX practice_links_second_idx ON practice_links(second_concept_id);

CREATE TRIGGER validate_practice_link_insert
BEFORE INSERT ON practice_links
FOR EACH ROW
WHEN NOT EXISTS (
    SELECT 1 FROM entities
    WHERE id = NEW.entity_id
        AND kind = 'practice_link'
        AND deleted_at IS NULL
        AND last_change_id = NEW.last_change_id
)
    OR NOT EXISTS (
        SELECT 1 FROM entities
        WHERE id = NEW.first_concept_id AND kind = 'concept' AND deleted_at IS NULL
    )
    OR NOT EXISTS (
        SELECT 1 FROM entities
        WHERE id = NEW.second_concept_id AND kind = 'concept' AND deleted_at IS NULL
    )
    OR EXISTS (
        SELECT 1 FROM practice_links
        INNER JOIN entities ON entities.id = practice_links.entity_id
        WHERE first_concept_id = NEW.first_concept_id
            AND second_concept_id = NEW.second_concept_id
            AND entities.deleted_at IS NULL
    )
BEGIN
    SELECT RAISE(ABORT, 'practice link requires unique active concepts and a matching entity');
END;

CREATE TRIGGER validate_practice_link_update
BEFORE UPDATE ON practice_links
FOR EACH ROW
WHEN NEW.entity_id != OLD.entity_id
    OR NEW.first_concept_id != OLD.first_concept_id
    OR NEW.second_concept_id != OLD.second_concept_id
    OR NEW.last_change_id = OLD.last_change_id
    OR NOT EXISTS (
        SELECT 1 FROM entities
        WHERE id = NEW.entity_id
            AND kind = 'practice_link'
            AND deleted_at IS NULL
            AND last_change_id = NEW.last_change_id
    )
BEGIN
    SELECT RAISE(ABORT, 'practice link update requires a matching entity change');
END;

CREATE TRIGGER prevent_practice_link_delete
BEFORE DELETE ON practice_links
FOR EACH ROW
BEGIN
    SELECT RAISE(ABORT, 'practice links must be deleted with an entity tombstone');
END;
