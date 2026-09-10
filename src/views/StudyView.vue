<script setup>
import { AnimatePresence, m } from 'motion-v';
import { computed } from 'vue';

import CardQualityAction from '../components/CardQualityAction.vue';
import ContentState from '../components/ContentState.vue';
import DeferredEditQueue from '../components/DeferredEditQueue.vue';
import ExplainResponse from '../components/ExplainResponse.vue';
import PageHeader from '../components/PageHeader.vue';
import ProblemResponse from '../components/ProblemResponse.vue';
import StudyAnswerFeedback from '../components/StudyAnswerFeedback.vue';
import StudyCardContent from '../components/StudyCardContent.vue';
import TypeAnswerResponse from '../components/TypeAnswerResponse.vue';
import { COMMAND_IDS } from '../commands/registry';
import {
  useCommandHandler,
  useCommands
} from '../composables/useCommands';
import { useStudySession } from '../composables/useStudySession';
import { useStudyFocus } from '../composables/useStudyFocus';
import { useStudyDeferredEdits } from '../composables/useStudyDeferredEdits';
import { gradingModeItems, gradingOptionsByMode } from '../study/grading';

const commands = useCommands();
const session = useStudySession({
  onStateChanged: () => focusCurrentState(),
  onAnswerRevealed: () => focusAnswer(),
  onFeedbackContinued: () => focusFirstGradingAction()
});
const {
  answerFeedbackPending,
  answerRevealed,
  assessmentError,
  assessmentPending,
  beginMasteryRound,
  canRevealAnswer,
  canUndoLastGrade,
  completedCount,
  continueToGrading,
  correctionPending,
  currentAnswerFeedback,
  currentCard,
  explainSettings,
  finishCurrentPretest,
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
  sessionGradingMode,
  sessionResumeNotice,
  showAnswer,
  skipCurrentPretest,
  studyMedia,
  studyResponse,
  totalAvailableCards,
  totalCards,
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
  focusGradingAfterFeedback,
  focusAnswer,
  focusFirstGradingAction
} = useStudyFocus( session );
const {
  deferredEdits,
  deferredError,
  deferredLoading,
  deferredPendingConceptId,
  deferredStartPending,
  currentConceptQueued,
  canQueueCurrentConcept,
  queueCurrentConcept,
  removeQueuedConcept,
  startDeferredEditing
} = useStudyDeferredEdits( session );

const cardTransition = {
  duration: 0.22,
  ease: [ 0.22, 1, 0.36, 1 ]
};

const gradingOptions = computed( () => {
  return gradingOptionsForMode( gradingMode.value );
});

const completedReviewCount = computed( () => (
  Object.values( ratingCounts.value ).reduce( ( total, count ) => total + count, 0 )
) );

const completionDescription = computed( () => {
  if ( completedReviewCount.value && pretestTotal.value ) {
    return 'Reviews and pretests saved locally.';
  }

  if ( pretestTotal.value ) {
    return 'Pretests saved locally. Attempted concepts will return as ordinary reviews next time.';
  }

  return 'Reviews saved locally.';
});

const masteryOptions = computed( () => [
  {
    color: 'error',
    command: commands.command( COMMAND_IDS.studyMasteryMissed ),
    icon: 'i-lucide-rotate-ccw',
    outcome: 'missed',
    recalled: false,
    variant: 'soft'
  },

  {
    color: 'primary',
    command: commands.command( COMMAND_IDS.studyMasteryRecalled ),
    icon: 'i-lucide-check',
    outcome: 'recalled',
    recalled: true,
    variant: 'solid'
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

const revealActionCopy = computed( () => {
  if ( pretestActive.value ) {
    return 'Make your best attempt, then inspect the answer. This does not affect scheduling.';
  }

  if ( typeAnswerSettings.value ) {
    return 'Enter an answer before checking it.';
  }

  if ( explainSettings.value ) {
    return 'Use the scratchpad if useful, then compare your explanation.';
  }

  if ( problemSettings.value ) {
    return 'Use the workpad if useful, then check the solution.';
  }

  if ( currentCard.value?.retrievalKind === 'cloze' ) {
    return 'Recall the missing text before revealing the answer.';
  }

  if ( currentCard.value?.retrievalKind === 'imageOcclusion' ) {
    return 'Recall what is hidden before revealing the answer.';
  }

  return 'Attempt the prompt before revealing the answer.';
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

const assessmentActionCopy = computed( () => {
  if ( problemSettings.value ) {
    return 'How did the problem-solving attempt go?';
  }

  return 'How did the recall attempt go?';
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

function studyCardName( card ) {
  if ( card.retrievalKind === 'cloze' ) {
    return 'Cloze';
  }

  if ( card.retrievalKind === 'typeAnswer' ) {
    return 'Type answer';
  }

  if ( card.retrievalKind === 'explain' ) {
    return 'Explain';
  }

  if ( card.retrievalKind === 'problem' ) {
    return 'Problem';
  }

  if ( card.retrievalKind === 'imageOcclusion' ) {
    return 'Image occlusion';
  }

  return card.template?.name ?? 'Standard recall';
}
</script>

<template>
  <div
    class="page study-page"
    data-twill-page="study"
  >
    <PageHeader title="Study">
      <template #actions>
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
            :disabled="gradingModeLocked
              || assessmentPending
              || initialLoading
              || pretestPending
              || undoPending"
            :loading="gradingModePending"
            value-key="value"
            leading-icon="i-lucide-list-checks"
            size="sm"
            class="grading-mode-control__select"
            @update:model-value="updateGradingMode"
          />
        </div>

        <UButton
          :to="{ name: 'library' }"
          leading-icon="i-lucide-library"
          color="neutral"
          variant="link"
        >
          Library
        </UButton>
      </template>
    </PageHeader>

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
      title="Study session restored"
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
          @click="loadStudyQueue"
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
      v-else-if="!hasCards && totalAvailableCards === 0"
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
      title="Nothing due"
      :description="nextReviewDescription"
    >
      <template #actions>
        <UButton
          leading-icon="i-lucide-refresh-cw"
          size="lg"
          :disabled="gradingModePending"
          @click="loadStudyQueue"
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
            <div class="study-progress__primary">
              <span v-if="isComplete">Session complete</span>
              <span v-else-if="masteryReady">Mastery round ready</span>
              <span v-else-if="masteryActive">
                Retry {{ masteryCompletedCount + 1 }} of {{ masteryTotal }}
              </span>
              <span v-else-if="pretestActive || pretestTeachingActive">
                Pretest {{ position }} of {{ totalCards }}
              </span>
              <span v-else>Card {{ position }} of {{ totalCards }}</span>

              <span
                v-if="mixedPracticeEnabled"
                class="study-progress__mode"
                title="This session mixes small groups of due cards using concept, retrieval form, learning state, and shared-tag signals."
              >
                <UIcon
                  name="i-lucide-shuffle"
                  aria-hidden="true"
                />
                Mixed practice
              </span>
            </div>

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
          </div>

          <UButton
            v-if="canUndoLastGrade || undoPending"
            leading-icon="i-lucide-undo-2"
            color="neutral"
            variant="subtle"
            size="md"
            class="study-progress__undo"
            :disabled="!canUndoLastGrade"
            :loading="undoPending"
            :aria-keyshortcuts="undoCommand.ariaKeyshortcuts"
            :title="undoCommand.tooltip"
            @click="undoLastGrade"
          >
            Undo last grade
          </UButton>
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

      <UAlert
        v-if="deferredEdits.length && !isComplete"
        class="study-deferred-notice"
        :title="`${ deferredEdits.length } ${ deferredEdits.length === 1
          ? 'concept'
          : 'concepts' } queued for editing`"
        description="Queued edits will be ready when this study session ends."
        icon="i-lucide-list-checks"
        color="neutral"
        variant="subtle"
      />

      <AnimatePresence
        mode="wait"
        :initial="false"
      >
        <m.article
          v-if="currentCard"
          :key="currentCard.id"
          class="study-card"
          data-twill-study-card
          :data-twill-card-id="currentCard.id"
          :initial="{ opacity: 0, x: 18 }"
          :animate="{ opacity: 1, x: 0 }"
          :exit="{ opacity: 0, x: -14 }"
          :transition="cardTransition"
          :on-animation-complete="focusCurrentState"
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
                :form-name="studyCardName( currentCard )"
                :disabled="assessmentPending || gradingModePending || pretestPending || undoPending"
              />

              <UButton
                :leading-icon="currentConceptQueued
                  ? 'i-lucide-check'
                  : 'i-lucide-list-plus'"
                color="neutral"
                :variant="currentConceptQueued ? 'subtle' : 'link'"
                size="sm"
                class="study-edit-later"
                :disabled="currentConceptQueued
                  || deferredLoading
                  || Boolean( deferredPendingConceptId )"
                :loading="deferredPendingConceptId === currentCard.conceptId"
                :aria-keyshortcuts="queueEditCommand.ariaKeyshortcuts"
                :title="queueEditCommand.tooltip"
                @click="queueCurrentConcept"
              >
                {{ currentConceptQueued ? 'Queued' : 'Edit later' }}
              </UButton>
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
            />

            <TypeAnswerResponse
              v-if="typeAnswerSettings"
              ref="typeAnswerResponse"
              v-model="studyResponse"
              :accepted-answers="typeAnswerSettings.acceptedAnswers"
              :revealed="answerRevealed"
              @submit="showAnswer"
            />

            <ExplainResponse
              v-if="explainSettings"
              ref="explainResponse"
              v-model="studyResponse"
              :settings="explainSettings"
              :revealed="answerRevealed"
            />

            <ProblemResponse
              v-if="problemSettings"
              ref="problemResponse"
              v-model="studyResponse"
              :settings="problemSettings"
              :revealed="answerRevealed"
            />

            <StudyAnswerFeedback
              v-if="answerRevealed && currentAnswerFeedback"
              ref="answerFeedback"
              :feedback="currentAnswerFeedback"
            />
          </div>

          <footer class="study-card__footer">
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

            <AnimatePresence
              mode="wait"
              :initial="false"
            >
              <m.div
                v-if="!answerRevealed"
                key="reveal"
                class="study-actions"
                :initial="{ opacity: 0, y: 5 }"
                :animate="{ opacity: 1, y: 0 }"
                :exit="{ opacity: 0, y: -5 }"
              >
                <p>{{ revealActionCopy }}</p>

                <div class="study-actions__primary">
                  <UButton
                    ref="revealButton"
                    leading-icon="i-lucide-eye"
                    size="lg"
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
                    size="lg"
                    :disabled="pretestPending"
                    :loading="pretestPending
                      && pendingPretestOutcome === 'skipped'"
                    @click="skipCurrentPretest"
                  >
                    Skip pretest
                  </UButton>
                </div>
              </m.div>

              <m.div
                v-else-if="answerFeedbackPending"
                key="feedback"
                class="study-actions"
                :initial="{ opacity: 0, y: 5 }"
                :animate="{ opacity: 1, y: 0 }"
                :exit="{ opacity: 0, y: -5 }"
              >
                <p>
                  {{ pretestTeachingActive
                    ? 'Review the answer and feedback before continuing.'
                    : 'Review the feedback before grading.' }}
                </p>

                <UButton
                  leading-icon="i-lucide-arrow-right"
                  size="lg"
                  @click="continueToGrading"
                >
                  {{ pretestTeachingActive ? 'Continue' : 'Continue to grading' }}
                </UButton>
              </m.div>

              <m.div
                v-else-if="pretestTeachingActive"
                ref="gradingActions"
                key="pretest-complete"
                class="study-actions"
                :initial="{ opacity: 0, y: 5 }"
                :animate="{ opacity: 1, y: 0 }"
                :exit="{ opacity: 0, y: -5 }"
              >
                <p>
                  This attempt is recorded separately. The concept will return
                  as an ordinary review in a later session.
                </p>

                <UButton
                  leading-icon="i-lucide-arrow-right"
                  size="lg"
                  @click="finishCurrentPretest"
                >
                  Continue
                </UButton>
              </m.div>

              <m.div
                v-else-if="masteryActive"
                id="study-mastery-actions"
                ref="gradingActions"
                key="mastery"
                class="study-actions study-actions--assessment"
                :initial="{ opacity: 0, y: 5 }"
                :animate="{ opacity: 1, y: 0 }"
                :exit="{ opacity: 0, y: -5 }"
                :on-animation-complete="focusGradingAfterFeedback"
              >
                <p>Did you recall it this time?</p>

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
                    size="lg"
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
              </m.div>

              <m.div
                v-else
                id="study-grading-actions"
                ref="gradingActions"
                key="assess"
                class="study-actions study-actions--assessment"
                :initial="{ opacity: 0, y: 5 }"
                :animate="{ opacity: 1, y: 0 }"
                :exit="{ opacity: 0, y: -5 }"
                :on-animation-complete="focusGradingAfterFeedback"
              >
                <p>{{ assessmentActionCopy }}</p>

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
                    size="lg"
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
              </m.div>
            </AnimatePresence>
          </footer>
        </m.article>

        <m.section
          v-else-if="masteryReady"
          key="mastery-ready"
          class="study-complete study-mastery-ready"
          :initial="{ opacity: 0, y: 12 }"
          :animate="{ opacity: 1, y: 0 }"
          :exit="{ opacity: 0, y: -8 }"
          :transition="cardTransition"
          :on-animation-complete="focusCurrentState"
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
            <p>Retries do not change saved schedules.</p>
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
          :initial="{ opacity: 0, y: 12 }"
          :animate="{ opacity: 1, y: 0 }"
          :exit="{ opacity: 0, y: -8 }"
          :transition="cardTransition"
          :on-animation-complete="focusCurrentState"
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
            <p>{{ completionDescription }}</p>
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
              <p>Recorded separately from review grades.</p>
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
              Still-learning cards remain on their saved review schedules.
            </p>
          </section>

          <DeferredEditQueue
            v-if="deferredEdits.length"
            :items="deferredEdits"
            :pending-concept-id="deferredPendingConceptId"
            :starting="deferredStartPending"
            @remove="removeQueuedConcept"
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
      </AnimatePresence>
    </div>

    <DeferredEditQueue
      v-if="!initialLoading && !loadError && !hasCards && deferredEdits.length"
      :items="deferredEdits"
      :pending-concept-id="deferredPendingConceptId"
      :starting="deferredStartPending"
      @remove="removeQueuedConcept"
      @start="startDeferredEditing"
    />
  </div>
</template>
