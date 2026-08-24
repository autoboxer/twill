use std::collections::BTreeSet;

use fsrs::{ItemState, MemoryState, FSRS};
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::data::{current_timestamp, EntityKind, WriteTransaction};
use crate::library::media::query_media_for_concepts;
use crate::library::models::TemplateMode;
use crate::library::retrieval_forms::{
    parse_retrieval_form_configuration, retrieval_form_configuration,
};
use crate::library::{
    ClozeSettings, ImageOcclusionSettings, LibraryError, LibraryResult,
    RetrievalFormKind, ReviewOutcome, ReviewRating, ReviewReversalOutcome,
    SchedulingSettings, SchedulingState, StudyCard, StudyQueue, StudyTemplate,
    TypeAnswerSettings,
    UpdateSchedulingSettingsInput,
};

const FSRS_ALGORITHM: &str = "fsrs";
const FSRS_VERSION: &str = "6.6.1";
const MILLISECONDS_PER_DAY: i64 = 86_400_000;
const MINIMUM_DESIRED_RETENTION: f64 = 0.80;
const MAXIMUM_DESIRED_RETENTION: f64 = 0.97;
const MINIMUM_INTERVAL_DAYS: i64 = 1;
const MAXIMUM_INTERVAL_DAYS: i64 = 36_500;

struct StoredSchedule {
    state: SchedulingState,
    due_at: i64,
    stability: Option<f64>,
    difficulty: Option<f64>,
    last_reviewed_at: Option<i64>,
    last_review_id: Option<String>,
    review_count: i64,
    lapse_count: i64,
}

struct SchedulerConfiguration {
    id: String,
    algorithm_version: String,
    parameters_json: String,
    desired_retention: f64,
    maximum_interval_days: i64,
    scheduler: FSRS,
}

pub fn create_recall_card(
    transaction: &WriteTransaction<'_>,
    concept_id: &str,
    template_id: Option<&str>,
) -> LibraryResult<()> {
    create_card(
        transaction,
        concept_id,
        RetrievalFormKind::Recall,
        template_id,
        None,
        None,
        None,
    )
}

pub fn create_type_answer_card(
    transaction: &WriteTransaction<'_>,
    concept_id: &str,
    settings: &TypeAnswerSettings,
) -> LibraryResult<()> {
    create_card(
        transaction,
        concept_id,
        RetrievalFormKind::TypeAnswer,
        None,
        Some(settings),
        None,
        None,
    )
}

pub fn create_cloze_card(
    transaction: &WriteTransaction<'_>,
    concept_id: &str,
    group_id: &str,
) -> LibraryResult<()> {
    let settings = ClozeSettings {
        group_id: group_id.to_owned(),
    };

    create_card(
        transaction,
        concept_id,
        RetrievalFormKind::Cloze,
        None,
        None,
        Some(&settings),
        None,
    )
}

pub fn create_image_occlusion_card(
    transaction: &WriteTransaction<'_>,
    concept_id: &str,
    group_id: &str,
) -> LibraryResult<()> {
    let settings = ImageOcclusionSettings {
        group_id: group_id.to_owned(),
    };

    create_card(
        transaction,
        concept_id,
        RetrievalFormKind::ImageOcclusion,
        None,
        None,
        None,
        Some(&settings),
    )
}

fn create_card(
    transaction: &WriteTransaction<'_>,
    concept_id: &str,
    retrieval_kind: RetrievalFormKind,
    template_id: Option<&str>,
    type_answer: Option<&TypeAnswerSettings>,
    cloze: Option<&ClozeSettings>,
    image_occlusion: Option<&ImageOcclusionSettings>,
) -> LibraryResult<()> {
    let configuration = retrieval_form_configuration(
        retrieval_kind,
        type_answer,
        cloze,
        image_occlusion,
    )?;
    let entity = transaction.create_entity(EntityKind::Card)?;

    transaction.execute(
        "INSERT INTO cards (
            entity_id,
            concept_id,
            retrieval_kind,
            configuration_json,
            template_id,
            last_change_id
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            entity.id,
            concept_id,
            retrieval_kind.as_str(),
            configuration,
            template_id,
            entity.last_change_id
        ],
    )?;
    transaction.execute(
        "INSERT INTO card_scheduling (
            card_id,
            state,
            due_at,
            stability,
            difficulty,
            last_reviewed_at,
            last_review_id,
            review_count,
            lapse_count
        ) VALUES (?1, 'new', ?2, NULL, NULL, NULL, NULL, 0, 0)",
        params![entity.id, entity.created_at],
    )?;

    Ok(())
}

pub fn query_study_queue(connection: &Connection, now: i64) -> LibraryResult<StudyQueue> {
    let total_cards = connection.query_row(
        "SELECT COUNT(*)
        FROM cards
        INNER JOIN entities AS card_entities
            ON card_entities.id = cards.entity_id
        INNER JOIN concepts
            ON concepts.entity_id = cards.concept_id
        INNER JOIN entities AS concept_entities
            ON concept_entities.id = concepts.entity_id
        LEFT JOIN entities AS template_entities
            ON template_entities.id = cards.template_id
        WHERE card_entities.deleted_at IS NULL
            AND concept_entities.deleted_at IS NULL
            AND concepts.archived_at IS NULL
            AND (
                cards.template_id IS NULL
                OR template_entities.deleted_at IS NULL
            )",
        [],
        |row| row.get(0),
    )?;
    let next_due_at = connection.query_row(
        "SELECT MIN(card_scheduling.due_at)
        FROM card_scheduling
        INNER JOIN cards
            ON cards.entity_id = card_scheduling.card_id
        INNER JOIN entities AS card_entities
            ON card_entities.id = cards.entity_id
        INNER JOIN concepts
            ON concepts.entity_id = cards.concept_id
        INNER JOIN entities AS concept_entities
            ON concept_entities.id = concepts.entity_id
        LEFT JOIN entities AS template_entities
            ON template_entities.id = cards.template_id
        WHERE card_entities.deleted_at IS NULL
            AND concept_entities.deleted_at IS NULL
            AND concepts.archived_at IS NULL
            AND (
                cards.template_id IS NULL
                OR template_entities.deleted_at IS NULL
            )
            AND card_scheduling.due_at > ?1",
        [now],
        |row| row.get(0),
    )?;
    let mut statement = connection.prepare(
        "SELECT
            cards.entity_id,
            concepts.entity_id,
            concepts.last_change_id,
            concepts.title,
            concepts.content_json,
            cards.retrieval_kind,
            cards.configuration_json,
            templates.entity_id,
            templates.name,
            templates.content_json,
            card_scheduling.state,
            card_scheduling.due_at
        FROM card_scheduling
        INNER JOIN cards
            ON cards.entity_id = card_scheduling.card_id
        INNER JOIN entities AS card_entities
            ON card_entities.id = cards.entity_id
        INNER JOIN concepts
            ON concepts.entity_id = cards.concept_id
        INNER JOIN entities AS concept_entities
            ON concept_entities.id = concepts.entity_id
        LEFT JOIN templates
            ON templates.entity_id = cards.template_id
        LEFT JOIN entities AS template_entities
            ON template_entities.id = cards.template_id
        WHERE card_entities.deleted_at IS NULL
            AND concept_entities.deleted_at IS NULL
            AND concepts.archived_at IS NULL
            AND (
                cards.template_id IS NULL
                OR template_entities.deleted_at IS NULL
            )
            AND card_scheduling.due_at <= ?1
        ORDER BY card_scheduling.due_at, cards.entity_id",
    )?;
    let rows = statement.query_map([now], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, String>(6)?,
            row.get::<_, Option<String>>(7)?,
            row.get::<_, Option<String>>(8)?,
            row.get::<_, Option<String>>(9)?,
            row.get::<_, String>(10)?,
            row.get::<_, i64>(11)?,
        ))
    })?;
    let card_rows = rows.collect::<Result<Vec<_>, _>>()?;

    drop(statement);

    let mut media_concept_ids = BTreeSet::new();
    let cards = card_rows
        .into_iter()
        .map(|row| {
            let (
                id,
                concept_id,
                concept_last_change_id,
                concept_title,
                content,
                retrieval_kind,
                configuration,
                template_id,
                template_name,
                template_content,
                state,
                due_at,
            ) = row;
            let template = match (template_id, template_name, template_content) {
                (Some(id), Some(name), Some(content)) => Some(StudyTemplate {
                    id,
                    name,
                    content: serde_json::from_str(&content)?,
                }),
                (None, None, None) => None,
                _ => return Err(LibraryError::InvalidRetrievalForm),
            };
            let retrieval_kind = RetrievalFormKind::try_from(retrieval_kind.as_str())?;
            let parsed = parse_retrieval_form_configuration(retrieval_kind, &configuration)?;

            if template
                .as_ref()
                .is_some_and(|template| template.content.mode == TemplateMode::Custom)
                || retrieval_kind == RetrievalFormKind::ImageOcclusion
            {
                media_concept_ids.insert(concept_id.clone());
            }

            Ok(StudyCard {
                id,
                concept_id,
                concept_last_change_id,
                concept_title,
                content: serde_json::from_str(&content)?,
                retrieval_kind,
                cloze: parsed.cloze,
                image_occlusion: parsed.image_occlusion,
                type_answer: parsed.type_answer,
                template,
                scheduling_state: SchedulingState::try_from(state.as_str())?,
                due_at,
            })
        })
        .collect::<LibraryResult<Vec<_>>>()?;
    let media = query_media_for_concepts(connection, &media_concept_ids)?;

    Ok(StudyQueue {
        cards,
        media,
        next_due_at,
        total_cards,
    })
}

pub fn query_scheduling_settings(
    connection: &Connection,
) -> LibraryResult<SchedulingSettings> {
    let configuration = query_scheduler_configuration(connection)?;

    Ok(scheduling_settings(&configuration))
}

pub fn update_scheduling_settings(
    transaction: &WriteTransaction<'_>,
    input: UpdateSchedulingSettingsInput,
) -> LibraryResult<SchedulingSettings> {
    validate_scheduling_settings(&input)?;

    let current = query_scheduler_configuration(transaction)?;

    if current.desired_retention == input.desired_retention
        && current.maximum_interval_days == input.maximum_interval_days
    {
        return Ok(scheduling_settings(&current));
    }

    let configuration_id = Uuid::now_v7().hyphenated().to_string();
    let created_at = current_timestamp()?;

    transaction.execute(
        "INSERT INTO scheduler_configurations (
            id,
            algorithm,
            algorithm_version,
            parameters_json,
            desired_retention,
            created_at,
            maximum_interval_days
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            configuration_id,
            FSRS_ALGORITHM,
            current.algorithm_version,
            current.parameters_json,
            input.desired_retention,
            created_at,
            input.maximum_interval_days,
        ],
    )?;
    transaction.execute(
        "UPDATE active_scheduler_configuration
        SET configuration_id = ?1
        WHERE singleton = 1",
        [&configuration_id],
    )?;

    query_scheduling_settings(transaction)
}

pub fn record_review(
    transaction: &WriteTransaction<'_>,
    card_id: &str,
    rating: ReviewRating,
    now: i64,
) -> LibraryResult<ReviewOutcome> {
    let schedule = query_schedule(transaction, card_id)?;

    if schedule.due_at > now {
        return Err(LibraryError::CardNotDue {
            card_id: card_id.to_owned(),
            due_at: schedule.due_at,
        });
    }

    let configuration = query_scheduler_configuration(transaction)?;
    let elapsed_days = elapsed_days(schedule.last_reviewed_at, now);
    let memory_state = memory_state(&schedule)?;
    let next_states = configuration.scheduler.next_states(
        memory_state,
        configuration.desired_retention as f32,
        elapsed_days,
    )?;
    let next_state = select_next_state(next_states, rating);
    let scheduling_state = next_scheduling_state(schedule.state, rating);
    let scheduled_interval_days = f64::from(next_state.interval)
        .min(configuration.maximum_interval_days as f64);
    let due_at = calculate_due_at(now, scheduled_interval_days)?;
    let stability = f64::from(next_state.memory.stability);
    let difficulty = f64::from(next_state.memory.difficulty);
    let review_count = schedule
        .review_count
        .checked_add(1)
        .ok_or(LibraryError::InvalidSchedule)?;
    let lapse_count = schedule
        .lapse_count
        .checked_add(i64::from(
            schedule.state == SchedulingState::Review && rating == ReviewRating::Again,
        ))
        .ok_or(LibraryError::InvalidSchedule)?;
    let review = transaction.create_entity_at(EntityKind::Review, now)?;

    transaction.execute(
        "INSERT INTO reviews (
            entity_id,
            card_id,
            rating,
            reviewed_at,
            elapsed_days,
            scheduled_interval_days,
            state_before,
            state_after,
            stability,
            difficulty,
            due_at,
            scheduler_configuration_id,
            last_change_id
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13
        )",
        params![
            review.id,
            card_id,
            rating.value(),
            now,
            elapsed_days,
            scheduled_interval_days,
            schedule.state.as_str(),
            scheduling_state.as_str(),
            stability,
            difficulty,
            due_at,
            configuration.id,
            review.last_change_id,
        ],
    )?;
    transaction.execute(
        "UPDATE card_scheduling
        SET state = ?1,
            due_at = ?2,
            stability = ?3,
            difficulty = ?4,
            last_reviewed_at = ?5,
            last_review_id = ?6,
            last_reversal_id = NULL,
            review_count = ?7,
            lapse_count = ?8
        WHERE card_id = ?9",
        params![
            scheduling_state.as_str(),
            due_at,
            stability,
            difficulty,
            now,
            review.id,
            review_count,
            lapse_count,
            card_id,
        ],
    )?;

    Ok(ReviewOutcome {
        review_id: review.id,
        card_id: card_id.to_owned(),
        rating,
        scheduling_state,
        reviewed_at: now,
        due_at,
        scheduled_interval_days,
    })
}

pub fn reverse_review(
    transaction: &WriteTransaction<'_>,
    review_id: &str,
    now: i64,
) -> LibraryResult<ReviewReversalOutcome> {
    if let Some(reversal) = query_review_reversal(transaction, review_id)? {
        return Ok(reversal);
    }

    let review = transaction
        .query_row(
            "SELECT card_id, reviewed_at
            FROM reviews
            WHERE entity_id = ?1",
            [review_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
        )
        .optional()?;
    let Some((card_id, reviewed_at)) = review else {
        return Err(LibraryError::ReviewNotFound(review_id.to_owned()));
    };
    let schedule = query_schedule(transaction, &card_id)?;

    if schedule.last_review_id.as_deref() != Some(review_id) {
        return Err(LibraryError::ReviewNotReversible);
    }

    let reversed_at = now.max(reviewed_at);
    let reversal = transaction.create_entity_at(EntityKind::ReviewReversal, reversed_at)?;

    transaction.execute(
        "INSERT INTO review_reversals (
            entity_id,
            review_id,
            card_id,
            reversed_at,
            last_change_id
        ) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            reversal.id,
            review_id,
            card_id,
            reversed_at,
            reversal.last_change_id,
        ],
    )?;

    Ok(ReviewReversalOutcome {
        reversal_id: reversal.id,
        review_id: review_id.to_owned(),
        card_id,
        reversed_at,
    })
}

fn query_review_reversal(
    connection: &Connection,
    review_id: &str,
) -> LibraryResult<Option<ReviewReversalOutcome>> {
    Ok(connection
        .query_row(
            "SELECT entity_id, review_id, card_id, reversed_at
            FROM review_reversals
            WHERE review_id = ?1",
            [review_id],
            |row| {
                Ok(ReviewReversalOutcome {
                    reversal_id: row.get(0)?,
                    review_id: row.get(1)?,
                    card_id: row.get(2)?,
                    reversed_at: row.get(3)?,
                })
            },
        )
        .optional()?)
}

fn query_schedule(
    connection: &Connection,
    card_id: &str,
) -> LibraryResult<StoredSchedule> {
    let result = connection.query_row(
        "SELECT
            card_scheduling.state,
            card_scheduling.due_at,
            card_scheduling.stability,
            card_scheduling.difficulty,
            card_scheduling.last_reviewed_at,
            card_scheduling.last_review_id,
            card_scheduling.review_count,
            card_scheduling.lapse_count
        FROM card_scheduling
        INNER JOIN cards
            ON cards.entity_id = card_scheduling.card_id
        INNER JOIN entities AS card_entities
            ON card_entities.id = cards.entity_id
        INNER JOIN concepts
            ON concepts.entity_id = cards.concept_id
        INNER JOIN entities AS concept_entities
            ON concept_entities.id = concepts.entity_id
        WHERE card_scheduling.card_id = ?1
            AND card_entities.deleted_at IS NULL
            AND concept_entities.deleted_at IS NULL
            AND concepts.archived_at IS NULL",
        [card_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, Option<f64>>(2)?,
                row.get::<_, Option<f64>>(3)?,
                row.get::<_, Option<i64>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, i64>(7)?,
            ))
        },
    );

    match result {
        Ok((
            state,
            due_at,
            stability,
            difficulty,
            last_reviewed_at,
            last_review_id,
            review_count,
            lapse_count,
        )) => {
            Ok(StoredSchedule {
                state: SchedulingState::try_from(state.as_str())?,
                due_at,
                stability,
                difficulty,
                last_reviewed_at,
                last_review_id,
                review_count,
                lapse_count,
            })
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            Err(LibraryError::CardNotFound(card_id.to_owned()))
        }
        Err(error) => Err(error.into()),
    }
}

fn query_scheduler_configuration(
    connection: &Connection,
) -> LibraryResult<SchedulerConfiguration> {
    let (
        id,
        algorithm,
        algorithm_version,
        parameters_json,
        desired_retention,
        maximum_interval_days,
    ) = connection.query_row(
        "SELECT
            scheduler_configurations.id,
            scheduler_configurations.algorithm,
            scheduler_configurations.algorithm_version,
            scheduler_configurations.parameters_json,
            scheduler_configurations.desired_retention,
            scheduler_configurations.maximum_interval_days
        FROM active_scheduler_configuration
        INNER JOIN scheduler_configurations
            ON scheduler_configurations.id = configuration_id
        WHERE singleton = 1",
        [],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, f64>(4)?,
                row.get::<_, i64>(5)?,
            ))
        },
    )?;

    if algorithm != FSRS_ALGORITHM || algorithm_version != FSRS_VERSION {
        return Err(LibraryError::UnsupportedSchedulerConfiguration(id));
    }

    let parameters: Vec<f32> = serde_json::from_str(&parameters_json)?;
    let scheduler = FSRS::new(&parameters)?;

    Ok(SchedulerConfiguration {
        id,
        algorithm_version,
        parameters_json,
        desired_retention,
        maximum_interval_days,
        scheduler,
    })
}

fn scheduling_settings(configuration: &SchedulerConfiguration) -> SchedulingSettings {
    SchedulingSettings {
        algorithm_version: configuration.algorithm_version.clone(),
        desired_retention: configuration.desired_retention,
        maximum_interval_days: configuration.maximum_interval_days,
    }
}

fn validate_scheduling_settings(input: &UpdateSchedulingSettingsInput) -> LibraryResult<()> {
    if !input.desired_retention.is_finite()
        || input.desired_retention < MINIMUM_DESIRED_RETENTION
        || input.desired_retention > MAXIMUM_DESIRED_RETENTION
    {
        return Err(LibraryError::InvalidDesiredRetention {
            minimum: (MINIMUM_DESIRED_RETENTION * 100.0) as i64,
            maximum: (MAXIMUM_DESIRED_RETENTION * 100.0) as i64,
        });
    }

    if !(MINIMUM_INTERVAL_DAYS..=MAXIMUM_INTERVAL_DAYS)
        .contains(&input.maximum_interval_days)
    {
        return Err(LibraryError::InvalidMaximumInterval {
            minimum: MINIMUM_INTERVAL_DAYS,
            maximum: MAXIMUM_INTERVAL_DAYS,
        });
    }

    Ok(())
}

fn memory_state(schedule: &StoredSchedule) -> LibraryResult<Option<MemoryState>> {
    match (schedule.stability, schedule.difficulty) {
        (None, None) => Ok(None),
        (Some(stability), Some(difficulty)) => {
            let memory = MemoryState {
                stability: stability as f32,
                difficulty: difficulty as f32,
            };

            if memory.stability.is_finite() && memory.difficulty.is_finite() {
                Ok(Some(memory))
            } else {
                Err(LibraryError::InvalidSchedulingState(
                    schedule.state.as_str().to_owned(),
                ))
            }
        }
        _ => Err(LibraryError::InvalidSchedulingState(
            schedule.state.as_str().to_owned(),
        )),
    }
}

fn elapsed_days(last_reviewed_at: Option<i64>, now: i64) -> u32 {
    let Some(last_reviewed_at) = last_reviewed_at else {
        return 0;
    };
    let elapsed = now.saturating_sub(last_reviewed_at) / MILLISECONDS_PER_DAY;

    u32::try_from(elapsed).unwrap_or(u32::MAX)
}

fn select_next_state(next_states: fsrs::NextStates, rating: ReviewRating) -> ItemState {
    match rating {
        ReviewRating::Again => next_states.again,
        ReviewRating::Hard => next_states.hard,
        ReviewRating::Good => next_states.good,
        ReviewRating::Easy => next_states.easy,
    }
}

fn next_scheduling_state(
    current: SchedulingState,
    rating: ReviewRating,
) -> SchedulingState {
    match (current, rating) {
        (SchedulingState::New | SchedulingState::Learning, ReviewRating::Again) => {
            SchedulingState::Learning
        }
        (SchedulingState::Review | SchedulingState::Relearning, ReviewRating::Again) => {
            SchedulingState::Relearning
        }
        (_, _) => SchedulingState::Review,
    }
}

fn calculate_due_at(now: i64, interval_days: f64) -> LibraryResult<i64> {
    let interval_milliseconds = interval_days * MILLISECONDS_PER_DAY as f64;

    if !interval_days.is_finite()
        || interval_days <= 0.0
        || !interval_milliseconds.is_finite()
        || interval_milliseconds > i64::MAX as f64
    {
        return Err(LibraryError::InvalidSchedule);
    }

    let interval_milliseconds = interval_milliseconds.round() as i64;

    if interval_milliseconds <= 0 {
        return Err(LibraryError::InvalidSchedule);
    }

    now.checked_add(interval_milliseconds)
        .ok_or(LibraryError::InvalidSchedule)
}

impl ReviewRating {
    const fn value(self) -> i64 {
        self as i64
    }
}

impl SchedulingState {
    const fn as_str(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Learning => "learning",
            Self::Review => "review",
            Self::Relearning => "relearning",
        }
    }
}

impl TryFrom<&str> for SchedulingState {
    type Error = LibraryError;

    fn try_from(value: &str) -> LibraryResult<Self> {
        match value {
            "new" => Ok(Self::New),
            "learning" => Ok(Self::Learning),
            "review" => Ok(Self::Review),
            "relearning" => Ok(Self::Relearning),
            _ => Err(LibraryError::InvalidSchedulingState(value.to_owned())),
        }
    }
}
