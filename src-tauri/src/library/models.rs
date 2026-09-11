use serde::Deserialize;

mod authoring;
mod card_quality;
mod concepts;
mod css_snippets;
mod preferences;
mod retrieval;
mod study;
mod templates;

pub use authoring::{
    AuthoringDraft, AuthoringDraftKind, AuthoringDraftLocator, AuthoringDraftTargetStatus,
    AuthoringSaveContext, DeferredConceptEdit, DeferredEditQueue, DeferredEditTargetStatus,
    FinalizeConceptInput, FinalizeTemplateInput, QueueDeferredEditInput, UpsertAuthoringDraftInput,
    AUTHORING_DRAFT_SCHEMA_VERSION,
};
pub use card_quality::{
    CardQualityConcern, CardQualityDisposition, CardQualityEvidence, CardQualityKind,
    CardQualityQueue, CardQualityQueueItem, CardQualitySignal, CardQualitySource,
    CardQualityStatus, CloseCardQualityConcernInput, CreateCardQualityConcernInput,
    DismissCardQualitySignalInput,
};
pub use concepts::{
    ConceptContent, ConceptDetail, ConceptSummary, CreateConceptInput, CreateNamedItemInput,
    LibraryCardMatch, LibraryCardState, LibraryOrganizations, LibraryPage, LibraryQuery, LibrarySort,
    MediaSummary, NamedItem, OrganizationSummary,
    RenameNamedItemInput, SetConceptArchivedInput, UpdateConceptInput, RICH_CONTENT_SCHEMA_VERSION,
};
pub use css_snippets::{
    CreateCssSnippetInput, CssSnippet, CssSnippetCatalog, CssSnippetContent,
    SetCssSnippetEnabledInput, UpdateCssSnippetInput, CSS_SNIPPET_SCHEMA_VERSION,
};
pub use preferences::{
    AppearancePreferences, AppearanceTheme, DevicePreferences, GradingMode, MotionPreference,
    ReadingFont, ReadingTextSize, SetAppearancePreferencesInput, SetGradingModeInput,
    SetMixedPracticeEnabledInput, SetPretestingEnabledInput, SetStartupDestinationInput,
    StartupDestination,
};
pub use retrieval::{
    CardSummary, ClozeSettings, ExplainSettings, ImageOcclusionSettings, ProblemSettings,
    RetrievalFormKind, TypeAnswerSettings,
};
pub use study::{
    PretestOutcome, PretestRecord, RecordPretestInput, RecordReviewInput, ReverseReviewInput,
    ReviewOutcome, ReviewRating, ReviewReversalOutcome, SchedulingSettings, SchedulingState,
    StudyCard, StudyConcept, StudyQueue, StudyTemplate, UpdateSchedulingSettingsInput,
};
pub use templates::{
    CreateTemplateInput, TemplateBlock, TemplateCatalog, TemplateContent, TemplateDetail,
    TemplateMode, TemplateSummary, UpdateTemplateInput, TEMPLATE_SCHEMA_VERSION,
};

#[cfg(test)]
pub use concepts::AnswerFeedback;
#[cfg(test)]
pub use retrieval::ExplainFocus;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EntityIdInput {
    pub id: String,
}
