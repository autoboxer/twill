use std::collections::HashSet;

use serde_json::{Map, Value};
use uuid::Uuid;

use crate::library::{
    ClozeSettings, ExplainSettings, ImageOcclusionSettings, LibraryError, LibraryResult,
    ProblemSettings, RetrievalFormKind, TypeAnswerSettings,
};

const MAXIMUM_ACCEPTED_ANSWERS: usize = 20;
const MAXIMUM_ACCEPTED_ANSWER_LENGTH: usize = 500;
const MAXIMUM_EXPLAIN_KEY_POINTS: usize = 12;
const MAXIMUM_EXPLAIN_KEY_POINT_LENGTH: usize = 280;
const MAXIMUM_PROBLEM_CHECKPOINTS: usize = 12;
const MAXIMUM_PROBLEM_CHECKPOINT_LENGTH: usize = 280;
const EMPTY_CONFIGURATION: &str = "{}";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RetrievalFormConfiguration {
    Recall,
    TypeAnswer(TypeAnswerSettings),
    Explain(ExplainSettings),
    Problem(ProblemSettings),
    Cloze(ClozeSettings),
    ImageOcclusion(ImageOcclusionSettings),
}

impl RetrievalFormConfiguration {
    pub const fn kind(&self) -> RetrievalFormKind {
        match self {
            Self::Recall => RetrievalFormKind::Recall,
            Self::TypeAnswer(_) => RetrievalFormKind::TypeAnswer,
            Self::Explain(_) => RetrievalFormKind::Explain,
            Self::Problem(_) => RetrievalFormKind::Problem,
            Self::Cloze(_) => RetrievalFormKind::Cloze,
            Self::ImageOcclusion(_) => RetrievalFormKind::ImageOcclusion,
        }
    }

    pub fn to_json(&self) -> LibraryResult<String> {
        match self {
            Self::Recall => Ok(EMPTY_CONFIGURATION.to_owned()),
            Self::TypeAnswer(settings) => serde_json::to_string(settings).map_err(Into::into),
            Self::Explain(settings) => serde_json::to_string(settings).map_err(Into::into),
            Self::Problem(settings) => serde_json::to_string(settings).map_err(Into::into),
            Self::Cloze(settings) => serde_json::to_string(settings).map_err(Into::into),
            Self::ImageOcclusion(settings) => serde_json::to_string(settings).map_err(Into::into),
        }
    }

    pub fn type_answer(&self) -> Option<&TypeAnswerSettings> {
        match self {
            Self::TypeAnswer(settings) => Some(settings),
            _ => None,
        }
    }

    pub fn explain(&self) -> Option<&ExplainSettings> {
        match self {
            Self::Explain(settings) => Some(settings),
            _ => None,
        }
    }

    pub fn problem(&self) -> Option<&ProblemSettings> {
        match self {
            Self::Problem(settings) => Some(settings),
            _ => None,
        }
    }

    pub fn cloze(&self) -> Option<&ClozeSettings> {
        match self {
            Self::Cloze(settings) => Some(settings),
            _ => None,
        }
    }

    pub fn image_occlusion(&self) -> Option<&ImageOcclusionSettings> {
        match self {
            Self::ImageOcclusion(settings) => Some(settings),
            _ => None,
        }
    }
}

pub(crate) fn normalize_explain(settings: ExplainSettings) -> LibraryResult<ExplainSettings> {
    if settings.key_points.is_empty() {
        return Err(LibraryError::MissingExplainKeyPoint);
    }

    if settings.key_points.len() > MAXIMUM_EXPLAIN_KEY_POINTS {
        return Err(LibraryError::TooManyExplainKeyPoints {
            maximum: MAXIMUM_EXPLAIN_KEY_POINTS,
        });
    }

    let mut key_points = Vec::with_capacity(settings.key_points.len());
    let mut normalized_points = HashSet::new();

    for key_point in settings.key_points {
        let key_point = key_point.split_whitespace().collect::<Vec<_>>().join(" ");

        if key_point.is_empty() {
            return Err(LibraryError::MissingExplainKeyPoint);
        }

        if key_point.chars().count() > MAXIMUM_EXPLAIN_KEY_POINT_LENGTH {
            return Err(LibraryError::ValueTooLong {
                field: "Explain key point",
                maximum: MAXIMUM_EXPLAIN_KEY_POINT_LENGTH,
            });
        }

        if !normalized_points.insert(key_point.to_lowercase()) {
            return Err(LibraryError::DuplicateExplainKeyPoint);
        }

        key_points.push(key_point);
    }

    Ok(ExplainSettings {
        focus: settings.focus,
        key_points,
    })
}

pub(crate) fn normalize_type_answer(
    settings: TypeAnswerSettings,
) -> LibraryResult<TypeAnswerSettings> {
    if settings.accepted_answers.is_empty() {
        return Err(LibraryError::MissingAcceptedAnswer);
    }

    if settings.accepted_answers.len() > MAXIMUM_ACCEPTED_ANSWERS {
        return Err(LibraryError::TooManyAcceptedAnswers {
            maximum: MAXIMUM_ACCEPTED_ANSWERS,
        });
    }

    let mut accepted_answers = Vec::with_capacity(settings.accepted_answers.len());
    let mut normalized_answers = HashSet::new();

    for answer in settings.accepted_answers {
        let answer = answer.split_whitespace().collect::<Vec<_>>().join(" ");

        if answer.is_empty() {
            return Err(LibraryError::MissingAcceptedAnswer);
        }

        if answer.chars().count() > MAXIMUM_ACCEPTED_ANSWER_LENGTH {
            return Err(LibraryError::ValueTooLong {
                field: "Accepted answer",
                maximum: MAXIMUM_ACCEPTED_ANSWER_LENGTH,
            });
        }

        if !normalized_answers.insert(answer.to_lowercase()) {
            return Err(LibraryError::DuplicateAcceptedAnswer);
        }

        accepted_answers.push(answer);
    }

    Ok(TypeAnswerSettings { accepted_answers })
}

pub(crate) fn normalize_problem(settings: ProblemSettings) -> LibraryResult<ProblemSettings> {
    if settings.checkpoints.is_empty() {
        return Err(LibraryError::MissingProblemCheckpoint);
    }

    if settings.checkpoints.len() > MAXIMUM_PROBLEM_CHECKPOINTS {
        return Err(LibraryError::TooManyProblemCheckpoints {
            maximum: MAXIMUM_PROBLEM_CHECKPOINTS,
        });
    }

    let mut checkpoints = Vec::with_capacity(settings.checkpoints.len());
    let mut normalized_checkpoints = HashSet::new();

    for checkpoint in settings.checkpoints {
        let checkpoint = checkpoint.split_whitespace().collect::<Vec<_>>().join(" ");

        if checkpoint.is_empty() {
            return Err(LibraryError::MissingProblemCheckpoint);
        }

        if checkpoint.chars().count() > MAXIMUM_PROBLEM_CHECKPOINT_LENGTH {
            return Err(LibraryError::ValueTooLong {
                field: "Problem checkpoint",
                maximum: MAXIMUM_PROBLEM_CHECKPOINT_LENGTH,
            });
        }

        if !normalized_checkpoints.insert(checkpoint.to_lowercase()) {
            return Err(LibraryError::DuplicateProblemCheckpoint);
        }

        checkpoints.push(checkpoint);
    }

    Ok(ProblemSettings { checkpoints })
}

pub(crate) fn parse_retrieval_form_configuration(
    retrieval_kind: RetrievalFormKind,
    configuration: &str,
) -> LibraryResult<RetrievalFormConfiguration> {
    match retrieval_kind {
        RetrievalFormKind::Recall => {
            let configuration: Map<String, Value> = serde_json::from_str(configuration)?;

            if configuration.is_empty() {
                Ok(RetrievalFormConfiguration::Recall)
            } else {
                Err(LibraryError::InvalidRetrievalForm)
            }
        }
        RetrievalFormKind::TypeAnswer => {
            let settings = serde_json::from_str(configuration)?;
            let settings =
                normalize_type_answer(settings).map_err(|_| LibraryError::InvalidRetrievalForm)?;

            Ok(RetrievalFormConfiguration::TypeAnswer(settings))
        }
        RetrievalFormKind::Explain => {
            let settings = serde_json::from_str(configuration)?;
            let settings =
                normalize_explain(settings).map_err(|_| LibraryError::InvalidRetrievalForm)?;

            Ok(RetrievalFormConfiguration::Explain(settings))
        }
        RetrievalFormKind::Problem => {
            let settings = serde_json::from_str(configuration)?;
            let settings =
                normalize_problem(settings).map_err(|_| LibraryError::InvalidRetrievalForm)?;

            Ok(RetrievalFormConfiguration::Problem(settings))
        }
        RetrievalFormKind::Cloze => {
            let settings: ClozeSettings = serde_json::from_str(configuration)?;

            validate_group_id(&settings.group_id)?;

            Ok(RetrievalFormConfiguration::Cloze(settings))
        }
        RetrievalFormKind::ImageOcclusion => {
            let settings: ImageOcclusionSettings = serde_json::from_str(configuration)?;

            validate_group_id(&settings.group_id)?;

            Ok(RetrievalFormConfiguration::ImageOcclusion(settings))
        }
    }
}

fn validate_group_id(group_id: &str) -> LibraryResult<()> {
    if Uuid::parse_str(group_id).is_err() {
        return Err(LibraryError::InvalidRetrievalForm);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{parse_retrieval_form_configuration, RetrievalFormConfiguration};
    use crate::library::models::ExplainFocus;
    use crate::library::{
        ClozeSettings, ExplainSettings, ImageOcclusionSettings, ProblemSettings, RetrievalFormKind,
        TypeAnswerSettings,
    };

    #[test]
    fn configuration_variants_preserve_stored_json_and_kind() {
        let group_id = "0192abf7-22b3-7000-8000-000000000001";
        let cases = [
            (
                RetrievalFormConfiguration::Recall,
                RetrievalFormKind::Recall,
                json!({}),
            ),
            (
                RetrievalFormConfiguration::TypeAnswer(TypeAnswerSettings {
                    accepted_answers: vec!["Alpha".to_owned()],
                }),
                RetrievalFormKind::TypeAnswer,
                json!({ "acceptedAnswers": ["Alpha"] }),
            ),
            (
                RetrievalFormConfiguration::Explain(ExplainSettings {
                    focus: ExplainFocus::CauseAndEffect,
                    key_points: vec!["Heat increases motion".to_owned()],
                }),
                RetrievalFormKind::Explain,
                json!({ "focus": "causeAndEffect", "keyPoints": ["Heat increases motion"] }),
            ),
            (
                RetrievalFormConfiguration::Problem(ProblemSettings {
                    checkpoints: vec!["Check the units".to_owned()],
                }),
                RetrievalFormKind::Problem,
                json!({ "checkpoints": ["Check the units"] }),
            ),
            (
                RetrievalFormConfiguration::Cloze(ClozeSettings {
                    group_id: group_id.to_owned(),
                }),
                RetrievalFormKind::Cloze,
                json!({ "groupId": group_id }),
            ),
            (
                RetrievalFormConfiguration::ImageOcclusion(ImageOcclusionSettings {
                    group_id: group_id.to_owned(),
                }),
                RetrievalFormKind::ImageOcclusion,
                json!({ "groupId": group_id }),
            ),
        ];

        for (form, kind, expected) in cases {
            assert_eq!(form.kind(), kind);

            let serialized = form.to_json().unwrap();
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&serialized).unwrap(),
                expected
            );

            let parsed = parse_retrieval_form_configuration(kind, &serialized).unwrap();
            assert_eq!(parsed, form);

            let populated_settings = [
                parsed.type_answer().is_some(),
                parsed.explain().is_some(),
                parsed.problem().is_some(),
                parsed.cloze().is_some(),
                parsed.image_occlusion().is_some(),
            ]
            .into_iter()
            .filter(|populated| *populated)
            .count();
            assert_eq!(
                populated_settings,
                usize::from(kind != RetrievalFormKind::Recall)
            );
        }
    }

    #[test]
    fn configuration_reads_reject_wrong_shapes_and_invalid_settings() {
        let cases = [
            (RetrievalFormKind::Recall, "null"),
            (RetrievalFormKind::Recall, "[]"),
            (RetrievalFormKind::Recall, r#"{"unexpected":true}"#),
            (RetrievalFormKind::TypeAnswer, "{}"),
            (RetrievalFormKind::TypeAnswer, r#"{"acceptedAnswers":[]}"#),
            (
                RetrievalFormKind::TypeAnswer,
                r#"{"acceptedAnswers":["Alpha"," alpha "]}"#,
            ),
            (
                RetrievalFormKind::TypeAnswer,
                r#"{"acceptedAnswers":["Alpha"],"checkpoints":["Step"]}"#,
            ),
            (
                RetrievalFormKind::Explain,
                r#"{"focus":"why","keyPoints":[" "]}"#,
            ),
            (
                RetrievalFormKind::Explain,
                r#"{"focus":"unknown","keyPoints":["Point"]}"#,
            ),
            (
                RetrievalFormKind::Problem,
                r#"{"checkpoints":["Step"," step "]}"#,
            ),
            (RetrievalFormKind::Cloze, r#"{"groupId":"invalid"}"#),
            (RetrievalFormKind::ImageOcclusion, r#"{"groupId":""}"#),
        ];

        for (kind, configuration) in cases {
            assert!(
                parse_retrieval_form_configuration(kind, configuration).is_err(),
                "{kind:?}: {configuration}"
            );
        }
    }

    #[test]
    fn configuration_reads_preserve_text_normalization() {
        let cases = [
            (
                RetrievalFormKind::TypeAnswer,
                json!({ "acceptedAnswers": ["  Alpha\n beta  "] }),
                json!({ "acceptedAnswers": ["Alpha beta"] }),
            ),
            (
                RetrievalFormKind::Explain,
                json!({ "focus": "why", "keyPoints": ["  One\t point  "] }),
                json!({ "focus": "why", "keyPoints": ["One point"] }),
            ),
            (
                RetrievalFormKind::Problem,
                json!({ "checkpoints": ["  Check\n units  "] }),
                json!({ "checkpoints": ["Check units"] }),
            ),
        ];

        for (kind, input, expected) in cases {
            let form = parse_retrieval_form_configuration(kind, &input.to_string()).unwrap();
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&form.to_json().unwrap()).unwrap(),
                expected
            );
        }
    }
}
