<script setup>
import { AnimatePresence, m } from 'motion-v';
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue';
import { useRouter } from 'vue-router';

import ContentState from '../components/ContentState.vue';
import DeferredEditQueue from '../components/DeferredEditQueue.vue';
import PageHeader from '../components/PageHeader.vue';
import StudyCardContent from '../components/StudyCardContent.vue';
import TypeAnswerResponse from '../components/TypeAnswerResponse.vue';
import { COMMAND_IDS } from '../commands/registry';
import {
  conceptLibraryErrorMessage,
  useConceptLibrary
} from '../composables/useConceptLibrary';
import {
  useCommandHandler,
  useCommands
} from '../composables/useCommands';
import { useDevicePreferences } from '../composables/useDevicePreferences';
import { useDeferredEdits } from '../composables/useDeferredEdits';
import { useRecallSession } from '../composables/useRecallSession';
import {
  preserveStudySession,
  takeStudySession
} from '../study/resume';
import { normalizeTypeAnswer } from '../type-answer/comparison';

const {
  clearError,
  getStudyQueue,
  recordReview,
  reverseReview
} = useConceptLibrary();
const {
  getDevicePreferences,
  setGradingMode
} = useDevicePreferences();
const {
  getDeferredEdits,
  queueDeferredEdit,
  removeDeferredEdit
} = useDeferredEdits();
const {
  answerRevealed,
  assess,
  begin,
  completedCount,
  correctionPending,
  createSnapshot,
  currentCard,
  hasCards,
  isComplete,
  lastAssessment,
  position,
  progress,
  ratingCounts,
  revealAnswer,
  restoreLastAssessment,
  restoreSnapshot,
  totalCards
} = useRecallSession();
const commands = useCommands();
const router = useRouter();

const assessmentError = ref( '' );
const assessmentPending = ref( false );
const completionHeading = ref( null );
const deferredEdits = ref([]);
const deferredError = ref( '' );
const deferredLoading = ref( true );
const deferredPendingConceptId = ref( '' );
const deferredStartPending = ref( false );
const gradingMode = ref( 'simple' );
const gradingModeError = ref( '' );
const gradingModePending = ref( false );
const initialLoading = ref( true );
const loadError = ref( '' );
const nextDueAt = ref( null );
const pendingAssessment = ref( '' );
const recoveryError = ref( '' );
const revealButton = ref( null );
const sessionGradingMode = ref( 'simple' );
const sessionChangedConceptIds = ref( new Set() );
const sessionResumeNotice = ref( '' );
const studyContent = ref( null );
const studyMedia = ref([]);
const totalAvailableCards = ref( 0 );
const typeAnswerResponse = ref( null );
const typedResponse = ref( '' );
const undoPending = ref( false );
let loadRequestSequence = 0;
let deferredRequestSequence = 0;
let viewActive = true;
const pausedResponses = new Map();

const cardTransition = {
  duration: 0.22,
  ease: [ 0.22, 1, 0.36, 1 ]
};

const gradingModeItems = [
  { label: 'Simple', value: 'simple' },
  { label: 'Advanced', value: 'advanced' }
];

const gradingOptionsByMode = {
  simple: [
    {
      color: 'error',
      commandId: COMMAND_IDS.studyGradeSimpleForgot,
      icon: 'i-lucide-rotate-ccw',
      rating: 'again',
      variant: 'soft'
    },

    {
      color: 'primary',
      commandId: COMMAND_IDS.studyGradeSimpleRemembered,
      icon: 'i-lucide-check',
      rating: 'good',
      variant: 'solid'
    }
  ],
  advanced: [
    {
      color: 'error',
      commandId: COMMAND_IDS.studyGradeAdvancedAgain,
      icon: 'i-lucide-rotate-ccw',
      rating: 'again',
      variant: 'soft'
    },

    {
      color: 'warning',
      commandId: COMMAND_IDS.studyGradeAdvancedHard,
      icon: 'i-lucide-gauge',
      rating: 'hard',
      variant: 'soft'
    },

    {
      color: 'primary',
      commandId: COMMAND_IDS.studyGradeAdvancedGood,
      icon: 'i-lucide-check',
      rating: 'good',
      variant: 'soft'
    },

    {
      color: 'success',
      commandId: COMMAND_IDS.studyGradeAdvancedEasy,
      icon: 'i-lucide-sparkles',
      rating: 'easy',
      variant: 'soft'
    }
  ]
};

const gradingModeLocked = computed( () => {
  return (
    completedCount.value > 0
    || correctionPending.value
  ) && !isComplete.value;
});

const canUndoLastGrade = computed( () => (
  Boolean( lastAssessment.value )
  && !sessionChangedConceptIds.value.has( lastAssessment.value?.conceptId )
  && !correctionPending.value
  && !assessmentPending.value
  && !gradingModePending.value
  && !undoPending.value
) );

const queuedConceptIds = computed( () => new Set(
  deferredEdits.value.map( ( item ) => item.conceptId )
) );

const currentConceptQueued = computed( () => (
  queuedConceptIds.value.has( currentCard.value?.conceptId )
) );

const canQueueCurrentConcept = computed( () => (
  Boolean( currentCard.value )
  && !currentConceptQueued.value
  && !deferredLoading.value
  && !deferredPendingConceptId.value
  && !deferredStartPending.value
) );

const gradingOptions = computed( () => {
  return gradingOptionsForMode( gradingMode.value );
});

const typeAnswerSettings = computed( () => {
  if ( currentCard.value?.retrievalKind !== 'typeAnswer' ) {
    return null;
  }

  return currentCard.value.typeAnswer;
});

const canRevealAnswer = computed( () => (
  !typeAnswerSettings.value || Boolean( normalizeTypeAnswer( typedResponse.value ) )
) );

const revealActionCopy = computed( () => {
  if ( typeAnswerSettings.value ) {
    return 'Enter an answer before checking it.';
  }

  if ( currentCard.value?.retrievalKind === 'cloze' ) {
    return 'Recall the missing text before revealing the answer.';
  }

  if ( currentCard.value?.retrievalKind === 'imageOcclusion' ) {
    return 'Recall what is hidden before revealing the answer.';
  }

  return 'Attempt the prompt before revealing the answer.';
});

const revealActionLabel = computed( () => typeAnswerSettings.value
  ? 'Check answer'
  : 'Reveal answer'
);

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

onMounted( async () => {
  const resumableSession = takeStudySession();

  if ( resumableSession ) {
    restoreStudySession( resumableSession );
    initialLoading.value = false;
    sessionResumeNotice.value = resumableSession.changedConceptIds?.length
      ? 'Your completed session was restored. Edited concepts will use their new content next time; their earlier grades cannot be undone here.'
      : 'Your study session was restored.';
    await nextTick();
    focusCurrentState();
  } else {
    void loadStudyQueue();
  }

  void loadDeferredEditQueue();
});

onBeforeUnmount( () => {
  viewActive = false;
  loadRequestSequence += 1;
});

const revealCommand = useCommandHandler( COMMAND_IDS.studyReveal, {
  enabled: computed( () => (
    Boolean( currentCard.value )
    && !initialLoading.value
    && !answerRevealed.value
    && !assessmentPending.value
    && !gradingModePending.value
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

async function loadStudyQueue() {
  if ( gradingModePending.value ) {
    return;
  }

  const request = ++loadRequestSequence;

  clearError();
  gradingModeError.value = '';
  loadError.value = '';
  sessionResumeNotice.value = '';
  initialLoading.value = true;

  try {
    const [ queue, preferences ] = await Promise.all([
      getStudyQueue(),
      getDevicePreferences()
    ]);

    if ( request !== loadRequestSequence ) {
      return;
    }

    studyMedia.value = queue.media;
    begin( queue.cards );
    pausedResponses.clear();
    sessionChangedConceptIds.value = new Set();
    typedResponse.value = '';
    gradingMode.value = preferences.gradingMode;
    nextDueAt.value = queue.nextDueAt;
    sessionGradingMode.value = preferences.gradingMode;
    totalAvailableCards.value = queue.totalCards;
  } catch ( cause ) {
    if ( request === loadRequestSequence ) {
      loadError.value = conceptLibraryErrorMessage( cause );
    }
  } finally {
    if ( request === loadRequestSequence ) {
      initialLoading.value = false;
    }
  }
}

async function loadDeferredEditQueue() {
  const request = ++deferredRequestSequence;

  deferredError.value = '';
  deferredLoading.value = true;

  try {
    const queue = await getDeferredEdits();

    if ( request === deferredRequestSequence && viewActive ) {
      deferredEdits.value = queue.items;
    }
  } catch ( cause ) {
    if ( request === deferredRequestSequence && viewActive ) {
      deferredError.value = conceptLibraryErrorMessage( cause );
    }
  } finally {
    if ( request === deferredRequestSequence && viewActive ) {
      deferredLoading.value = false;
    }
  }
}

async function queueCurrentConcept() {
  const card = currentCard.value;

  if ( !card || !canQueueCurrentConcept.value ) {
    return;
  }

  deferredError.value = '';
  deferredPendingConceptId.value = card.conceptId;

  try {
    await queueDeferredEdit( card.conceptId, card.conceptLastChangeId );
    await loadDeferredEditQueue();
  } catch ( cause ) {
    if ( viewActive ) {
      deferredError.value = conceptLibraryErrorMessage( cause );
    }
  } finally {
    if ( viewActive ) {
      deferredPendingConceptId.value = '';
    }
  }
}

async function removeQueuedConcept( conceptId ) {
  if ( deferredPendingConceptId.value || deferredStartPending.value ) {
    return;
  }

  deferredError.value = '';
  deferredPendingConceptId.value = conceptId;

  try {
    await removeDeferredEdit( conceptId );
    await loadDeferredEditQueue();
  } catch ( cause ) {
    if ( viewActive ) {
      deferredError.value = conceptLibraryErrorMessage( cause );
    }
  } finally {
    if ( viewActive ) {
      deferredPendingConceptId.value = '';
    }
  }
}

async function startDeferredEditing() {
  const firstItem = deferredEdits.value[ 0 ];

  if (
    deferredStartPending.value
    || deferredPendingConceptId.value
    || firstItem?.targetStatus !== 'current'
  ) {
    return;
  }

  deferredStartPending.value = true;
  deferredError.value = '';

  if ( hasCards.value ) {
    preserveStudySession( createStudySessionSnapshot() );
  }

  try {
    await router.push({
      name: 'concept-edit',
      params: { conceptId: firstItem.conceptId },
      query: { deferred: '1' }
    });
  } catch ( cause ) {
    if ( viewActive ) {
      deferredError.value = cause.message || 'Queued editing could not be started.';
    }
  } finally {
    if ( viewActive ) {
      deferredStartPending.value = false;
    }
  }
}

async function showAnswer() {
  if (
    !canRevealAnswer.value
    || assessmentPending.value
    || gradingModePending.value
    || undoPending.value
  ) {
    return;
  }

  revealAnswer();
  await nextTick();

  if ( typeAnswerSettings.value ) {
    typeAnswerResponse.value?.focus();
  } else {
    studyContent.value?.focus();
  }
}

async function recordAssessment( rating ) {
  const visibleRating = gradingOptions.value.some( ( option ) => {
    return option.rating === rating;
  });

  if (
    !visibleRating
    || assessmentPending.value
    || gradingModePending.value
    || undoPending.value
    || !answerRevealed.value
    || !currentCard.value
  ) {
    return;
  }

  const cardId = currentCard.value.id;
  const response = typedResponse.value;

  assessmentError.value = '';
  recoveryError.value = '';
  assessmentPending.value = true;
  pendingAssessment.value = rating;

  if ( completedCount.value === 0 ) {
    sessionGradingMode.value = gradingMode.value;
  }

  try {
    const review = await recordReview( cardId, rating );

    if ( !viewActive || currentCard.value?.id !== cardId ) {
      return;
    }

    assess({
      rating,
      response,
      reviewId: review.reviewId
    });
    typedResponse.value = takePausedResponse( currentCard.value?.id );
    await nextTick();

    focusCurrentState();
  } catch ( cause ) {
    if ( viewActive ) {
      assessmentError.value = conceptLibraryErrorMessage( cause );
    }
  } finally {
    if ( viewActive ) {
      assessmentPending.value = false;
      pendingAssessment.value = '';
    }
  }
}

async function undoLastGrade() {
  const assessment = lastAssessment.value;

  if ( !assessment || !canUndoLastGrade.value ) {
    return;
  }

  const visibleCard = currentCard.value;

  if ( visibleCard ) {
    pausedResponses.set( visibleCard.id, typedResponse.value );
  }

  assessmentError.value = '';
  recoveryError.value = '';
  undoPending.value = true;

  try {
    await reverseReview( assessment.reviewId );

    if (
      !viewActive
      || lastAssessment.value?.reviewId !== assessment.reviewId
    ) {
      return;
    }

    const restored = restoreLastAssessment( assessment.reviewId );

    if ( !restored ) {
      return;
    }

    typedResponse.value = restored.response ?? '';
  } catch ( cause ) {
    if ( viewActive ) {
      recoveryError.value = conceptLibraryErrorMessage( cause );
    }
  } finally {
    if ( viewActive ) {
      undoPending.value = false;
    }
  }
}

async function updateGradingMode( nextMode ) {
  if (
    gradingModePending.value
    || assessmentPending.value
    || undoPending.value
    || gradingModeLocked.value
    || !gradingOptionsByMode[ nextMode ]
    || nextMode === gradingMode.value
  ) {
    return;
  }

  gradingModeError.value = '';
  gradingModePending.value = true;

  try {
    const preferences = await setGradingMode( nextMode );

    if ( !viewActive ) {
      return;
    }

    gradingMode.value = preferences.gradingMode;

    if ( completedCount.value === 0 ) {
      sessionGradingMode.value = preferences.gradingMode;
    }
  } catch ( cause ) {
    if ( viewActive ) {
      gradingModeError.value = conceptLibraryErrorMessage( cause );
    }
  } finally {
    if ( viewActive ) {
      gradingModePending.value = false;
    }
  }
}

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
      && answerRevealed.value
      && !assessmentPending.value
      && !gradingModePending.value
      && !undoPending.value
    ) ),
    execute: () => recordAssessment( rating )
  });
}

function focusCurrentState() {
  if ( isComplete.value ) {
    completionHeading.value?.focus();
    return;
  }

  if ( !currentCard.value ) {
    return;
  }

  if ( answerRevealed.value ) {
    if ( correctionPending.value ) {
      focusRevealedAnswer();
    }

    return;
  }

  if ( typeAnswerSettings.value ) {
    typeAnswerResponse.value?.focus();
  } else {
    focusButton( revealButton.value );
  }
}

function focusRevealedAnswer() {
  if ( typeAnswerSettings.value ) {
    typeAnswerResponse.value?.focus();
  } else {
    studyContent.value?.focus();
  }
}

function takePausedResponse( cardId ) {
  if ( !cardId ) {
    return '';
  }

  const response = pausedResponses.get( cardId ) ?? '';

  pausedResponses.delete( cardId );

  return response;
}

function createStudySessionSnapshot() {
  return {
    changedConceptIds: [ ...sessionChangedConceptIds.value ],
    gradingMode: gradingMode.value,
    nextDueAt: nextDueAt.value,
    pausedResponses: [ ...pausedResponses.entries() ],
    recall: createSnapshot(),
    sessionGradingMode: sessionGradingMode.value,
    studyMedia: [ ...studyMedia.value ],
    totalAvailableCards: totalAvailableCards.value,
    typedResponse: typedResponse.value
  };
}

function restoreStudySession( session ) {
  restoreSnapshot( session.recall );
  gradingMode.value = session.gradingMode;
  nextDueAt.value = session.nextDueAt;
  sessionGradingMode.value = session.sessionGradingMode;
  sessionChangedConceptIds.value = new Set( session.changedConceptIds ?? []);
  studyMedia.value = [ ...session.studyMedia ];
  totalAvailableCards.value = session.totalAvailableCards;
  typedResponse.value = session.typedResponse;
  pausedResponses.clear();

  for ( const [ cardId, response ] of session.pausedResponses ) {
    pausedResponses.set( cardId, response );
  }
}

function studyCardName( card ) {
  if ( card.retrievalKind === 'cloze' ) {
    return 'Cloze';
  }

  if ( card.retrievalKind === 'typeAnswer' ) {
    return 'Type answer';
  }

  if ( card.retrievalKind === 'imageOcclusion' ) {
    return 'Image occlusion';
  }

  return card.template?.name ?? 'Standard recall';
}

function focusButton( button ) {
  const element = button?.$el ?? button;

  element?.focus();
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
            <span v-if="isComplete">Session complete</span>
            <span v-else>Card {{ position }} of {{ totalCards }}</span>
            <span>{{ completedCount }} completed</span>
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
          aria-label="Study progress"
          aria-valuemin="0"
          :aria-valuemax="totalCards"
          :aria-valuenow="completedCount"
        >
          <div
            class="study-progress__bar"
            :style="{ width: `${ progress }%` }"
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
                {{ studyCardName( currentCard ) }}
              </span>
              <h2>{{ currentCard.conceptTitle }}</h2>
            </div>

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
          </header>

          <div class="study-card__body">
            <StudyCardContent
              ref="studyContent"
              :card="currentCard"
              :answer-revealed="answerRevealed"
              :media="studyMedia"
            />

            <TypeAnswerResponse
              v-if="typeAnswerSettings"
              ref="typeAnswerResponse"
              v-model="typedResponse"
              :accepted-answers="typeAnswerSettings.acceptedAnswers"
              :revealed="answerRevealed"
              @submit="showAnswer"
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

                <UButton
                  ref="revealButton"
                  leading-icon="i-lucide-eye"
                  size="lg"
                  :disabled="!canRevealAnswer"
                  :aria-keyshortcuts="revealCommand.ariaKeyshortcuts"
                  :title="revealCommand.tooltip"
                  @click="showAnswer"
                >
                  {{ revealActionLabel }}
                </UButton>
              </m.div>

              <m.div
                v-else
                key="assess"
                class="study-actions study-actions--assessment"
                :initial="{ opacity: 0, y: 5 }"
                :animate="{ opacity: 1, y: 0 }"
                :exit="{ opacity: 0, y: -5 }"
              >
                <p>How did the recall attempt go?</p>

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
            <p>Reviews saved locally.</p>
          </div>

          <dl
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
