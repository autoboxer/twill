use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::{
    ClozeSettings, ConceptContent, ExplainSettings, ImageLabelingSettings,
    ImageOcclusionSettings, MediaSummary, ProblemSettings, RetrievalFormKind,
    TemplateContent, TypeAnswerSettings,
};

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
    pub content_version: String,
    pub concept_id: String,
    pub retrieval_kind: RetrievalFormKind,
    pub explain: Option<ExplainSettings>,
    pub problem: Option<ProblemSettings>,
    pub cloze: Option<ClozeSettings>,
    pub image_occlusion: Option<ImageOcclusionSettings>,
    pub image_labeling: Option<ImageLabelingSettings>,
    pub type_answer: Option<TypeAnswerSettings>,
    pub template_id: Option<String>,
    pub scheduling_state: SchedulingState,
    pub due_at: i64,
    pub pretest_eligible: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyConcept {
    pub id: String,
    pub last_change_id: String,
    pub title: String,
    pub content: ConceptContent,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyQueue {
    pub cards: Vec<StudyCard>,
    pub concepts: Vec<StudyConcept>,
    pub templates: Vec<StudyTemplate>,
    pub media: Vec<MediaSummary>,
    pub related_concepts: BTreeMap<String, BTreeSet<String>>,
    pub next_due_at: Option<i64>,
    pub total_cards: i64,
    pub due_cards: i64,
    pub mixed_practice_enabled: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct StudyQuery {
    pub query: String,
    pub deck_id: Option<String>,
    pub tag_id: Option<String>,
    pub card_type: Option<RetrievalFormKind>,
    pub state: Option<SchedulingState>,
    pub card_limit: Option<u32>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
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

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchedulingSettings {
    pub algorithm_version: String,
    pub desired_retention: f64,
    pub maximum_interval_days: i64,
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
    #[serde(default)]
    pub assisted: Option<bool>,
    #[serde(default)]
    pub content_version: Option<String>,
    #[serde(default)]
    pub exposure_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecordExposureInput {
    pub concept_id: String,
    pub card_id: Option<String>,
    pub content_version: Option<String>,
    #[serde(default)]
    pub uncertainty_marked: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningEvidence {
    pub delayed_remembered: i64,
    pub delayed_total: i64,
    pub recent: i64,
    pub first: i64,
    pub helped: i64,
    pub unknown: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyWorkload {
    pub due: i64,
    pub overdue: i64,
    pub new: i64,
    pub introduced: i64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PretestOutcome {
    Attempted,
    Skipped,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecordPretestInput {
    pub card_id: String,
    pub outcome: PretestOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PretestRecord {
    pub pretest_id: String,
    pub concept_id: String,
    pub card_id: String,
    pub outcome: PretestOutcome,
    pub occurred_at: i64,
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
    pub assisted: Option<bool>,
    pub scheduling_state: SchedulingState,
    pub reviewed_at: i64,
    pub due_at: i64,
    pub scheduled_interval_days: f64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewHistoryEntry {
    pub id: String,
    pub rating: ReviewRating,
    pub assisted: Option<bool>,
    pub reviewed_at: i64,
    pub current_content: Option<bool>,
    pub reversed_at: Option<i64>,
    pub evidence_kind: Option<String>,
    pub uncertainty_marked: Option<bool>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewHistory {
    pub reviews: Vec<ReviewHistoryEntry>,
    pub has_more: bool,
    pub evidence: LearningEvidence,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewReversalOutcome {
    pub reversal_id: String,
    pub review_id: String,
    pub card_id: String,
    pub reversed_at: i64,
}
