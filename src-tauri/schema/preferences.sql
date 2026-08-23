CREATE TABLE device_preferences (
    singleton INTEGER PRIMARY KEY NOT NULL CHECK (singleton = 1),
    grading_mode TEXT NOT NULL CHECK (
        grading_mode IN ('simple', 'advanced')
    ),
    startup_destination TEXT NOT NULL CHECK (
        startup_destination IN ('study', 'library')
    )
) STRICT;

CREATE TRIGGER validate_device_preferences_update
BEFORE UPDATE ON device_preferences
FOR EACH ROW
WHEN NEW.singleton != OLD.singleton
BEGIN
    SELECT RAISE(ABORT, 'device preference identity is immutable');
END;

CREATE TRIGGER prevent_device_preferences_delete
BEFORE DELETE ON device_preferences
FOR EACH ROW
BEGIN
    SELECT RAISE(ABORT, 'device preferences are required');
END;

INSERT INTO device_preferences (
    singleton,
    grading_mode,
    startup_destination
)
VALUES (1, 'simple', 'study');
