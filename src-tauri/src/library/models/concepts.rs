use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::{CardSummary, ExplainSettings, ProblemSettings, TypeAnswerSettings};

pub const RICH_CONTENT_SCHEMA_VERSION: u32 = 1;

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
    pub active_concept_count: i64,
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
pub struct MediaSummary {
    pub id: String,
    pub mime_type: String,
    pub byte_size: i64,
    pub width: i64,
    pub height: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnswerFeedback {
    pub explanation: Value,
    pub common_mistakes: Value,
}

impl Default for AnswerFeedback {
    fn default() -> Self {
        Self {
            explanation: empty_rich_text_document(),
            common_mistakes: empty_rich_text_document(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConceptContent {
    pub schema_version: u32,
    pub prompt: Value,
    pub answer: Value,
    #[serde(default)]
    pub feedback: AnswerFeedback,
}

impl Default for ConceptContent {
    fn default() -> Self {
        Self {
            schema_version: RICH_CONTENT_SCHEMA_VERSION,
            prompt: empty_rich_text_document(),
            answer: empty_rich_text_document(),
            feedback: AnswerFeedback::default(),
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
pub struct LibraryPage {
    pub concepts: Vec<ConceptSummary>,
    pub archived_count: i64,
    pub concept_count: i64,
    pub total_count: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct LibraryQuery {
    pub query: String,
    pub include_archived: bool,
    pub deck_id: Option<String>,
    pub tag_id: Option<String>,
    pub page: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryOrganizations {
    pub decks: Vec<OrganizationSummary>,
    pub tags: Vec<OrganizationSummary>,
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
    pub problem: Option<ProblemSettings>,
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
    pub problem: Option<ProblemSettings>,
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
