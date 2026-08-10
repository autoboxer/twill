use fsrs::{ItemState, MemoryState, FSRS};
use rusqlite::{params, Connection};

use crate::data::{EntityKind, WriteTransaction};
use crate::library::{
    CardSummary, GradingMode, LibraryError, LibraryResult, ReviewOutcome, ReviewRating,
    SchedulingState, StudyCard, StudyPreferences, StudyQueue,
};

const FSRS_ALGORITHM: &str = "fsrs";
const FSRS_VERSION: &str = "6.6.1";
const MILLISECONDS_PER_DAY: i64 = 86_400_000;

struct StoredSchedule {
    state: SchedulingState,
    due_at: i64,
    stability: Option<f64>,
    difficulty: Option<f64>,
    last_reviewed_at: Option<i64>,
    review_count: i64,
    lapse_count: i64,
}

struct SchedulerConfiguration {
    id: String,
    desired_retention: f32,
    scheduler: FSRS,
}

pub fn create_recall_card(
    transaction: &WriteTransaction<'_>,
    concept_id: &str,
) -> LibraryResult<CardSummary> {
    let entity = transaction.create_entity(EntityKind::Card)?;

    transaction.execute(
        "INSERT INTO cards (entity_id, concept_id, last_change_id)
        VALUES (?1, ?2, ?3)",
        params![entity.id, concept_id, entity.last_change_id],
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

    Ok(CardSummary { id: entity.id })
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
        WHERE card_entities.deleted_at IS NULL
            AND concept_entities.deleted_at IS NULL
            AND concepts.archived_at IS NULL",
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
        WHERE card_entities.deleted_at IS NULL
            AND concept_entities.deleted_at IS NULL
            AND concepts.archived_at IS NULL
            AND card_scheduling.due_at > ?1",
        [now],
        |row| row.get(0),
    )?;
    let mut statement = connection.prepare(
        "SELECT
            cards.entity_id,
            concepts.entity_id,
            concepts.title,
            concepts.content_json,
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
        WHERE card_entities.deleted_at IS NULL
            AND concept_entities.deleted_at IS NULL
            AND concepts.archived_at IS NULL
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
            row.get::<_, i64>(5)?,
        ))
    })?;
    let cards = rows
        .map(|row| {
            let (id, concept_id, concept_title, content, state, due_at) = row?;

            Ok(StudyCard {
                id,
                concept_id,
                concept_title,
                content: serde_json::from_str(&content)?,
                scheduling_state: SchedulingState::try_from(state.as_str())?,
                due_at,
            })
        })
        .collect::<LibraryResult<Vec<_>>>()?;

    Ok(StudyQueue {
        cards,
        next_due_at,
        total_cards,
    })
}

pub fn query_study_preferences(connection: &Connection) -> LibraryResult<StudyPreferences> {
    let grading_mode: String = connection.query_row(
        "SELECT grading_mode
        FROM device_preferences
        WHERE singleton = 1",
        [],
        |row| row.get(0),
    )?;

    Ok(StudyPreferences {
        grading_mode: GradingMode::try_from(grading_mode.as_str())?,
    })
}

pub fn update_grading_mode(
    transaction: &WriteTransaction<'_>,
    grading_mode: GradingMode,
) -> LibraryResult<StudyPreferences> {
    transaction.execute(
        "UPDATE device_preferences
        SET grading_mode = ?1
        WHERE singleton = 1
            AND grading_mode != ?1",
        [grading_mode.as_str()],
    )?;

    query_study_preferences(transaction)
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
        configuration.desired_retention,
        elapsed_days,
    )?;
    let next_state = select_next_state(next_states, rating);
    let scheduling_state = next_scheduling_state(schedule.state, rating);
    let scheduled_interval_days = f64::from(next_state.interval);
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
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
            ))
        },
    );

    match result {
        Ok((state, due_at, stability, difficulty, last_reviewed_at, review_count, lapse_count)) => {
            Ok(StoredSchedule {
                state: SchedulingState::try_from(state.as_str())?,
                due_at,
                stability,
                difficulty,
                last_reviewed_at,
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
    let (id, algorithm, algorithm_version, parameters_json, desired_retention) = connection
        .query_row(
            "SELECT
                scheduler_configurations.id,
                scheduler_configurations.algorithm,
                scheduler_configurations.algorithm_version,
                scheduler_configurations.parameters_json,
                scheduler_configurations.desired_retention
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
                ))
            },
        )?;

    if algorithm != FSRS_ALGORITHM || algorithm_version != FSRS_VERSION {
        return Err(LibraryError::UnsupportedSchedulerConfiguration(id));
    }

    let parameters: Vec<f32> = serde_json::from_str(&parameters_json)?;
    let desired_retention = desired_retention as f32;
    let scheduler = FSRS::new(&parameters)?;

    Ok(SchedulerConfiguration {
        id,
        desired_retention,
        scheduler,
    })
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

impl GradingMode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Simple => "simple",
            Self::Advanced => "advanced",
        }
    }
}

impl TryFrom<&str> for GradingMode {
    type Error = LibraryError;

    fn try_from(value: &str) -> LibraryResult<Self> {
        match value {
            "simple" => Ok(Self::Simple),
            "advanced" => Ok(Self::Advanced),
            _ => Err(LibraryError::InvalidGradingMode(value.to_owned())),
        }
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
