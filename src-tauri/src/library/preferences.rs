use rusqlite::Connection;

use crate::data::WriteTransaction;
use crate::library::{
    AppearancePreferences, AppearanceTheme, DevicePreferences, GradingMode, LibraryError,
    LibraryResult, MotionPreference, ReadingFont, ReadingTextSize, StartupDestination,
};

pub fn query_device_preferences(
    connection: &Connection,
) -> LibraryResult<DevicePreferences> {
    let stored = connection.query_row(
        "SELECT
            grading_mode,
            startup_destination,
            theme,
            reading_font,
            reading_text_size,
            motion_preference,
            pretesting_enabled,
            mixed_practice_enabled
        FROM device_preferences
        WHERE singleton = 1",
        [],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, bool>(6)?,
                row.get::<_, bool>(7)?,
            ))
        },
    )?;

    Ok(DevicePreferences {
        grading_mode: GradingMode::try_from(stored.0.as_str())?,
        startup_destination: StartupDestination::try_from(stored.1.as_str())?,
        pretesting_enabled: stored.6,
        mixed_practice_enabled: stored.7,
        appearance: AppearancePreferences {
            theme: AppearanceTheme::try_from(stored.2.as_str())?,
            reading_font: ReadingFont::try_from(stored.3.as_str())?,
            reading_text_size: ReadingTextSize::try_from(stored.4.as_str())?,
            motion_preference: MotionPreference::try_from(stored.5.as_str())?,
        },
    })
}

pub fn update_mixed_practice_enabled(
    transaction: &WriteTransaction<'_>,
    enabled: bool,
) -> LibraryResult<DevicePreferences> {
    transaction.execute(
        "UPDATE device_preferences
        SET mixed_practice_enabled = ?1
        WHERE singleton = 1
            AND mixed_practice_enabled != ?1",
        [enabled],
    )?;

    query_device_preferences(transaction)
}

pub fn update_pretesting_enabled(
    transaction: &WriteTransaction<'_>,
    enabled: bool,
) -> LibraryResult<DevicePreferences> {
    transaction.execute(
        "UPDATE device_preferences
        SET pretesting_enabled = ?1
        WHERE singleton = 1
            AND pretesting_enabled != ?1",
        [enabled],
    )?;

    query_device_preferences(transaction)
}

pub fn update_grading_mode(
    transaction: &WriteTransaction<'_>,
    grading_mode: GradingMode,
) -> LibraryResult<DevicePreferences> {
    transaction.execute(
        "UPDATE device_preferences
        SET grading_mode = ?1
        WHERE singleton = 1
            AND grading_mode != ?1",
        [grading_mode.as_str()],
    )?;

    query_device_preferences(transaction)
}

pub fn update_startup_destination(
    transaction: &WriteTransaction<'_>,
    startup_destination: StartupDestination,
) -> LibraryResult<DevicePreferences> {
    transaction.execute(
        "UPDATE device_preferences
        SET startup_destination = ?1
        WHERE singleton = 1
            AND startup_destination != ?1",
        [startup_destination.as_str()],
    )?;

    query_device_preferences(transaction)
}

pub fn update_appearance_preferences(
    transaction: &WriteTransaction<'_>,
    appearance: AppearancePreferences,
) -> LibraryResult<DevicePreferences> {
    transaction.execute(
        "UPDATE device_preferences
        SET
            theme = ?1,
            reading_font = ?2,
            reading_text_size = ?3,
            motion_preference = ?4
        WHERE singleton = 1",
        (
            appearance.theme.as_str(),
            appearance.reading_font.as_str(),
            appearance.reading_text_size.as_str(),
            appearance.motion_preference.as_str(),
        ),
    )?;

    query_device_preferences(transaction)
}

impl GradingMode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Simple => "simple",
            Self::Advanced => "advanced",
        }
    }
}

impl TryFrom<&str> for GradingMode {
    type Error = LibraryError;

    fn try_from(value: &str) -> LibraryResult<Self> {
        match value {
            "simple" => Ok(Self::Simple),
            "advanced" => Ok(Self::Advanced),
            _ => Err(LibraryError::InvalidGradingMode(value.to_owned())),
        }
    }
}

impl StartupDestination {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Study => "study",
            Self::Library => "library",
        }
    }
}

impl TryFrom<&str> for StartupDestination {
    type Error = LibraryError;

    fn try_from(value: &str) -> LibraryResult<Self> {
        match value {
            "study" => Ok(Self::Study),
            "library" => Ok(Self::Library),
            _ => Err(LibraryError::InvalidStartupDestination(value.to_owned())),
        }
    }
}

impl AppearanceTheme {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Aubergine => "aubergine",
            Self::Dracula => "dracula",
            Self::OneDark => "one-dark",
            Self::TokyoNight => "tokyo-night",
            Self::CatppuccinMocha => "catppuccin-mocha",
            Self::Nord => "nord",
            Self::GruvboxDark => "gruvbox-dark",
            Self::SolarizedDark => "solarized-dark",
            Self::GithubLight => "github-light",
            Self::OneLight => "one-light",
            Self::CatppuccinLatte => "catppuccin-latte",
            Self::GruvboxLight => "gruvbox-light",
            Self::SolarizedLight => "solarized-light",
            Self::RosePineDawn => "rose-pine-dawn",
        }
    }
}

impl TryFrom<&str> for AppearanceTheme {
    type Error = LibraryError;

    fn try_from(value: &str) -> LibraryResult<Self> {
        match value {
            "aubergine" => Ok(Self::Aubergine),
            "dracula" => Ok(Self::Dracula),
            "one-dark" => Ok(Self::OneDark),
            "tokyo-night" => Ok(Self::TokyoNight),
            "catppuccin-mocha" => Ok(Self::CatppuccinMocha),
            "nord" => Ok(Self::Nord),
            "gruvbox-dark" => Ok(Self::GruvboxDark),
            "solarized-dark" => Ok(Self::SolarizedDark),
            "github-light" => Ok(Self::GithubLight),
            "one-light" => Ok(Self::OneLight),
            "catppuccin-latte" => Ok(Self::CatppuccinLatte),
            "gruvbox-light" => Ok(Self::GruvboxLight),
            "solarized-light" => Ok(Self::SolarizedLight),
            "rose-pine-dawn" => Ok(Self::RosePineDawn),
            _ => Err(invalid_preference("appearance theme", value)),
        }
    }
}

impl ReadingFont {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Inter => "inter",
            Self::SystemUi => "system_ui",
            Self::IbmPlexSans => "ibm_plex_sans",
            Self::SourceSerif4 => "source_serif_4",
            Self::JetBrainsMono => "jetbrains_mono",
        }
    }
}

impl TryFrom<&str> for ReadingFont {
    type Error = LibraryError;

    fn try_from(value: &str) -> LibraryResult<Self> {
        match value {
            "inter" => Ok(Self::Inter),
            "system_ui" => Ok(Self::SystemUi),
            "ibm_plex_sans" => Ok(Self::IbmPlexSans),
            "source_serif_4" => Ok(Self::SourceSerif4),
            "jetbrains_mono" => Ok(Self::JetBrainsMono),
            _ => Err(invalid_preference("reading font", value)),
        }
    }
}

impl ReadingTextSize {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
        }
    }
}

impl TryFrom<&str> for ReadingTextSize {
    type Error = LibraryError;

    fn try_from(value: &str) -> LibraryResult<Self> {
        match value {
            "small" => Ok(Self::Small),
            "medium" => Ok(Self::Medium),
            "large" => Ok(Self::Large),
            _ => Err(invalid_preference("reading text size", value)),
        }
    }
}

impl MotionPreference {
    const fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Full => "full",
            Self::Reduced => "reduced",
        }
    }
}

impl TryFrom<&str> for MotionPreference {
    type Error = LibraryError;

    fn try_from(value: &str) -> LibraryResult<Self> {
        match value {
            "system" => Ok(Self::System),
            "full" => Ok(Self::Full),
            "reduced" => Ok(Self::Reduced),
            _ => Err(invalid_preference("motion", value)),
        }
    }
}

fn invalid_preference(field: &'static str, value: &str) -> LibraryError {
    LibraryError::InvalidDevicePreference {
        field,
        value: value.to_owned(),
    }
}
