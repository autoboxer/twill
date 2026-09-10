use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{CreateConceptInput, CreateTemplateInput};

pub const AUTHORING_DRAFT_SCHEMA_VERSION: u32 = 1;

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
    pub position: i64,
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
    pub media_session_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthoringDraft {
    pub kind: AuthoringDraftKind,
    pub target_id: Option<String>,
    pub schema_version: u32,
    pub revision: String,
    pub base_change_id: Option<String>,
    pub payload: Value,
    pub media_ids: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub target_status: AuthoringDraftTargetStatus,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoringSaveContext {
    pub target_id: Option<String>,
    pub expected_change_id: Option<String>,
    pub expected_draft_revision: Option<String>,
    #[serde(default)]
    pub save_as_copy: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FinalizeConceptInput {
    pub context: AuthoringSaveContext,
    pub concept: CreateConceptInput,
    pub deferred_edit_position: Option<i64>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FinalizeTemplateInput {
    pub context: AuthoringSaveContext,
    pub template: CreateTemplateInput,
}
