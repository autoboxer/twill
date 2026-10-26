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
    pub image_labeling: Option<ImageLabelingSettings>,
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
    ImageLabeling,
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
            Self::ImageLabeling => "image_labeling",
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
            "image_labeling" => Ok(Self::ImageLabeling),
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
pub struct ImageLabelingSettings {
    pub target_id: String,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worked_example: Option<WorkedExample>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkedExample {
    pub steps: Vec<WorkedStep>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_diagnosis: Option<ProblemErrorDiagnosis>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProblemErrorDiagnosis {
    pub attempt: String,
    pub location: String,
    pub explanation: String,
    pub correction: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkedStep {
    pub id: String,
    pub prompt: String,
    pub solution: String,
    pub principle_prompt: String,
    pub show_in_completion: bool,
}
