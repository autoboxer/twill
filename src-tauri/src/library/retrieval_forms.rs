use std::collections::HashSet;

use serde_json::{Map, Value};
use uuid::Uuid;

use crate::library::{
    ClozeSettings, LibraryError, LibraryResult, RetrievalFormKind, TypeAnswerSettings,
};

const MAXIMUM_ACCEPTED_ANSWERS: usize = 20;
const MAXIMUM_ACCEPTED_ANSWER_LENGTH: usize = 500;
const EMPTY_CONFIGURATION: &str = "{}";

pub(crate) fn normalize_type_answer(
    settings: Option<TypeAnswerSettings>,
) -> LibraryResult<Option<TypeAnswerSettings>> {
    let Some(settings) = settings else {
        return Ok(None);
    };

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

    Ok(Some(TypeAnswerSettings { accepted_answers }))
}

pub(crate) fn retrieval_form_configuration(
    retrieval_kind: RetrievalFormKind,
    type_answer: Option<&TypeAnswerSettings>,
    cloze: Option<&ClozeSettings>,
) -> LibraryResult<String> {
    match (retrieval_kind, type_answer, cloze) {
        (RetrievalFormKind::Recall, None, None) => Ok(EMPTY_CONFIGURATION.to_owned()),
        (RetrievalFormKind::TypeAnswer, Some(settings), None) => {
            serde_json::to_string(settings).map_err(Into::into)
        }
        (RetrievalFormKind::Cloze, None, Some(settings)) => {
            serde_json::to_string(settings).map_err(Into::into)
        }
        _ => Err(LibraryError::InvalidRetrievalForm),
    }
}

pub(crate) fn parse_retrieval_form_configuration(
    retrieval_kind: RetrievalFormKind,
    configuration: &str,
) -> LibraryResult<(Option<TypeAnswerSettings>, Option<ClozeSettings>)> {
    match retrieval_kind {
        RetrievalFormKind::Recall => {
            let configuration: Map<String, Value> = serde_json::from_str(configuration)?;

            if configuration.is_empty() {
                Ok((None, None))
            } else {
                Err(LibraryError::InvalidRetrievalForm)
            }
        }
        RetrievalFormKind::TypeAnswer => {
            let settings = serde_json::from_str(configuration)?;
            let settings = normalize_type_answer(Some(settings))
                .map_err(|_| LibraryError::InvalidRetrievalForm)?;

            Ok((settings, None))
        }
        RetrievalFormKind::Cloze => {
            let settings: ClozeSettings = serde_json::from_str(configuration)?;

            if Uuid::parse_str(&settings.group_id).is_err() {
                return Err(LibraryError::InvalidRetrievalForm);
            }

            Ok((None, Some(settings)))
        }
    }
}
