use rusqlite::params;

use super::{concept_archived, invalid, link_objective, query_links, PracticeLinkLibrary};
use crate::library::{
    LibraryResult, LinkedPracticeInput, PracticeComparison, PracticeComparisonCase,
    SetPracticeComparisonInput,
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
    })
}

pub(super) fn read_comparison(
    json: Option<String>,
) -> rusqlite::Result<Option<PracticeComparison>> {
    json.map(|value| {
        if value.len() > 16_000 {
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
