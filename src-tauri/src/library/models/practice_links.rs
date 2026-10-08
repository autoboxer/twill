use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PracticeLink {
    pub id: String,
    pub concept_id: String,
    pub title: String,
    pub archived: bool,
    pub objective: String,
    pub last_change_id: String,
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
