CREATE TABLE card_quality_concerns (
    entity_id TEXT PRIMARY KEY NOT NULL,
    card_id TEXT NOT NULL,
    source TEXT NOT NULL CHECK (
        source IN ('manual', 'review_pattern')
    ),
    kind TEXT NOT NULL CHECK (
        kind IN (
            'ambiguous',
            'incorrect',
            'outdated',
            'too_easy',
            'too_difficult'
        )
    ),
    status TEXT NOT NULL CHECK (
        status IN ('open', 'resolved', 'dismissed')
    ),
    note TEXT NOT NULL DEFAULT '' CHECK (
        length(note) <= 500
        AND instr(note, char(0)) = 0
    ),
    observed_through_review_id TEXT,
    status_changed_at INTEGER CHECK (
        status_changed_at IS NULL
        OR status_changed_at >= 0
    ),
    last_change_id TEXT NOT NULL UNIQUE,
    CHECK (
        (
            source = 'manual'
            AND observed_through_review_id IS NULL
            AND (
                (status = 'open' AND status_changed_at IS NULL)
                OR (status != 'open' AND status_changed_at IS NOT NULL)
            )
        )
        OR (
            source = 'review_pattern'
            AND kind = 'too_difficult'
            AND status = 'dismissed'
            AND note = ''
            AND observed_through_review_id IS NOT NULL
            AND status_changed_at IS NOT NULL
        )
    ),
    FOREIGN KEY (entity_id) REFERENCES entities(id),
    FOREIGN KEY (card_id) REFERENCES cards(entity_id),
    FOREIGN KEY (observed_through_review_id) REFERENCES reviews(entity_id),
    FOREIGN KEY (last_change_id) REFERENCES change_log(id)
) STRICT;

CREATE INDEX card_quality_concerns_card_status_idx
    ON card_quality_concerns(card_id, status, source, kind);

CREATE INDEX card_quality_concerns_review_pattern_idx
    ON card_quality_concerns(
        card_id,
        source,
        status,
        observed_through_review_id
    );

CREATE TRIGGER validate_card_quality_concern_insert
BEFORE INSERT ON card_quality_concerns
FOR EACH ROW
WHEN NOT EXISTS (
    SELECT 1
    FROM entities
    WHERE id = NEW.entity_id
        AND kind = 'card_quality_concern'
        AND deleted_at IS NULL
        AND last_change_id = NEW.last_change_id
)
    OR NOT EXISTS (
        SELECT 1
        FROM cards
        INNER JOIN entities
            ON entities.id = cards.entity_id
        WHERE cards.entity_id = NEW.card_id
            AND entities.deleted_at IS NULL
    )
    OR (
        NEW.observed_through_review_id IS NOT NULL
        AND NOT EXISTS (
            SELECT 1
            FROM reviews
            WHERE reviews.entity_id = NEW.observed_through_review_id
                AND reviews.card_id = NEW.card_id
        )
    )
BEGIN
    SELECT RAISE(ABORT, 'card quality concern requires matching active data');
END;

CREATE TRIGGER validate_card_quality_concern_update
BEFORE UPDATE ON card_quality_concerns
FOR EACH ROW
WHEN OLD.source != 'manual'
    OR OLD.status != 'open'
    OR NEW.entity_id != OLD.entity_id
    OR NEW.card_id != OLD.card_id
    OR NEW.source != OLD.source
    OR NEW.kind != OLD.kind
    OR NEW.note != OLD.note
    OR NEW.observed_through_review_id IS NOT OLD.observed_through_review_id
    OR NEW.status NOT IN ('resolved', 'dismissed')
    OR NEW.status_changed_at IS NULL
    OR NEW.last_change_id = OLD.last_change_id
    OR NOT EXISTS (
        SELECT 1
        FROM entities
        WHERE id = NEW.entity_id
            AND kind = 'card_quality_concern'
            AND deleted_at IS NULL
            AND last_change_id = NEW.last_change_id
    )
BEGIN
    SELECT RAISE(ABORT, 'card quality concern update requires a matching resolution');
END;

CREATE TRIGGER prevent_card_quality_concern_delete
BEFORE DELETE ON card_quality_concerns
FOR EACH ROW
BEGIN
    SELECT RAISE(ABORT, 'card quality concerns must retain their history');
END;
