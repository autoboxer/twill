use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GradingMode {
    Simple,
    Advanced,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StartupDestination {
    Study,
    Library,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AppearanceTheme {
    Aubergine,
    Dracula,
    OneDark,
    TokyoNight,
    CatppuccinMocha,
    Nord,
    GruvboxDark,
    SolarizedDark,
    GithubLight,
    OneLight,
    CatppuccinLatte,
    GruvboxLight,
    SolarizedLight,
    RosePineDawn,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ReadingFont {
    Inter,
    SystemUi,
    IbmPlexSans,
    SourceSerif4,
    JetBrainsMono,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ReadingTextSize {
    Small,
    Medium,
    Large,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MotionPreference {
    System,
    Full,
    Reduced,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppearancePreferences {
    pub theme: AppearanceTheme,
    pub reading_font: ReadingFont,
    pub reading_text_size: ReadingTextSize,
    pub motion_preference: MotionPreference,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DevicePreferences {
    pub grading_mode: GradingMode,
    pub startup_destination: StartupDestination,
    pub pretesting_enabled: bool,
    pub mixed_practice_enabled: bool,
    pub appearance: AppearancePreferences,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SetGradingModeInput {
    pub grading_mode: GradingMode,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SetStartupDestinationInput {
    pub startup_destination: StartupDestination,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SetPretestingEnabledInput {
    pub enabled: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SetMixedPracticeEnabledInput {
    pub enabled: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SetAppearancePreferencesInput {
    pub appearance: AppearancePreferences,
}
