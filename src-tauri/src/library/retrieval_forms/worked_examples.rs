use std::collections::HashSet;

use uuid::Uuid;

use crate::library::models::{WorkedExample, WorkedStep};
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
}
