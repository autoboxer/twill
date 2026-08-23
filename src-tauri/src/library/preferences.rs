use rusqlite::Connection;

use crate::data::WriteTransaction;
use crate::library::{
    DevicePreferences, GradingMode, LibraryError, LibraryResult, StartupDestination,
};

pub fn query_device_preferences(
    connection: &Connection,
) -> LibraryResult<DevicePreferences> {
    let (grading_mode, startup_destination): (String, String) = connection.query_row(
        "SELECT grading_mode, startup_destination
        FROM device_preferences
        WHERE singleton = 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    Ok(DevicePreferences {
        grading_mode: GradingMode::try_from(grading_mode.as_str())?,
        startup_destination: StartupDestination::try_from(startup_destination.as_str())?,
    })
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
