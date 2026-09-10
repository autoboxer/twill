use serde::{Deserialize, Serialize};

use super::{ClozeSettings, ImageOcclusionSettings, NamedItem, RetrievalFormKind};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CardQualityKind {
    Ambiguous,
    Incorrect,
    Outdated,
    TooEasy,
    TooDifficult,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CardQualitySource {
    Manual,
    ReviewPattern,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CardQualityStatus {
    Open,
    Resolved,
    Dismissed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CardQualityDisposition {
    Resolved,
    Dismissed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CardQualitySignal {
    RepeatedDifficulty,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CardQualityConcern {
    pub id: String,
    pub card_id: String,
    pub source: CardQualitySource,
    pub kind: CardQualityKind,
    pub status: CardQualityStatus,
    pub note: String,
    pub observed_through_review_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CardQualityEvidence {
    pub reviews_considered: i64,
    pub again_count: i64,
    pub observed_through_review_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CardQualityQueueItem {
    pub concern_id: Option<String>,
    pub card_id: String,
    pub concept_id: String,
    pub concept_title: String,
    pub retrieval_kind: RetrievalFormKind,
    pub template: Option<NamedItem>,
    pub cloze: Option<ClozeSettings>,
    pub image_occlusion: Option<ImageOcclusionSettings>,
    pub source: CardQualitySource,
    pub kind: CardQualityKind,
    pub note: String,
    pub noticed_at: i64,
    pub evidence: Option<CardQualityEvidence>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CardQualityQueue {
    pub items: Vec<CardQualityQueueItem>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateCardQualityConcernInput {
    pub card_id: String,
    pub kind: CardQualityKind,
    pub note: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CloseCardQualityConcernInput {
    pub concern_id: String,
    pub disposition: CardQualityDisposition,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DismissCardQualitySignalInput {
    pub card_id: String,
    pub signal: CardQualitySignal,
    pub observed_through_review_id: String,
}
