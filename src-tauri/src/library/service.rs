use crate::data::{current_timestamp, LocalDataStore};
use crate::library::preferences::{
    query_device_preferences, update_appearance_preferences, update_grading_mode,
    update_mixed_practice_enabled, update_pretesting_enabled, update_startup_destination,
};
use crate::library::pretesting::record_pretest;
use crate::library::study::{
    query_scheduling_settings, query_study_queue, record_review, reverse_review,
    update_scheduling_settings,
};
use crate::library::{
    AppearancePreferences, ConceptDetail, DevicePreferences, GradingMode, LibraryError,
    LibraryResult, MediaSummary, PretestRecord, RecordPretestInput,
    RecordReviewInput, ReverseReviewInput, ReviewOutcome, ReviewReversalOutcome,
    SchedulingSettings, StartupDestination, StudyQueue, UpdateSchedulingSettingsInput,
};

mod assignments;
mod cards;
mod concepts;
mod organizations;
mod search;

#[cfg(test)]
mod tests;

pub(super) use concepts::{create_concept, update_concept};
use concepts::query_concept;

pub struct ConceptLibrary<'store> {
    store: &'store LocalDataStore,
}

impl<'store> ConceptLibrary<'store> {
    pub fn new(store: &'store LocalDataStore) -> Self {
        Self { store }
    }

    pub fn concept(&self, id: &str) -> LibraryResult<ConceptDetail> {
        self.store
            .read_result(|connection| query_concept(connection, id))
    }

    pub fn study_queue(&self) -> LibraryResult<StudyQueue> {
        self.study_queue_at(current_timestamp()?)
    }

    pub fn record_review(&self, input: RecordReviewInput) -> LibraryResult<ReviewOutcome> {
        self.record_review_at(input, current_timestamp()?)
    }

    pub fn record_pretest(&self, input: RecordPretestInput) -> LibraryResult<PretestRecord> {
        self.record_pretest_at(input, current_timestamp()?)
    }

    pub fn reverse_review(
        &self,
        input: ReverseReviewInput,
    ) -> LibraryResult<ReviewReversalOutcome> {
        self.reverse_review_at(input, current_timestamp()?)
    }

    pub fn device_preferences(&self) -> LibraryResult<DevicePreferences> {
        self.store.read_result(query_device_preferences)
    }

    pub fn set_grading_mode(&self, grading_mode: GradingMode) -> LibraryResult<DevicePreferences> {
        self.store
            .write_result(|transaction| update_grading_mode(transaction, grading_mode))
    }

    pub fn set_pretesting_enabled(&self, enabled: bool) -> LibraryResult<DevicePreferences> {
        self.store
            .write_result(|transaction| update_pretesting_enabled(transaction, enabled))
    }

    pub fn set_mixed_practice_enabled(&self, enabled: bool) -> LibraryResult<DevicePreferences> {
        self.store
            .write_result(|transaction| update_mixed_practice_enabled(transaction, enabled))
    }

    pub fn set_startup_destination(
        &self,
        startup_destination: StartupDestination,
    ) -> LibraryResult<DevicePreferences> {
        self.store.write_result(|transaction| {
            update_startup_destination(transaction, startup_destination)
        })
    }

    pub fn set_appearance_preferences(
        &self,
        appearance: AppearancePreferences,
    ) -> LibraryResult<DevicePreferences> {
        self.store
            .write_result(|transaction| update_appearance_preferences(transaction, appearance))
    }

    pub fn scheduling_settings(&self) -> LibraryResult<SchedulingSettings> {
        self.store.read_result(query_scheduling_settings)
    }

    pub fn update_scheduling_settings(
        &self,
        input: UpdateSchedulingSettingsInput,
    ) -> LibraryResult<SchedulingSettings> {
        self.store
            .write_result(|transaction| update_scheduling_settings(transaction, input))
    }

    pub fn import_image(&self, bytes: &[u8]) -> LibraryResult<MediaSummary> {
        crate::library::media::import_image(self.store, bytes)
    }

    pub fn media_bytes(&self, id: &str) -> LibraryResult<Vec<u8>> {
        crate::library::media::read_media(self.store, id)
    }

    fn study_queue_at(&self, now: i64) -> LibraryResult<StudyQueue> {
        self.store
            .read_result(|connection| query_study_queue(connection, now))
    }

    fn record_review_at(&self, input: RecordReviewInput, now: i64) -> LibraryResult<ReviewOutcome> {
        let card_id = input.card_id.trim().to_owned();

        self.store
            .write_result(|transaction| record_review(transaction, &card_id, input.rating, now))
    }

    fn record_pretest_at(
        &self,
        input: RecordPretestInput,
        now: i64,
    ) -> LibraryResult<PretestRecord> {
        let card_id = input.card_id.trim().to_owned();

        self.store
            .write_result(|transaction| record_pretest(transaction, &card_id, input.outcome, now))
    }

    fn reverse_review_at(
        &self,
        input: ReverseReviewInput,
        now: i64,
    ) -> LibraryResult<ReviewReversalOutcome> {
        let review_id = input.review_id.trim().to_owned();

        self.store
            .write_result(|transaction| reverse_review(transaction, &review_id, now))
    }
}

fn normalize_value(value: String, field: &'static str, maximum: usize) -> LibraryResult<String> {
    let value = value.trim().to_owned();

    if value.is_empty() {
        return Err(LibraryError::EmptyValue { field });
    }

    if value.chars().count() > maximum {
        return Err(LibraryError::ValueTooLong { field, maximum });
    }

    Ok(value)
}
