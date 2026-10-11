use serde::{Deserialize, Serialize};

use super::{ConceptDetail, StudyTemplate};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkedPracticeCase {
    pub link: PracticeLink,
    pub source_last_change_id: String,
    pub concept: ConceptDetail,
    pub templates: Vec<StudyTemplate>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LinkedPracticeInput {
    pub concept_id: String,
    pub link_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PracticeLink {
    pub id: String,
    pub concept_id: String,
    pub title: String,
    pub archived: bool,
    pub objective: String,
    pub comparison: Option<PracticeComparison>,
    pub last_change_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PracticeComparison {
    pub prompt: String,
    pub explanation: String,
    pub choice: Option<PracticeChoice>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PracticeChoice {
    pub options: Vec<PracticeChoiceOption>,
    pub correct_option_id: String,
    pub shuffle: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PracticeChoiceOption {
    pub id: String,
    pub text: String,
    pub feedback: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PracticeComparisonCase {
    pub link: PracticeLink,
    pub source: ConceptDetail,
    pub related: ConceptDetail,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SetPracticeComparisonInput {
    pub id: String,
    pub expected_change_id: String,
    pub comparison: Option<PracticeComparison>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatePracticeLinkInput {
    pub concept_id: String,
    pub related_concept_id: String,
    pub objective: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdatePracticeLinkInput {
    pub id: String,
    pub expected_change_id: String,
    pub objective: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RemovePracticeLinkInput {
    pub id: String,
    pub expected_change_id: String,
}
