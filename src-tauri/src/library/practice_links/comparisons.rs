use rusqlite::params;
use std::collections::HashSet;
use uuid::Uuid;

use super::{concept_archived, invalid, link_objective, query_links, PracticeLinkLibrary};
use crate::library::{
    LibraryResult, LinkedPracticeInput, PracticeChoice, PracticeChoiceOption, PracticeComparison,
    PracticeComparisonCase, SetPracticeComparisonInput,
};

impl PracticeLinkLibrary<'_> {
    pub fn comparison(&self, input: LinkedPracticeInput) -> LibraryResult<PracticeComparisonCase> {
        self.store.read_result(|connection| {
            if concept_archived(connection, &input.concept_id)? {
                return Err(invalid(
                    "Restore the current concept before comparison practice.",
                ));
            }

            let link = query_links(connection, &input.concept_id)?
                .into_iter()
                .find(|link| link.id == input.link_id)
                .ok_or_else(|| {
                    crate::library::LibraryError::PracticeLinkNotFound(input.link_id.clone())
                })?;

            if link.archived {
                return Err(invalid(
                    "Restore this archived case before comparison practice.",
                ));
            }

            if link.comparison.is_none() {
                return Err(invalid("Add a comparison before practicing these cases."));
            }

            let source = crate::library::service::query_concept(connection, &input.concept_id)?;
            let related = crate::library::service::query_concept(connection, &link.concept_id)?;

            Ok(PracticeComparisonCase {
                link,
                source,
                related,
            })
        })
    }

    pub fn set_comparison(&self, input: SetPracticeComparisonInput) -> LibraryResult<()> {
        let comparison = input.comparison.map(normalize_comparison).transpose()?;

        self.store.write_result(|transaction| {
            link_objective(transaction, &input.id, &input.expected_change_id)?;

            let current = transaction.query_row(
                "SELECT comparison_json FROM practice_links WHERE entity_id = ?1",
                [&input.id],
                |row| read_comparison(row.get(0)?),
            )?;

            if current == comparison {
                return Ok(());
            }

            let json = comparison.as_ref().map(serde_json::to_string).transpose()?;
            let entity = transaction.touch_entity(&input.id)?;

            transaction.execute(
                "UPDATE practice_links SET comparison_json = ?1, last_change_id = ?2
                WHERE entity_id = ?3",
                params![json, entity.last_change_id, input.id],
            )?;

            Ok(())
        })
    }
}

pub(super) fn normalize_comparison(
    comparison: PracticeComparison,
) -> LibraryResult<PracticeComparison> {
    let prompt = comparison.prompt.trim().to_owned();
    let explanation = comparison.explanation.trim().to_owned();

    if prompt.is_empty() || prompt.chars().count() > 500 {
        return Err(invalid("Write a comparison prompt in 1 to 500 characters."));
    }

    if explanation.is_empty() || explanation.chars().count() > 2_000 {
        return Err(invalid(
            "Write a comparison explanation in 1 to 2,000 characters.",
        ));
    }

    Ok(PracticeComparison {
        prompt,
        explanation,
        choice: comparison.choice.map(normalize_choice).transpose()?,
    })
}

fn normalize_choice(mut choice: PracticeChoice) -> LibraryResult<PracticeChoice> {
    if !(2..=6).contains(&choice.options.len()) {
        return Err(invalid("Provide two to six answer options."));
    }

    let mut ids = HashSet::new();
    let mut labels = HashSet::new();

    for option in &mut choice.options {
        normalize_choice_option(option)?;

        let label = option
            .text
            .to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        if !ids.insert(option.id.clone()) || !labels.insert(label) {
            return Err(invalid(
                "Give each answer option a different identity and text.",
            ));
        }
    }

    choice.correct_option_id = normalize_option_id(&choice.correct_option_id)?;

    if !ids.contains(&choice.correct_option_id) {
        return Err(invalid("Choose one of the options as the correct answer."));
    }

    Ok(choice)
}

fn normalize_choice_option(option: &mut PracticeChoiceOption) -> LibraryResult<()> {
    option.id = normalize_option_id(&option.id)?;
    option.text = option.text.trim().to_owned();
    option.feedback = option.feedback.trim().to_owned();

    if option.text.is_empty() || option.text.chars().count() > 300 {
        return Err(invalid("Write each option in 1 to 300 characters."));
    }

    if option.feedback.is_empty() || option.feedback.chars().count() > 1_000 {
        return Err(invalid(
            "Write feedback for each option in 1 to 1,000 characters.",
        ));
    }

    Ok(())
}

fn normalize_option_id(value: &str) -> LibraryResult<String> {
    Uuid::parse_str(value.trim())
        .map(|id| id.to_string())
        .map_err(|_| invalid("Each answer option needs a valid identity."))
}

pub(super) fn read_comparison(
    json: Option<String>,
) -> rusqlite::Result<Option<PracticeComparison>> {
    json.map(|value| {
        if value.len() > 64_000 {
            return Err(rusqlite::Error::InvalidQuery);
        }

        serde_json::from_str(&value).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })
    })
    .transpose()
}

#[cfg(test)]
mod tests;
