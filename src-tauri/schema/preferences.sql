CREATE TABLE device_preferences (
    singleton INTEGER PRIMARY KEY NOT NULL CHECK (singleton = 1),
    grading_mode TEXT NOT NULL CHECK (
        grading_mode IN ('simple', 'advanced')
    ),
    startup_destination TEXT NOT NULL CHECK (
        startup_destination IN ('study', 'library')
    ),
    theme TEXT NOT NULL CHECK (
        theme IN (
            'aubergine',
            'dracula',
            'one-dark',
            'tokyo-night',
            'catppuccin-mocha',
            'nord',
            'gruvbox-dark',
            'solarized-dark',
            'github-light',
            'one-light',
            'catppuccin-latte',
            'gruvbox-light',
            'solarized-light',
            'rose-pine-dawn'
        )
    ),
    reading_font TEXT NOT NULL CHECK (
        reading_font IN (
            'inter',
            'system_ui',
            'ibm_plex_sans',
            'source_serif_4',
            'jetbrains_mono'
        )
    ),
    reading_text_size TEXT NOT NULL CHECK (
        reading_text_size IN ('small', 'medium', 'large')
    ),
    motion_preference TEXT NOT NULL CHECK (
        motion_preference IN ('system', 'full', 'reduced')
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
    startup_destination,
    theme,
    reading_font,
    reading_text_size,
    motion_preference
)
VALUES (
    1,
    'simple',
    'study',
    'aubergine',
    'inter',
    'medium',
    'system'
);
