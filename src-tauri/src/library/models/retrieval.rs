use serde::{Deserialize, Serialize};

use super::{NamedItem, SchedulingState};
use crate::library::LibraryError;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CardSummary {
    pub id: String,
    pub retrieval_kind: RetrievalFormKind,
    pub explain: Option<ExplainSettings>,
    pub problem: Option<ProblemSettings>,
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
    Problem,
    Cloze,
    ImageOcclusion,
}

impl RetrievalFormKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Recall => "recall",
            Self::TypeAnswer => "type_answer",
            Self::Explain => "explain",
            Self::Problem => "problem",
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
            "problem" => Ok(Self::Problem),
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProblemSettings {
    pub checkpoints: Vec<String>,
}
