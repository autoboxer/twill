use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::error::LibraryError;

pub const RICH_CONTENT_SCHEMA_VERSION: u32 = 1;
pub const TEMPLATE_SCHEMA_VERSION: u32 = 1;
pub const CSS_SNIPPET_SCHEMA_VERSION: u32 = 1;
pub const AUTHORING_DRAFT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NamedItem {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationSummary {
    pub id: String,
    pub name: String,
    pub concept_count: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConceptSummary {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub archived: bool,
    pub decks: Vec<NamedItem>,
    pub tags: Vec<NamedItem>,
    pub card_count: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CardSummary {
    pub id: String,
    pub retrieval_kind: RetrievalFormKind,
    pub explain: Option<ExplainSettings>,
    pub cloze: Option<ClozeSettings>,
    pub image_occlusion: Option<ImageOcclusionSettings>,
    pub type_answer: Option<TypeAnswerSettings>,
    pub template: Option<NamedItem>,
    pub scheduling_state: SchedulingState,
    pub due_at: i64,
    pub review_count: i64,
    pub lapse_count: i64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RetrievalFormKind {
    Recall,
    TypeAnswer,
    Explain,
    Cloze,
    ImageOcclusion,
}

impl RetrievalFormKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Recall => "recall",
            Self::TypeAnswer => "type_answer",
            Self::Explain => "explain",
            Self::Cloze => "cloze",
            Self::ImageOcclusion => "image_occlusion",
        }
    }
}

impl TryFrom<&str> for RetrievalFormKind {
    type Error = LibraryError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "recall" => Ok(Self::Recall),
            "type_answer" => Ok(Self::TypeAnswer),
            "explain" => Ok(Self::Explain),
            "cloze" => Ok(Self::Cloze),
            "image_occlusion" => Ok(Self::ImageOcclusion),
            _ => Err(LibraryError::InvalidRetrievalFormKind(value.to_owned())),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClozeSettings {
    pub group_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImageOcclusionSettings {
    pub group_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TypeAnswerSettings {
    pub accepted_answers: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ExplainFocus {
    Why,
    How,
    CauseAndEffect,
    CompareAndContrast,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExplainSettings {
    pub focus: ExplainFocus,
    pub key_points: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyTemplate {
    pub id: String,
    pub name: String,
    pub content: TemplateContent,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyCard {
    pub id: String,
    pub concept_id: String,
    pub concept_last_change_id: String,
    pub concept_title: String,
    pub content: ConceptContent,
    pub retrieval_kind: RetrievalFormKind,
    pub explain: Option<ExplainSettings>,
    pub cloze: Option<ClozeSettings>,
    pub image_occlusion: Option<ImageOcclusionSettings>,
    pub type_answer: Option<TypeAnswerSettings>,
    pub template: Option<StudyTemplate>,
    pub scheduling_state: SchedulingState,
    pub due_at: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DeferredEditTargetStatus {
    Current,
    Changed,
    Archived,
    Missing,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeferredConceptEdit {
    pub concept_id: String,
    pub concept_title: String,
    pub base_change_id: String,
    pub queued_at: i64,
    pub target_status: DeferredEditTargetStatus,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeferredEditQueue {
    pub items: Vec<DeferredConceptEdit>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct QueueDeferredEditInput {
    pub concept_id: String,
    pub base_change_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyQueue {
    pub cards: Vec<StudyCard>,
    pub media: Vec<MediaSummary>,
    pub next_due_at: Option<i64>,
    pub total_cards: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SchedulingState {
    New,
    Learning,
    Review,
    Relearning,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ReviewRating {
    Again = 1,
    Hard = 2,
    Good = 3,
    Easy = 4,
}

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
    pub appearance: AppearancePreferences,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CssSnippetContent {
    pub schema_version: u32,
    pub source: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssSnippet {
    pub id: String,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub content: CssSnippetContent,
    pub enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssSnippetCatalog {
    pub snippets: Vec<CssSnippet>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateCssSnippetInput {
    pub name: String,
    pub content: CssSnippetContent,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateCssSnippetInput {
    pub id: String,
    pub name: String,
    pub content: CssSnippetContent,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SetCssSnippetEnabledInput {
    pub id: String,
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthoringDraftKind {
    Concept,
    Template,
}

impl AuthoringDraftKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Concept => "concept",
            Self::Template => "template",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthoringDraftTargetStatus {
    Current,
    Changed,
    Missing,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoringDraftLocator {
    pub kind: AuthoringDraftKind,
    pub target_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpsertAuthoringDraftInput {
    pub kind: AuthoringDraftKind,
    pub target_id: Option<String>,
    pub schema_version: u32,
    pub base_change_id: Option<String>,
    pub payload: Value,
    #[serde(default)]
    pub media_ids: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthoringDraft {
    pub kind: AuthoringDraftKind,
    pub target_id: Option<String>,
    pub schema_version: u32,
    pub base_change_id: Option<String>,
    pub payload: Value,
    pub media_ids: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub target_status: AuthoringDraftTargetStatus,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchedulingSettings {
    pub algorithm_version: String,
    pub desired_retention: f64,
    pub maximum_interval_days: i64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TemplateField {
    Title,
    Prompt,
    Answer,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TemplateAlignment {
    Left,
    Center,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum TemplateBlock {
    Field { field: TemplateField },
    Text { text: String },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VisualTemplateSide {
    pub blocks: Vec<TemplateBlock>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VisualTemplateAppearance {
    pub alignment: TemplateAlignment,
    pub show_field_labels: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VisualTemplate {
    pub front: VisualTemplateSide,
    pub answer: VisualTemplateSide,
    pub appearance: VisualTemplateAppearance,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CustomTemplate {
    pub front_html: String,
    pub answer_html: String,
    pub css: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TemplateMode {
    Visual,
    Custom,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TemplateContent {
    pub schema_version: u32,
    pub mode: TemplateMode,
    pub visual: VisualTemplate,
    pub custom: CustomTemplate,
}

impl Default for TemplateContent {
    fn default() -> Self {
        Self {
            schema_version: TEMPLATE_SCHEMA_VERSION,
            mode: TemplateMode::Visual,
            visual: VisualTemplate {
                front: VisualTemplateSide {
                    blocks: vec![TemplateBlock::Field {
                        field: TemplateField::Prompt,
                    }],
                },
                answer: VisualTemplateSide {
                    blocks: vec![
                        TemplateBlock::Field {
                            field: TemplateField::Prompt,
                        },
                        TemplateBlock::Field {
                            field: TemplateField::Answer,
                        },
                    ],
                },
                appearance: VisualTemplateAppearance {
                    alignment: TemplateAlignment::Left,
                    show_field_labels: true,
                },
            },
            custom: CustomTemplate {
                front_html: concat!(
                    "<main class=\"card\">\n",
                    "  {{ prompt }}\n",
                    "</main>",
                )
                .to_owned(),
                answer_html: concat!(
                    "<main class=\"card\">\n",
                    "  <section class=\"prompt\">{{ prompt }}</section>\n",
                    "  <hr>\n",
                    "  <section class=\"answer\">{{ answer }}</section>\n",
                    "</main>",
                )
                .to_owned(),
                css: concat!(
                    ".card {\n",
                    "  max-width: 42rem;\n",
                    "  margin: 0 auto;\n",
                    "  color: var(--twill-text-highlighted);\n",
                    "  font-family: var(--twill-reading-font);\n",
                    "  line-height: 1.6;\n",
                    "}\n\n",
                    "hr {\n",
                    "  margin: 2rem 0;\n",
                    "  border: 0;\n",
                    "  border-top: 1px solid var(--twill-border);\n",
                    "}",
                )
                .to_owned(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateSummary {
    pub id: String,
    pub name: String,
    pub updated_at: i64,
    pub mode: TemplateMode,
    pub retrieval_form_count: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateDetail {
    pub id: String,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_change_id: String,
    pub content: TemplateContent,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateCatalog {
    pub templates: Vec<TemplateSummary>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateTemplateInput {
    pub name: String,
    pub content: TemplateContent,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateTemplateInput {
    pub id: String,
    pub name: String,
    pub content: TemplateContent,
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

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SetAppearancePreferencesInput {
    pub appearance: AppearancePreferences,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateSchedulingSettingsInput {
    pub desired_retention: f64,
    pub maximum_interval_days: i64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecordReviewInput {
    pub card_id: String,
    pub rating: ReviewRating,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReverseReviewInput {
    pub review_id: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewOutcome {
    pub review_id: String,
    pub card_id: String,
    pub rating: ReviewRating,
    pub scheduling_state: SchedulingState,
    pub reviewed_at: i64,
    pub due_at: i64,
    pub scheduled_interval_days: f64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewReversalOutcome {
    pub reversal_id: String,
    pub review_id: String,
    pub card_id: String,
    pub reversed_at: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaSummary {
    pub id: String,
    pub mime_type: String,
    pub byte_size: i64,
    pub width: i64,
    pub height: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConceptContent {
    pub schema_version: u32,
    pub prompt: Value,
    pub answer: Value,
}

impl Default for ConceptContent {
    fn default() -> Self {
        Self {
            schema_version: RICH_CONTENT_SCHEMA_VERSION,
            prompt: empty_rich_text_document(),
            answer: empty_rich_text_document(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConceptDetail {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_change_id: String,
    pub archived: bool,
    pub decks: Vec<NamedItem>,
    pub tags: Vec<NamedItem>,
    pub cards: Vec<CardSummary>,
    pub content: ConceptContent,
    pub media: Vec<MediaSummary>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibrarySnapshot {
    pub concepts: Vec<ConceptSummary>,
    pub decks: Vec<OrganizationSummary>,
    pub tags: Vec<OrganizationSummary>,
    pub archived_count: i64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateConceptInput {
    pub title: String,
    #[serde(default)]
    pub deck_ids: Vec<String>,
    #[serde(default)]
    pub tag_ids: Vec<String>,
    #[serde(default)]
    pub content: ConceptContent,
    #[serde(default = "default_include_standard_recall")]
    pub include_standard_recall: bool,
    #[serde(default)]
    pub template_ids: Vec<String>,
    #[serde(default)]
    pub explain: Option<ExplainSettings>,
    #[serde(default)]
    pub type_answer: Option<TypeAnswerSettings>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateConceptInput {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub deck_ids: Vec<String>,
    #[serde(default)]
    pub tag_ids: Vec<String>,
    #[serde(default)]
    pub content: ConceptContent,
    #[serde(default = "default_include_standard_recall")]
    pub include_standard_recall: bool,
    #[serde(default)]
    pub template_ids: Vec<String>,
    #[serde(default)]
    pub explain: Option<ExplainSettings>,
    #[serde(default)]
    pub type_answer: Option<TypeAnswerSettings>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SetConceptArchivedInput {
    pub id: String,
    pub archived: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EntityIdInput {
    pub id: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateNamedItemInput {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RenameNamedItemInput {
    pub id: String,
    pub name: String,
}

fn empty_rich_text_document() -> Value {
    json!({
        "type": "doc",
        "content": [{ "type": "paragraph" }]
    })
}

const fn default_include_standard_recall() -> bool {
    true
}
