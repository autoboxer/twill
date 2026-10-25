use std::collections::HashSet;

use uuid::Uuid;

use crate::library::models::{ProblemErrorDiagnosis, WorkedExample, WorkedStep};
use crate::library::{LibraryError, LibraryResult};

const MAXIMUM_STEPS: usize = 12;
const MAXIMUM_PROMPT_LENGTH: usize = 280;
const MAXIMUM_SOLUTION_LENGTH: usize = 2000;
const MAXIMUM_PRINCIPLE_PROMPT_LENGTH: usize = 500;

pub(super) fn normalize(mut example: WorkedExample) -> LibraryResult<WorkedExample> {
    if example.steps.is_empty() || example.steps.len() > MAXIMUM_STEPS {
        return Err(LibraryError::InvalidWorkedExample {
            message: "a worked example must have between 1 and 12 steps",
        });
    }

    let mut ids = HashSet::new();

    for step in &mut example.steps {
        let id = Uuid::parse_str(&step.id).map_err(|_| LibraryError::InvalidWorkedExample {
            message: "worked steps must have valid identifiers",
        })?;

        if !ids.insert(id) {
            return Err(LibraryError::InvalidWorkedExample {
                message: "worked steps must have unique identifiers",
            });
        }

        step.id = id.to_string();
        normalize_step_text(step)?;
    }

    if let Some(diagnosis) = &mut example.error_diagnosis {
        normalize_error_diagnosis(diagnosis)?;
    }

    Ok(example)
}

fn normalize_step_text(step: &mut WorkedStep) -> LibraryResult<()> {
    step.prompt = normalize_text(&step.prompt, "Step prompt", MAXIMUM_PROMPT_LENGTH, true)?;
    step.solution = normalize_text(
        &step.solution,
        "Step solution",
        MAXIMUM_SOLUTION_LENGTH,
        true,
    )?;
    step.principle_prompt = normalize_text(
        &step.principle_prompt,
        "Principle question",
        MAXIMUM_PRINCIPLE_PROMPT_LENGTH,
        false,
    )?;

    Ok(())
}

fn normalize_error_diagnosis(diagnosis: &mut ProblemErrorDiagnosis) -> LibraryResult<()> {
    diagnosis.attempt = normalize_text(
        &diagnosis.attempt,
        "Example with an error",
        MAXIMUM_SOLUTION_LENGTH,
        true,
    )?;
    diagnosis.location = normalize_text(
        &diagnosis.location,
        "Error location",
        MAXIMUM_PROMPT_LENGTH,
        true,
    )?;
    diagnosis.explanation = normalize_text(
        &diagnosis.explanation,
        "Error explanation",
        MAXIMUM_SOLUTION_LENGTH,
        true,
    )?;
    diagnosis.correction = normalize_text(
        &diagnosis.correction,
        "Corrected work",
        MAXIMUM_SOLUTION_LENGTH,
        true,
    )?;

    Ok(())
}

fn normalize_text(
    text: &str,
    field: &'static str,
    maximum: usize,
    required: bool,
) -> LibraryResult<String> {
    let text = text.trim();

    if required && text.is_empty() {
        return Err(LibraryError::EmptyValue { field });
    }

    if text.chars().count() > maximum {
        return Err(LibraryError::ValueTooLong { field, maximum });
    }

    Ok(text.to_owned())
}

#[cfg(test)]
mod tests {
    use super::normalize;
    use crate::library::models::{WorkedExample, WorkedStep};
    use crate::library::retrieval_forms::parse_retrieval_form_configuration;
    use crate::library::{LibraryError, RetrievalFormKind};

    fn example() -> WorkedExample {
        WorkedExample {
            error_diagnosis: None,
            steps: vec![WorkedStep {
                id: "0192abf7-22b3-7000-8000-000000000001".to_owned(),
                prompt: "  Find the acceleration.  ".to_owned(),
                solution: "  a = F / m\na = 12 / 3 = 4 m/s²  ".to_owned(),
                principle_prompt: "  Why does this equation apply?  ".to_owned(),
                show_in_completion: false,
            }],
        }
    }

    #[test]
    fn normalization_preserves_order_line_breaks_visibility_and_unicode_limits() {
        let mut input = example();
        let mut second = input.steps[0].clone();

        second.id = "0192ABF7-22B3-7000-8000-000000000002".to_owned();
        second.prompt = "🌿".repeat(280);
        second.solution = "網".repeat(2000);
        second.principle_prompt = "é".repeat(500);
        second.show_in_completion = true;
        input.steps.push(second);

        let saved = normalize(input).unwrap();

        assert_eq!(saved.steps[0].prompt, "Find the acceleration.");
        assert_eq!(saved.steps[0].solution, "a = F / m\na = 12 / 3 = 4 m/s²");
        assert_eq!(
            saved.steps[0].principle_prompt,
            "Why does this equation apply?"
        );
        assert!(!saved.steps[0].show_in_completion);
        assert!(saved.steps[1].show_in_completion);
        assert_eq!(saved.steps[1].id, "0192abf7-22b3-7000-8000-000000000002");
    }

    #[test]
    fn invalid_counts_identifiers_and_empty_solutions_are_rejected() {
        for count in [0, 13] {
            let mut input = example();
            input.steps = vec![input.steps[0].clone(); count];
            assert!(normalize(input).is_err());
        }

        let mut input = example();
        input.steps.push(input.steps[0].clone());
        assert!(normalize(input.clone()).is_err());
        input.steps.pop();
        input.steps[0].id = "not-a-uuid".to_owned();
        assert!(normalize(input).is_err());
        input = example();
        input.steps[0].prompt = " \n ".to_owned();
        assert!(matches!(
            normalize(input),
            Err(LibraryError::EmptyValue {
                field: "Step prompt"
            })
        ));
        input = example();
        input.steps[0].solution = " \n ".to_owned();
        assert!(matches!(
            normalize(input),
            Err(LibraryError::EmptyValue {
                field: "Step solution"
            })
        ));
    }

    #[test]
    fn field_limits_apply_on_configuration_read_and_unknown_fields_are_rejected() {
        for field in ["prompt", "solution", "principlePrompt"] {
            let mut settings = serde_json::json!({
                "checkpoints": ["Include units."],
                "workedExample": example(),
            });
            let limit = match field {
                "prompt" => 280,
                "solution" => 2000,
                _ => 500,
            };
            settings["workedExample"]["steps"][0][field] =
                serde_json::json!("🌿".repeat(limit + 1));

            assert!(parse_retrieval_form_configuration(
                RetrievalFormKind::Problem,
                &settings.to_string()
            )
            .is_err());
        }

        let mut settings = serde_json::json!({
            "checkpoints": ["Include units."],
            "workedExample": example(),
        });
        assert!(parse_retrieval_form_configuration(
            RetrievalFormKind::Problem,
            &settings.to_string()
        )
        .is_ok());
        settings["workedExample"]["steps"][0]["unexpected"] = serde_json::json!(true);
        assert!(parse_retrieval_form_configuration(
            RetrievalFormKind::Problem,
            &settings.to_string()
        )
        .is_err());
    }

    fn diagnosis_configuration() -> serde_json::Value {
        serde_json::json!({
            "checkpoints": ["Include units."],
            "workedExample": {
                "steps": example().steps,
                "errorDiagnosis": {
                    "attempt": "  a = m / F\na = 3 / 12  ",
                    "location": "  The rearrangement.  ",
                    "explanation": "  Divide force by mass, not mass by force.  ",
                    "correction": "  a = F / m\na = 4 m/s²  "
                }
            }
        })
    }

    #[test]
    fn error_diagnosis_normalizes_text_and_preserves_a_correct_example() {
        let settings = diagnosis_configuration();
        let saved =
            normalize(serde_json::from_value(settings["workedExample"].clone()).unwrap()).unwrap();
        let diagnosis = saved.error_diagnosis.unwrap();

        assert_eq!(diagnosis.attempt, "a = m / F\na = 3 / 12");
        assert_eq!(diagnosis.location, "The rearrangement.");
        assert_eq!(
            diagnosis.explanation,
            "Divide force by mass, not mass by force."
        );
        assert_eq!(diagnosis.correction, "a = F / m\na = 4 m/s²");
        assert_eq!(saved.steps[0].solution, "a = F / m\na = 12 / 3 = 4 m/s²");
    }

    #[test]
    fn diagnosis_requires_all_fields_with_unicode_limits_on_write_and_read() {
        for field in ["attempt", "location", "explanation", "correction"] {
            let maximum = if field == "location" { 280 } else { 2000 };
            let mut settings = diagnosis_configuration();
            settings["workedExample"]["errorDiagnosis"][field] =
                serde_json::json!("🌿".repeat(maximum));
            assert!(parse_retrieval_form_configuration(
                RetrievalFormKind::Problem,
                &settings.to_string()
            )
            .is_ok());

            for invalid in [" \n ".to_owned(), "🌿".repeat(maximum + 1)] {
                settings["workedExample"]["errorDiagnosis"][field] = serde_json::json!(invalid);
                let example = serde_json::from_value(settings["workedExample"].clone()).unwrap();
                assert!(normalize(example).is_err());
                assert!(parse_retrieval_form_configuration(
                    RetrievalFormKind::Problem,
                    &settings.to_string()
                )
                .is_err());
            }

            settings["workedExample"]["errorDiagnosis"]
                .as_object_mut()
                .unwrap()
                .remove(field);
            assert!(parse_retrieval_form_configuration(
                RetrievalFormKind::Problem,
                &settings.to_string()
            )
            .is_err());
        }

        for invalid in [
            serde_json::json!([]),
            serde_json::json!("mistake"),
            serde_json::json!({
                "attempt": "a = m / F", "location": "Rearrangement", "explanation": "Reversed.",
                "correction": "a = F / m", "unexpected": true
            }),
        ] {
            let mut settings = diagnosis_configuration();
            settings["workedExample"]["errorDiagnosis"] = invalid;
            assert!(parse_retrieval_form_configuration(
                RetrievalFormKind::Problem,
                &settings.to_string()
            )
            .is_err());
        }

        let mut settings = diagnosis_configuration();
        settings["workedExample"]["steps"] = serde_json::json!([]);
        assert!(parse_retrieval_form_configuration(
            RetrievalFormKind::Problem,
            &settings.to_string()
        )
        .is_err());
    }
}
