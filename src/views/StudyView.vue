<script setup>
import { m } from 'motion-v';
import { computed, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';

import CardQualityAction from '../components/CardQualityAction.vue';
import ContentState from '../components/ContentState.vue';
import DeferredEditQueue from '../components/DeferredEditQueue.vue';
import DeferredEditNoteDialog from '../components/DeferredEditNoteDialog.vue';
import ExplainResponse from '../components/ExplainResponse.vue';
import ProblemResponse from '../components/ProblemResponse.vue';
import StudyAnswerFeedback from '../components/StudyAnswerFeedback.vue';
import StudyAnswerParts from '../components/StudyAnswerParts.vue';
import StudyAssistance from '../components/StudyAssistance.vue';
import StudyCardContent from '../components/StudyCardContent.vue';
import StudyHelpButton from '../components/StudyHelpButton.vue';
import StudySessionBuilder from '../components/StudySessionBuilder.vue';
import StudySessionControls from '../components/StudySessionControls.vue';
import TypeAnswerResponse from '../components/TypeAnswerResponse.vue';
import { COMMAND_IDS } from '../commands/registry';
import {
  useCommandHandler,
  useCommands
} from '../composables/useCommands';
import { useStudySession } from '../composables/useStudySession';
import { useStudyFocus } from '../composables/useStudyFocus';
import { useStudyDeferredEdits } from '../composables/useStudyDeferredEdits';
import { useAppearance } from '../composables/useAppearance';
import { useStartupReady } from '../composables/useStartupReady';
import { gradingModeItems, gradingOptionsByMode } from '../study/grading';
import { emptyStudySelection, studySelectionFromLibrary } from '../study/selection';
import { retrievalFormLabel as studyCardName } from '../retrieval-forms/catalog';
import { supportsAnswerParts } from '../answer-parts/documents';
import { usesAnswerPartComparison } from '../study/comparison';

const commands = useCommands();
const { resolvedMotion } = useAppearance();
const route = useRoute();
const router = useRouter();
const builderOpen = ref( false );
const builderSelection = ref( emptyStudySelection() );
const session = useStudySession({
  onStateChanged: () => focusCurrentState(),
  onAnswerRevealed: () => focusAnswer(),
  onFeedbackContinued: () => focusFirstGradingAction()
});
const {
  actionsBlocked,
  answerFeedbackPending,
  answerRevealed,
  assisted,
  assessmentError,
  assessmentPending,
  beginMasteryRound,
  canRevealAnswer,
  canUndoLastGrade,
  completedCount,
  comparisonChecks,
  continueToGrading,
  correctionPending,
  currentAnswerFeedback,
  currentCard,
  endSession,
  explainSettings,
  finishCurrentPretest,
  focusedSession,
  gradingMode,
  gradingModeError,
  gradingModeLocked,
  gradingModePending,
  hasCards,
  initialLoading,
  isComplete,
  loadError,
  loadStudyQueue,
  masteryActionEnabled,
  masteryActive,
  masteryCompletedCount,
  masteryMissedCount,
  masteryReady,
  masteryRecalledCount,
  masteryStarted,
  masteryTotal,
  mixedPracticeEnabled,
  nextDueAt,
  navigationNotice,
  pauseSession,
  pendingAssessment,
  pendingPretestOutcome,
  position,
  pretestActive,
  pretestAttemptedCount,
  pretestPending,
  pretestSkippedCount,
  pretestTeachingActive,
  pretestTotal,
  problemSettings,
  ratingCounts,
  recordAssessment,
  recordMasteryAssessment,
  recoveryError,
  resumeSession,
  responseBeforeAssistance,
  revealedAssistance,
  revealedAnswerParts,
  sessionBusy,
  sessionEnded,
  sessionPaused,
  sessionGradingMode,
  selectedCardCount,
  sessionResumeNotice,
  sessionSelection,
  setComparison,
  showAnswer,
  skipCurrentPretest,
  studyMedia,
  studyResponse,
  totalAvailableCards,
  totalCards,
  toggleAssistance,
  toggleAnswerParts,
  typeAnswerSettings,
  undoLastGrade,
  undoPending,
  updateGradingMode
} = session;
const {
  answerFeedback,
  completionHeading,
  gradingActions,
  masteryHeading,
  problemResponse,
  explainResponse,
  revealButton,
  studyContent,
  typeAnswerResponse,
  focusCurrentState,
  focusAnswer,
  focusFirstGradingAction
} = useStudyFocus( session );
const {
  canEditCurrentConcept,
  deferredEdits,
  deferredError,
  deferredLoading,
  deferredPendingConceptId,
  deferredStartPending,
  immediateEditError,
  noteError,
  notePending,
  noteTarget,
  closeQueuedNote,
  editCurrentConcept,
  editCurrentNote,
  currentConceptQueued,
  canQueueCurrentConcept,
  queueCurrentConcept,
  removeQueuedConcept,
  openQueuedNote,
  saveQueuedNote,
  startDeferredEditing
} = useStudyDeferredEdits( session );

const builderDisabled = computed( () => sessionBusy.value || deferredStartPending.value );
const selectiveAnswer = computed( () => supportsAnswerParts( currentCard.value ) );
const comparisonResponse = computed( () => (
  answerRevealed.value && assisted.value ? responseBeforeAssistance.value : studyResponse.value
) );
const writtenComparison = computed( () => (
  Boolean( typeAnswerSettings.value || comparisonResponse.value.trim() )
) );
const comparisonGuidance = computed( () => {
  if ( !answerRevealed.value ) {
    return '';
  }

  const judgments = Object.values( comparisonChecks.value );

  if ( judgments.includes( 'missing' ) || judgments.includes( 'partial' ) ) {
    if ( pretestTeachingActive.value ) {
      return 'Review any required ideas you missed or only partly recalled before continuing.';
    }

    const grade = masteryActive.value ? 'Still missed' : gradingMode.value === 'advanced' ? 'Again' : 'Forgot';

    return `Choose ${ grade } if a required idea was missing or incomplete.`;
  }

  return '';
});

watch( () => route.fullPath, () => {
  if ( route.name === 'study' && route.query.build === '1' ) {
    builderSelection.value = studySelectionFromLibrary( route.query );
    builderOpen.value = true;
  }
}, { immediate: true });

watch( builderOpen, ( open ) => {
  if ( !open && route.name === 'study' && route.query.build === '1' ) {
    void router.replace({ name: 'study' });
  }
});

function openBuilder() {
  builderSelection.value = { ...sessionSelection.value };
  builderOpen.value = true;
}

function startFocusedSession( selection ) {
  return loadStudyQueue( selection, true );
}

const cardTransition = {
  duration: 0.12,
  ease: [ 0.22, 1, 0.36, 1 ]
};

const gradingOptions = computed( () => {
  return gradingOptionsForMode( gradingMode.value );
});

const completedReviewCount = computed( () => (
  Object.values( ratingCounts.value ).reduce( ( total, count ) => total + count, 0 )
) );

const completionDescription = computed( () => {
  return pretestAttemptedCount.value
    ? 'Attempted concepts will return as ordinary reviews in a later session.'
    : '';
});

const masteryOptions = computed( () => [
  {
    color: 'error',
    command: commands.command( COMMAND_IDS.studyMasteryMissed ),
    icon: 'i-lucide-rotate-ccw',
    outcome: 'missed',
    recalled: false,
    variant: 'subtle'
  },

  {
    color: 'primary',
    command: commands.command( COMMAND_IDS.studyMasteryRecalled ),
    icon: 'i-lucide-check',
    outcome: 'recalled',
    recalled: true,
    variant: 'subtle'
  }
].map( ( option ) => ({
  ...option,
  label: option.command.label,
  shortcut: option.command.shortcutLabel
}) ) );

const masteryPhase = computed( () => (
  masteryReady.value || masteryStarted.value
) );

const visibleCompletedCount = computed( () => (
  masteryPhase.value ? masteryCompletedCount.value : completedCount.value
) );

const visibleTotalCards = computed( () => (
  masteryPhase.value ? masteryTotal.value : totalCards.value
) );

const visibleProgress = computed( () => {
  if ( !visibleTotalCards.value ) {
    return 0;
  }

  return visibleCompletedCount.value / visibleTotalCards.value * 100;
});

const progressLabel = computed( () => {
  if ( isComplete.value ) {
    return 'Complete';
  }

  if ( masteryReady.value ) {
    return 'Mastery round ready';
  }

  if ( masteryActive.value ) {
    return `Retry ${ masteryCompletedCount.value + 1 } / ${ masteryTotal.value }`;
  }

  if ( pretestActive.value || pretestTeachingActive.value ) {
    return `Pretest ${ position.value } / ${ totalCards.value }`;
  }

  return `Card ${ position.value } / ${ totalCards.value }`;
});

const revealActionLabel = computed( () => {
  if ( typeAnswerSettings.value ) {
    return 'Check answer';
  }

  if ( explainSettings.value ) {
    return 'Compare explanation';
  }

  if ( problemSettings.value ) {
    return 'Check solution';
  }

  return 'Reveal answer';
});

const sessionResultItems = computed( () => {
  if ( sessionGradingMode.value === 'simple' ) {
    return [
      { label: 'Remembered', rating: 'good' },
      { label: 'Forgot', rating: 'again' }
    ];
  }

  return gradingOptionsForMode( 'advanced' ).map( ( option ) => ({
    label: option.label,
    rating: option.rating
  }) );
});

const nextReviewDescription = computed( () => {
  if ( nextDueAt.value === null ) {
    return 'No reviews are currently due.';
  }

  const formattedTime = new Intl.DateTimeFormat( undefined, {
    dateStyle: 'medium',
    timeStyle: 'short'
  }).format( new Date( nextDueAt.value ) );

  return `Next review: ${ formattedTime }`;
});

const revealCommand = useCommandHandler( COMMAND_IDS.studyReveal, {
  enabled: computed( () => (
    Boolean( currentCard.value )
    && !initialLoading.value
    && !answerRevealed.value
    && !assessmentPending.value
    && !gradingModePending.value
    && !pretestPending.value
    && !undoPending.value
    && canRevealAnswer.value
  ) ),
  execute: showAnswer
});

const undoCommand = useCommandHandler( COMMAND_IDS.studyUndoLastGrade, {
  enabled: canUndoLastGrade,
  execute: undoLastGrade
});

const queueEditCommand = useCommandHandler( COMMAND_IDS.studyQueueEdit, {
  enabled: canQueueCurrentConcept,
  execute: queueCurrentConcept
});

registerGradingCommand(
  COMMAND_IDS.studyGradeSimpleForgot,
  'simple',
  'again'
);
registerGradingCommand(
  COMMAND_IDS.studyGradeSimpleRemembered,
  'simple',
  'good'
);
registerGradingCommand(
  COMMAND_IDS.studyGradeAdvancedAgain,
  'advanced',
  'again'
);
registerGradingCommand(
  COMMAND_IDS.studyGradeAdvancedHard,
  'advanced',
  'hard'
);
registerGradingCommand(
  COMMAND_IDS.studyGradeAdvancedGood,
  'advanced',
  'good'
);
registerGradingCommand(
  COMMAND_IDS.studyGradeAdvancedEasy,
  'advanced',
  'easy'
);

useCommandHandler( COMMAND_IDS.studyMasteryMissed, {
  enabled: computed( () => masteryActionEnabled() ),
  execute: () => recordMasteryAssessment( false )
});

useCommandHandler( COMMAND_IDS.studyMasteryRecalled, {
  enabled: computed( () => masteryActionEnabled() ),
  execute: () => recordMasteryAssessment( true )
});

function gradingOptionsForMode( mode ) {
  return gradingOptionsByMode[ mode ].map( ( option ) => {
    const command = commands.command( option.commandId );

    return {
      ...option,
      command,
      label: command.label,
      shortcut: command.shortcutLabel
    };
  });
}

function registerGradingCommand( commandId, mode, rating ) {
  useCommandHandler( commandId, {
    enabled: computed( () => (
      gradingMode.value === mode
      && !actionsBlocked.value
      && !masteryActive.value
      && !pretestTeachingActive.value
      && answerRevealed.value
      && !answerFeedbackPending.value
      && !assessmentPending.value
      && !gradingModePending.value
      && !undoPending.value
    ) ),
    execute: () => recordAssessment( rating )
  });
}

useStartupReady( initialLoading );
</script>

<template>
  <div
    class="page study-page"
    data-twill-page="study"
  >
    <header class="study-toolbar">
      <div class="study-toolbar__summary">
        <h1>Study</h1>
        <span v-if="hasCards && !initialLoading && !loadError && !sessionEnded">
          {{ progressLabel }}
        </span>
      </div>

      <div class="study-toolbar__actions">
        <div
          class="grading-mode-control"
          :title="gradingModeLocked
            ? 'Finish the current session to change grading mode.'
            : undefined"
        >
          <label for="grading-mode">Grading</label>

          <USelect
            id="grading-mode"
            :model-value="gradingMode"
            :items="gradingModeItems"
            :disabled="gradingModeLocked || actionsBlocked"
            :loading="gradingModePending"
            value-key="value"
            leading-icon="i-lucide-list-checks"
            size="sm"
            class="grading-mode-control__select"
            @update:model-value="updateGradingMode"
          />

          <StudyHelpButton topic="grading" label="Help with grading" />
        </div>

        <UTooltip text="Build session">
          <UButton
            leading-icon="i-lucide-list-filter"
            aria-label="Build session"
            color="neutral"
            variant="ghost"
            size="sm"
            :disabled="builderDisabled"
            @click="openBuilder"
          />
        </UTooltip>

        <UButton
          :to="{ name: 'library' }"
          leading-icon="i-lucide-library"
          color="neutral"
          variant="link"
          size="sm"
        >
          Library
        </UButton>
        <StudySessionControls
          v-if="hasCards && !initialLoading && !loadError && !sessionEnded"
          :busy="builderDisabled"
          :complete="isComplete"
          :paused="sessionPaused"
          @pause="pauseSession"
          @resume="resumeSession"
          @end="endSession"
        />
      </div>
    </header>

    <UAlert
      v-if="navigationNotice && sessionBusy"
      class="study-mode-error"
      :description="navigationNotice"
      color="neutral"
      variant="subtle"
      role="status"
    />

    <UAlert
      v-if="gradingModeError"
      class="study-mode-error"
      :description="gradingModeError"
      icon="i-lucide-circle-alert"
      color="error"
      variant="soft"
    />

    <UAlert
      v-if="sessionResumeNotice"
      class="study-mode-error"
      title="Session updated"
      :description="sessionResumeNotice"
      icon="i-lucide-history"
      color="primary"
      variant="subtle"
    />

    <UAlert
      v-if="deferredError"
      class="study-mode-error"
      title="Queued edits need attention"
      :description="deferredError"
      icon="i-lucide-circle-alert"
      color="error"
      variant="subtle"
    />

    <UAlert
      v-if="immediateEditError"
      class="study-mode-error"
      title="Editing could not be started"
      :description="immediateEditError"
      icon="i-lucide-circle-alert"
      color="error"
      variant="subtle"
    />

    <ContentState
      v-if="initialLoading"
      kind="loading"
      title="Loading study cards"
    />

    <ContentState
      v-else-if="loadError"
      kind="error"
      title="Study cards could not be loaded"
      :description="loadError"
    >
      <template #actions>
        <UButton
          leading-icon="i-lucide-refresh-cw"
          :disabled="gradingModePending"
          @click="loadStudyQueue()"
        >
          Retry
        </UButton>

        <UButton
          :to="{ name: 'library' }"
          color="neutral"
          variant="link"
        >
          Open library
        </UButton>
      </template>
    </ContentState>

    <ContentState
      v-else-if="sessionEnded"
      title="Session ended"
    >
      <template #actions>
        <UButton variant="subtle" :disabled="sessionBusy" @click="loadStudyQueue()">
          Start another session
        </UButton>
      </template>
    </ContentState>

    <ContentState
      v-else-if="sessionPaused"
      title="Paused"
      :description="`${ visibleCompletedCount } / ${ visibleTotalCards } ${ masteryPhase ? 'retries' : 'cards' } completed`"
    />

    <ContentState
      v-else-if="!hasCards && totalAvailableCards === 0 && !focusedSession"
      title="No cards to study"
      description="Create or restore a concept to make a study card available."
    >
      <template #actions>
        <UButton
          to="/create"
          leading-icon="i-lucide-square-pen"
          size="lg"
        >
          Create concept
        </UButton>

        <UButton
          :to="{ name: 'library' }"
          leading-icon="i-lucide-library"
          color="neutral"
          variant="link"
          size="lg"
        >
          Open library
        </UButton>
      </template>
    </ContentState>

    <ContentState
      v-else-if="!hasCards"
      :title="sessionResumeNotice ? 'No cards remain in this session' : focusedSession ? 'No matching cards due' : 'Nothing due'"
      :description="sessionResumeNotice ? 'Start a new session to study updated cards.' : focusedSession ? 'Change your filters or check again later.' : nextReviewDescription"
    >
      <template #actions>
        <UButton
          leading-icon="i-lucide-refresh-cw"
          size="lg"
          :disabled="gradingModePending"
          @click="loadStudyQueue()"
        >
          Check again
        </UButton>

        <UButton
          :to="{ name: 'library' }"
          leading-icon="i-lucide-library"
          color="neutral"
          variant="link"
          size="lg"
        >
          Open library
        </UButton>
      </template>
    </ContentState>

    <div
      v-else
      class="study-session"
    >
      <div class="study-progress">
        <div class="study-progress__row">
          <div class="study-progress__copy">
            <span v-if="masteryReady">
              {{ masteryTotal }} {{ masteryTotal === 1 ? 'retry' : 'retries' }}
            </span>
            <span v-else-if="masteryStarted">
              {{ masteryCompletedCount }} of {{ masteryTotal }} retries completed
            </span>
            <span
              v-else
              class="study-progress__count"
            >
              {{ completedCount }} completed
            </span>

            <span v-if="focusedSession">{{ selectedCardCount }} selected</span>

            <span
              v-if="mixedPracticeEnabled"
              class="study-progress__mode"
              title="Practise different topics and card types together."
            >
              <UIcon name="i-lucide-shuffle" aria-hidden="true" />
              Mixed practice
            </span>

            <span v-if="deferredEdits.length" class="study-progress__queued" role="status">
              <UIcon name="i-lucide-list-checks" aria-hidden="true" />
              {{ deferredEdits.length }} queued {{ deferredEdits.length === 1 ? 'edit' : 'edits' }}
            </span>
          </div>

          <UTooltip v-if="canUndoLastGrade || undoPending" :text="undoCommand.tooltip">
            <UButton
              leading-icon="i-lucide-undo-2"
              aria-label="Undo last grade"
              color="neutral"
              variant="ghost"
              size="sm"
              class="study-progress__undo"
              :disabled="!canUndoLastGrade"
              :loading="undoPending"
              :aria-keyshortcuts="undoCommand.ariaKeyshortcuts"
              @click="undoLastGrade"
            />
          </UTooltip>
        </div>

        <div
          class="study-progress__track"
          role="progressbar"
          :aria-label="masteryPhase ? 'Mastery progress' : 'Study progress'"
          aria-valuemin="0"
          :aria-valuemax="visibleTotalCards"
          :aria-valuenow="visibleCompletedCount"
        >
          <div
            class="study-progress__bar"
            :style="{ width: `${ visibleProgress }%` }"
          />
        </div>
      </div>

      <UAlert
        v-if="recoveryError"
        class="study-recovery-error"
        title="Grade could not be undone"
        :description="recoveryError"
        icon="i-lucide-circle-alert"
        color="error"
        variant="subtle"
      />

      <m.article
        v-if="currentCard"
        :key="currentCard.id"
        class="study-card"
        data-twill-study-card
        :data-twill-card-id="currentCard.id"
        :initial="{ opacity: resolvedMotion === 'reduced' ? 1 : 0.96 }"
        :animate="{ opacity: 1 }"
        :transition="cardTransition"
      >
        <header class="study-card__header">
          <div>
            <span class="study-card__eyebrow">
              {{ masteryActive
                ? `Mastery retry · ${ studyCardName( currentCard ) }`
                : pretestActive || pretestTeachingActive
                  ? `Pretest · ${ studyCardName( currentCard ) }`
                  : studyCardName( currentCard ) }}
            </span>
            <h2>{{ currentCard.conceptTitle }}</h2>
          </div>

          <div class="study-card__actions">
            <CardQualityAction
              :card="currentCard"
              :disabled="assessmentPending || gradingModePending || pretestPending || undoPending"
            />

            <UTooltip text="Edit now">
              <UButton
                leading-icon="i-lucide-pencil"
                aria-label="Edit now"
                color="neutral"
                variant="ghost"
                size="sm"
                :disabled="!canEditCurrentConcept"
                :loading="deferredStartPending"
                @click="editCurrentConcept"
              />
            </UTooltip>

            <UTooltip :text="currentConceptQueued ? 'Queued for editing · Edit note' : queueEditCommand.tooltip">
              <UButton
                :leading-icon="currentConceptQueued
                  ? 'i-lucide-check'
                  : 'i-lucide-list-plus'"
                :aria-label="currentConceptQueued ? 'Edit queued note' : 'Edit later'"
                color="neutral"
                :variant="currentConceptQueued ? 'subtle' : 'ghost'"
                size="sm"
                class="study-edit-later"
                :disabled="actionsBlocked || deferredLoading || deferredStartPending || Boolean( deferredPendingConceptId )"
                :loading="deferredPendingConceptId === currentCard.conceptId"
                :aria-keyshortcuts="currentConceptQueued ? undefined : queueEditCommand.ariaKeyshortcuts"
                @click="currentConceptQueued ? editCurrentNote() : queueCurrentConcept()"
              />
            </UTooltip>
          </div>
        </header>

        <div class="study-card__body">
          <UAlert
            v-if="pretestActive || pretestTeachingActive"
            class="study-pretest-notice"
            title="Pretest"
            description="Attempt this exact prompt before studying its answer. The result is separate from review grading."
            icon="i-lucide-brain"
            color="primary"
            variant="subtle"
          />

          <StudyCardContent
            ref="studyContent"
            :card="currentCard"
            :answer-revealed="answerRevealed"
            :media="studyMedia"
            :hide-answer="selectiveAnswer"
          />

          <TypeAnswerResponse
            v-if="typeAnswerSettings"
            ref="typeAnswerResponse"
            :model-value="comparisonResponse"
            :accepted-answers="typeAnswerSettings.acceptedAnswers"
            :revealed="answerRevealed"
            @update:model-value="studyResponse = $event"
            @submit="showAnswer"
          />

          <ExplainResponse
            v-if="explainSettings"
            ref="explainResponse"
            :model-value="comparisonResponse"
            :settings="explainSettings"
            :revealed="answerRevealed"
            :checks="comparisonChecks"
            :disabled="actionsBlocked"
            @update:model-value="studyResponse = $event"
            @compare="setComparison"
          />

          <ProblemResponse
            v-if="problemSettings"
            ref="problemResponse"
            :model-value="comparisonResponse"
            :settings="problemSettings"
            :revealed="answerRevealed"
            :checks="comparisonChecks"
            :disabled="actionsBlocked"
            @update:model-value="studyResponse = $event"
            @compare="setComparison"
          />

          <StudyAssistance
            v-if="!pretestActive"
            :content="currentCard.content"
            :revealed="revealedAssistance"
            :disabled="actionsBlocked"
            @toggle="toggleAssistance"
          />

          <StudyAnswerParts
            v-if="selectiveAnswer && !pretestActive"
            :document="currentCard.content.answer"
            :full-answer-revealed="answerRevealed"
            :revealed="revealedAnswerParts"
            :disabled="actionsBlocked"
            :comparison-enabled="usesAnswerPartComparison( currentCard )"
            :written-response="writtenComparison"
            :checks="comparisonChecks"
            @toggle="toggleAnswerParts"
            @compare="setComparison"
          />

          <div
            v-if="assisted"
            class="study-assistance-notice"
            data-twill-assisted
            role="status"
          >
            <strong>Help used</strong>
            <span>Grade your answer before help.</span>
            <div v-if="answerRevealed && studyResponse && studyResponse !== responseBeforeAssistance" class="study-assisted-response">
              <span>Your response after help</span>
              <p>{{ studyResponse }}</p>
            </div>
          </div>

          <StudyAnswerFeedback
            v-if="answerRevealed && currentAnswerFeedback"
            ref="answerFeedback"
            :feedback="currentAnswerFeedback"
          />
        </div>

        <footer class="study-card__footer">
          <p v-if="comparisonGuidance" class="study-comparison-guidance" role="status">
            {{ comparisonGuidance }}
          </p>

          <UAlert
            v-if="correctionPending"
            class="study-correction-notice"
            title="Grade undone"
            description="Choose the intended grade to continue."
            icon="i-lucide-undo-2"
            color="primary"
            variant="subtle"
          />

          <UAlert
            v-if="assessmentError"
            class="study-assessment-error"
            :description="assessmentError"
            icon="i-lucide-circle-alert"
            color="error"
            variant="soft"
          />

          <div
            v-if="!answerRevealed"
            key="reveal"
            class="study-actions"
          >
            <div class="study-actions__primary">
              <UButton
                ref="revealButton"
                leading-icon="i-lucide-eye"
                size="sm"
                variant="subtle"
                :disabled="!canRevealAnswer || pretestPending"
                :loading="pretestPending
                  && pendingPretestOutcome === 'attempted'"
                :aria-keyshortcuts="revealCommand.ariaKeyshortcuts"
                :title="revealCommand.tooltip"
                @click="showAnswer"
              >
                {{ revealActionLabel }}
              </UButton>

              <UButton
                v-if="pretestActive"
                color="neutral"
                variant="link"
                size="sm"
                :disabled="pretestPending"
                :loading="pretestPending
                  && pendingPretestOutcome === 'skipped'"
                @click="skipCurrentPretest"
              >
                Skip pretest
              </UButton>
            </div>
          </div>

          <div
            v-else-if="answerFeedbackPending"
            key="feedback"
            class="study-actions"
          >
            <UButton
              leading-icon="i-lucide-arrow-right"
              size="sm"
              variant="subtle"
              @click="continueToGrading"
            >
              {{ pretestTeachingActive ? 'Continue' : 'Continue to grading' }}
            </UButton>
          </div>

          <div
            v-else-if="pretestTeachingActive"
            ref="gradingActions"
            key="pretest-complete"
            class="study-actions"
          >
            <p>
              This attempt is recorded separately. The concept will return
              as an ordinary review in a later session.
            </p>

            <UButton
              leading-icon="i-lucide-arrow-right"
              size="sm"
              variant="subtle"
              @click="finishCurrentPretest"
            >
              Continue
            </UButton>
          </div>

          <div
            v-else-if="masteryActive"
            id="study-mastery-actions"
            ref="gradingActions"
            key="mastery"
            class="study-actions study-actions--assessment"
          >
            <div class="study-actions__buttons">
              <UButton
                v-for="option in masteryOptions"
                :key="option.outcome"
                :leading-icon="option.icon"
                :color="option.color"
                :variant="option.variant"
                :disabled="assessmentPending"
                :loading="assessmentPending
                  && pendingAssessment === option.outcome"
                :aria-keyshortcuts="option.command.ariaKeyshortcuts"
                :title="option.command.tooltip"
                size="sm"
                class="study-grade-button"
                @click="recordMasteryAssessment( option.recalled )"
              >
                <span>{{ option.label }}</span>

                <kbd
                  class="study-grade-button__shortcut"
                  aria-hidden="true"
                >
                  {{ option.shortcut }}
                </kbd>
              </UButton>
            </div>
          </div>

          <div
            v-else
            id="study-grading-actions"
            ref="gradingActions"
            key="assess"
            class="study-actions study-actions--assessment"
          >
            <div class="study-actions__buttons">
              <UButton
                v-for="option in gradingOptions"
                :key="option.rating"
                :leading-icon="option.icon"
                :color="option.color"
                :variant="option.variant"
                :disabled="assessmentPending || gradingModePending"
                :loading="assessmentPending && pendingAssessment === option.rating"
                :aria-keyshortcuts="option.command.ariaKeyshortcuts"
                :title="option.command.tooltip"
                size="sm"
                class="study-grade-button"
                @click="recordAssessment( option.rating )"
              >
                <span>{{ option.label }}</span>

                <kbd
                  class="study-grade-button__shortcut"
                  aria-hidden="true"
                >
                  {{ option.shortcut }}
                </kbd>
              </UButton>
            </div>
          </div>
        </footer>
      </m.article>

      <m.section
        v-else-if="masteryReady"
        key="mastery-ready"
        class="study-complete study-mastery-ready"
        :initial="{ opacity: resolvedMotion === 'reduced' ? 1 : 0.96 }"
        :animate="{ opacity: 1 }"
        :transition="cardTransition"
      >
        <span class="study-complete__icon" aria-hidden="true">
          <UIcon name="i-lucide-repeat-2" />
        </span>

        <div>
          <h2
            ref="masteryHeading"
            tabindex="-1"
          >
            Mastery round ready
          </h2>
          <p>
            {{ masteryTotal }}
            {{ masteryTotal === 1 ? 'card needs' : 'cards need' }}
            one more retrieval.
          </p>
          <p>Retries do not change review dates.</p>
        </div>

        <div class="study-complete__actions">
          <UButton
            leading-icon="i-lucide-play"
            size="lg"
            @click="beginMasteryRound"
          >
            Start mastery round
          </UButton>

          <UButton
            :to="{ name: 'library' }"
            color="neutral"
            variant="link"
            size="lg"
          >
            Finish for now
          </UButton>
        </div>
      </m.section>

      <m.section
        v-else-if="isComplete"
        key="complete"
        class="study-complete"
        :initial="{ opacity: resolvedMotion === 'reduced' ? 1 : 0.96 }"
        :animate="{ opacity: 1 }"
        :transition="cardTransition"
      >
        <span class="study-complete__icon" aria-hidden="true">
          <UIcon name="i-lucide-check" />
        </span>

        <div>
          <h2
            ref="completionHeading"
            tabindex="-1"
          >
            Session complete
          </h2>
          <p v-if="completionDescription">{{ completionDescription }}</p>
        </div>

        <dl
          v-if="completedReviewCount"
          class="study-results"
          :class="{
            'study-results--advanced': sessionGradingMode === 'advanced'
          }"
        >
          <div
            v-for="item in sessionResultItems"
            :key="item.rating"
          >
            <dt>{{ item.label }}</dt>
            <dd>{{ ratingCounts[ item.rating ] }}</dd>
          </div>
        </dl>

        <section
          v-if="pretestTotal"
          class="study-pretest-results"
          aria-labelledby="pretest-results-heading"
        >
          <div>
            <h3 id="pretest-results-heading">Pretesting</h3>
            <p>Not graded.</p>
          </div>

          <dl class="study-results">
            <div>
              <dt>Attempted</dt>
              <dd>{{ pretestAttemptedCount }}</dd>
            </div>

            <div>
              <dt>Skipped</dt>
              <dd>{{ pretestSkippedCount }}</dd>
            </div>
          </dl>
        </section>

        <section
          v-if="masteryTotal"
          class="study-mastery-results"
          aria-labelledby="mastery-results-heading"
        >
          <div>
            <h3 id="mastery-results-heading">Mastery round</h3>
            <p>One retry per missed card.</p>
          </div>

          <dl class="study-results">
            <div>
              <dt>Recalled</dt>
              <dd>{{ masteryRecalledCount }}</dd>
            </div>

            <div>
              <dt>Still learning</dt>
              <dd>{{ masteryMissedCount }}</dd>
            </div>
          </dl>

          <p v-if="masteryMissedCount">
            Missed cards will return when due.
          </p>
        </section>

        <DeferredEditQueue
          v-if="deferredEdits.length"
          :items="deferredEdits"
          :pending-concept-id="deferredPendingConceptId"
          :starting="deferredStartPending"
          @remove="removeQueuedConcept"
          @note="openQueuedNote"
          @start="startDeferredEditing"
        />

        <div class="study-complete__actions">
          <UButton
            :to="{ name: 'library' }"
            leading-icon="i-lucide-library"
            color="neutral"
            variant="link"
            size="lg"
          >
            Open library
          </UButton>
        </div>
      </m.section>
    </div>

    <DeferredEditQueue
      v-if="!initialLoading && !loadError && !hasCards && deferredEdits.length"
      :items="deferredEdits"
      :pending-concept-id="deferredPendingConceptId"
      :starting="deferredStartPending"
      @remove="removeQueuedConcept"
      @note="openQueuedNote"
      @start="startDeferredEditing"
    />
    <StudySessionBuilder
      v-model:open="builderOpen"
      :selection="builderSelection"
      :disabled="builderDisabled"
      :replacing="hasCards && !isComplete"
      :start-session="startFocusedSession"
      @after:leave="focusCurrentState"
    />

    <DeferredEditNoteDialog
      :target="noteTarget"
      :pending="notePending"
      :error="noteError"
      @close="closeQueuedNote"
      @save="saveQueuedNote"
    />
  </div>
</template>
