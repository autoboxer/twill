CREATE TABLE scheduler_configurations (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(trim(id)) BETWEEN 1 AND 100),
    algorithm TEXT NOT NULL CHECK (algorithm = 'fsrs'),
    algorithm_version TEXT NOT NULL CHECK (length(trim(algorithm_version)) > 0),
    parameters_json TEXT NOT NULL CHECK (
        json_valid(parameters_json)
        AND json_type(parameters_json) = 'array'
        AND json_array_length(parameters_json) = 21
    ),
    desired_retention REAL NOT NULL CHECK (
        desired_retention > 0.0
        AND desired_retention < 1.0
    ),
    maximum_interval_days INTEGER NOT NULL DEFAULT 36500 CHECK (
        maximum_interval_days BETWEEN 1 AND 36500
    ),
    created_at INTEGER NOT NULL CHECK (created_at >= 0)
) STRICT;

CREATE TABLE active_scheduler_configuration (
    singleton INTEGER PRIMARY KEY NOT NULL CHECK (singleton = 1),
    configuration_id TEXT NOT NULL,
    FOREIGN KEY (configuration_id) REFERENCES scheduler_configurations(id)
) STRICT;

CREATE TABLE reviews (
    entity_id TEXT PRIMARY KEY NOT NULL,
    card_id TEXT NOT NULL,
    rating INTEGER NOT NULL CHECK (rating BETWEEN 1 AND 4),
    reviewed_at INTEGER NOT NULL CHECK (reviewed_at >= 0),
    elapsed_days INTEGER NOT NULL CHECK (elapsed_days >= 0),
    scheduled_interval_days REAL NOT NULL CHECK (scheduled_interval_days > 0.0),
    state_before TEXT NOT NULL CHECK (
        state_before IN ('new', 'learning', 'review', 'relearning')
    ),
    state_after TEXT NOT NULL CHECK (
        state_after IN ('learning', 'review', 'relearning')
    ),
    stability REAL NOT NULL CHECK (stability > 0.0),
    difficulty REAL NOT NULL CHECK (difficulty > 0.0),
    due_at INTEGER NOT NULL CHECK (due_at > reviewed_at),
    scheduler_configuration_id TEXT NOT NULL,
    last_change_id TEXT NOT NULL UNIQUE,
    FOREIGN KEY (entity_id) REFERENCES entities(id),
    FOREIGN KEY (card_id) REFERENCES cards(entity_id),
    FOREIGN KEY (scheduler_configuration_id) REFERENCES scheduler_configurations(id),
    FOREIGN KEY (last_change_id) REFERENCES change_log(id)
) STRICT;

CREATE TABLE card_scheduling (
    card_id TEXT PRIMARY KEY NOT NULL,
    state TEXT NOT NULL CHECK (
        state IN ('new', 'learning', 'review', 'relearning')
    ),
    due_at INTEGER NOT NULL CHECK (due_at >= 0),
    stability REAL CHECK (stability IS NULL OR stability > 0.0),
    difficulty REAL CHECK (difficulty IS NULL OR difficulty > 0.0),
    last_reviewed_at INTEGER CHECK (last_reviewed_at IS NULL OR last_reviewed_at >= 0),
    last_review_id TEXT UNIQUE,
    review_count INTEGER NOT NULL CHECK (review_count >= 0),
    lapse_count INTEGER NOT NULL CHECK (
        lapse_count >= 0
        AND lapse_count <= review_count
    ),
    CHECK (
        (stability IS NULL AND difficulty IS NULL)
        OR (stability IS NOT NULL AND difficulty IS NOT NULL)
    ),
    CHECK (
        (review_count = 0
            AND state = 'new'
            AND stability IS NULL
            AND last_reviewed_at IS NULL
            AND last_review_id IS NULL)
        OR (review_count > 0
            AND state != 'new'
            AND stability IS NOT NULL
            AND last_reviewed_at IS NOT NULL
            AND last_review_id IS NOT NULL)
    ),
    FOREIGN KEY (card_id) REFERENCES cards(entity_id),
    FOREIGN KEY (last_review_id) REFERENCES reviews(entity_id)
) STRICT;

CREATE INDEX reviews_card_reviewed_idx
    ON reviews(card_id, reviewed_at, entity_id);

CREATE INDEX card_scheduling_due_idx
    ON card_scheduling(due_at, card_id);

CREATE TRIGGER prevent_scheduler_configuration_update
BEFORE UPDATE ON scheduler_configurations
FOR EACH ROW
BEGIN
    SELECT RAISE(ABORT, 'scheduler configurations are immutable');
END;

CREATE TRIGGER prevent_scheduler_configuration_delete
BEFORE DELETE ON scheduler_configurations
FOR EACH ROW
BEGIN
    SELECT RAISE(ABORT, 'scheduler configurations are immutable');
END;

CREATE TRIGGER prevent_active_scheduler_configuration_delete
BEFORE DELETE ON active_scheduler_configuration
FOR EACH ROW
BEGIN
    SELECT RAISE(ABORT, 'an active scheduler configuration is required');
END;

CREATE TRIGGER validate_review_insert
BEFORE INSERT ON reviews
FOR EACH ROW
WHEN NOT EXISTS (
    SELECT 1
    FROM entities
    WHERE id = NEW.entity_id
        AND kind = 'review'
        AND deleted_at IS NULL
        AND created_at = NEW.reviewed_at
        AND last_change_id = NEW.last_change_id
)
BEGIN
    SELECT RAISE(ABORT, 'review requires a matching active entity');
END;

CREATE TRIGGER prevent_review_update
BEFORE UPDATE ON reviews
FOR EACH ROW
BEGIN
    SELECT RAISE(ABORT, 'reviews are immutable');
END;

CREATE TRIGGER prevent_review_delete
BEFORE DELETE ON reviews
FOR EACH ROW
BEGIN
    SELECT RAISE(ABORT, 'reviews are immutable');
END;

CREATE TRIGGER prevent_review_entity_update
BEFORE UPDATE ON entities
FOR EACH ROW
WHEN OLD.kind = 'review'
BEGIN
    SELECT RAISE(ABORT, 'review entities are immutable');
END;

CREATE TRIGGER validate_card_scheduling_insert
BEFORE INSERT ON card_scheduling
FOR EACH ROW
WHEN NOT EXISTS (
    SELECT 1
    FROM entities
    WHERE id = NEW.card_id
        AND kind = 'card'
        AND deleted_at IS NULL
)
BEGIN
    SELECT RAISE(ABORT, 'card scheduling requires a matching active card');
END;

CREATE TRIGGER validate_card_scheduling_update
BEFORE UPDATE ON card_scheduling
FOR EACH ROW
WHEN NEW.card_id != OLD.card_id
    OR NEW.review_count != OLD.review_count + 1
    OR NEW.lapse_count != OLD.lapse_count + CASE
        WHEN OLD.state = 'review'
            AND (
                SELECT rating
                FROM reviews
                WHERE entity_id = NEW.last_review_id
            ) = 1
        THEN 1
        ELSE 0
    END
    OR NEW.last_reviewed_at < OLD.last_reviewed_at
    OR NOT EXISTS (
        SELECT 1
        FROM reviews
        WHERE reviews.entity_id = NEW.last_review_id
            AND reviews.card_id = NEW.card_id
            AND reviews.reviewed_at = NEW.last_reviewed_at
            AND reviews.state_before = OLD.state
            AND reviews.state_after = NEW.state
            AND reviews.stability = NEW.stability
            AND reviews.difficulty = NEW.difficulty
            AND reviews.due_at = NEW.due_at
    )
BEGIN
    SELECT RAISE(ABORT, 'card scheduling must advance from a matching review');
END;

CREATE TRIGGER prevent_card_scheduling_delete
BEFORE DELETE ON card_scheduling
FOR EACH ROW
BEGIN
    SELECT RAISE(ABORT, 'card scheduling is retained with its card');
END;

INSERT INTO scheduler_configurations (
    id,
    algorithm,
    algorithm_version,
    parameters_json,
    desired_retention,
    maximum_interval_days,
    created_at
) VALUES (
    'fsrs-6.6.1-default-0.90',
    'fsrs',
    '6.6.1',
    '[0.212,1.2931,2.3065,8.2956,6.4133,0.8334,3.0194,0.001,1.8722,0.1666,0.796,1.4835,0.0614,0.2629,1.6483,0.6014,1.8729,0.5425,0.0912,0.0658,0.1542]',
    0.90,
    36500,
    0
);

INSERT INTO active_scheduler_configuration (singleton, configuration_id)
VALUES (1, 'fsrs-6.6.1-default-0.90');
