use serde::{Deserialize, Serialize};

use super::{
    ClozeSettings, ConceptContent, ExplainSettings, ImageOcclusionSettings, MediaSummary,
    ProblemSettings, RetrievalFormKind, TemplateContent, TypeAnswerSettings,
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
    pub concept_id: String,
    pub concept_last_change_id: String,
    pub concept_title: String,
    pub content: ConceptContent,
    pub retrieval_kind: RetrievalFormKind,
    pub explain: Option<ExplainSettings>,
    pub problem: Option<ProblemSettings>,
    pub cloze: Option<ClozeSettings>,
    pub image_occlusion: Option<ImageOcclusionSettings>,
    pub type_answer: Option<TypeAnswerSettings>,
    pub template: Option<StudyTemplate>,
    pub scheduling_state: SchedulingState,
    pub due_at: i64,
    pub pretest_eligible: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyQueue {
    pub cards: Vec<StudyCard>,
    pub media: Vec<MediaSummary>,
    pub next_due_at: Option<i64>,
    pub total_cards: i64,
    pub mixed_practice_enabled: bool,
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
