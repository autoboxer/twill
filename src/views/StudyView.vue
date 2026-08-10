<script setup>
import { AnimatePresence, m } from 'motion-v';
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue';

import ContentState from '../components/ContentState.vue';
import PageHeader from '../components/PageHeader.vue';
import RichContentRenderer from '../components/RichContentRenderer.vue';
import {
  conceptLibraryErrorMessage,
  useConceptLibrary
} from '../composables/useConceptLibrary';
import { useRecallSession } from '../composables/useRecallSession';
import { useStudyPreferences } from '../composables/useStudyPreferences';

const {
  clearError,
  getStudyQueue,
  recordReview
} = useConceptLibrary();
const {
  getStudyPreferences,
  setGradingMode
} = useStudyPreferences();
const {
  answerRevealed,
  assess,
  begin,
  completedCount,
  currentCard,
  hasCards,
  isComplete,
  position,
  progress,
  ratingCounts,
  revealAnswer,
  totalCards
} = useRecallSession();

const answerHeading = ref( null );
const assessmentError = ref( '' );
const assessmentPending = ref( false );
const completionHeading = ref( null );
const gradingMode = ref( 'simple' );
const gradingModeError = ref( '' );
const gradingModePending = ref( false );
const initialLoading = ref( true );
const loadError = ref( '' );
const nextDueAt = ref( null );
const pendingAssessment = ref( '' );
const revealButton = ref( null );
const sessionGradingMode = ref( 'simple' );
const totalAvailableCards = ref( 0 );
let loadRequestSequence = 0;
let viewActive = true;

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
      icon: 'i-lucide-rotate-ccw',
      label: 'Forgot',
      rating: 'again',
      shortcut: '1',
      variant: 'soft'
    },

    {
      color: 'primary',
      icon: 'i-lucide-check',
      label: 'Remembered',
      rating: 'good',
      shortcut: '2',
      variant: 'solid'
    }
  ],
  advanced: [
    {
      color: 'error',
      icon: 'i-lucide-rotate-ccw',
      label: 'Again',
      rating: 'again',
      shortcut: '1',
      variant: 'soft'
    },

    {
      color: 'warning',
      icon: 'i-lucide-gauge',
      label: 'Hard',
      rating: 'hard',
      shortcut: '2',
      variant: 'soft'
    },

    {
      color: 'primary',
      icon: 'i-lucide-check',
      label: 'Good',
      rating: 'good',
      shortcut: '3',
      variant: 'soft'
    },

    {
      color: 'success',
      icon: 'i-lucide-sparkles',
      label: 'Easy',
      rating: 'easy',
      shortcut: '4',
      variant: 'soft'
    }
  ]
};

const gradingModeLocked = computed( () => {
  return completedCount.value > 0 && !isComplete.value;
});

const gradingOptions = computed( () => {
  return gradingOptionsByMode[ gradingMode.value ];
});

const sessionResultItems = computed( () => {
  if ( sessionGradingMode.value === 'simple' ) {
    return [
      { label: 'Remembered', rating: 'good' },
      { label: 'Forgot', rating: 'again' }
    ];
  }

  return gradingOptionsByMode.advanced.map( ( option ) => ({
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

onMounted( () => {
  window.addEventListener( 'keydown', handleStudyKeydown );
  loadStudyQueue();
});

onBeforeUnmount( () => {
  viewActive = false;
  loadRequestSequence += 1;
  window.removeEventListener( 'keydown', handleStudyKeydown );
});

async function loadStudyQueue() {
  if ( gradingModePending.value ) {
    return;
  }

  const request = ++loadRequestSequence;

  clearError();
  gradingModeError.value = '';
  loadError.value = '';
  initialLoading.value = true;

  try {
    const [ queue, preferences ] = await Promise.all([
      getStudyQueue(),
      getStudyPreferences()
    ]);

    if ( request !== loadRequestSequence ) {
      return;
    }

    begin( queue.cards );
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

async function showAnswer() {
  revealAnswer();
  await nextTick();
  answerHeading.value?.focus();
}

async function recordAssessment( rating ) {
  const visibleRating = gradingOptions.value.some( ( option ) => {
    return option.rating === rating;
  });

  if (
    !visibleRating
    || assessmentPending.value
    || gradingModePending.value
    || !answerRevealed.value
    || !currentCard.value
  ) {
    return;
  }

  const cardId = currentCard.value.id;

  assessmentError.value = '';
  assessmentPending.value = true;
  pendingAssessment.value = rating;

  if ( completedCount.value === 0 ) {
    sessionGradingMode.value = gradingMode.value;
  }

  try {
    await recordReview( cardId, rating );

    if ( !viewActive || currentCard.value?.id !== cardId ) {
      return;
    }

    assess( rating );
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

async function updateGradingMode( nextMode ) {
  if (
    gradingModePending.value
    || assessmentPending.value
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

function handleStudyKeydown( event ) {
  const target = event.target;
  const editing = target instanceof HTMLElement && (
    target.isContentEditable
    || [ 'INPUT', 'SELECT', 'TEXTAREA' ].includes( target.tagName )
    || Boolean( target.closest( '[role="combobox"], [role="listbox"]' ) )
  );

  if (
    editing
    || event.defaultPrevented
    || event.repeat
    || event.altKey
    || event.ctrlKey
    || event.metaKey
    || event.shiftKey
    || !answerRevealed.value
    || assessmentPending.value
    || gradingModePending.value
  ) {
    return;
  }

  const option = gradingOptions.value.find( ( item ) => {
    return item.shortcut === event.key;
  });

  if ( option ) {
    event.preventDefault();
    recordAssessment( option.rating );
  }
}

function focusCurrentState() {
  if ( isComplete.value ) {
    completionHeading.value?.focus();
    return;
  }

  if ( currentCard.value && !answerRevealed.value ) {
    focusButton( revealButton.value );
  }
}

function focusButton( button ) {
  const element = button?.$el ?? button;

  element?.focus();
}
</script>

<template>
  <div class="page study-page">
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
            :disabled="gradingModeLocked || assessmentPending || initialLoading"
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
          variant="ghost"
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
          variant="soft"
        >
          Open library
        </UButton>
      </template>
    </ContentState>

    <ContentState
      v-else-if="!hasCards && totalAvailableCards === 0"
      title="No cards to study"
      description="Create or restore a concept to make a recall card available."
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
          variant="subtle"
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
          variant="subtle"
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
        <div class="study-progress__copy">
          <span v-if="isComplete">Session complete</span>
          <span v-else>Card {{ position }} of {{ totalCards }}</span>
          <span>{{ completedCount }} completed</span>
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

      <AnimatePresence
        mode="wait"
        :initial="false"
      >
        <m.article
          v-if="currentCard"
          :key="currentCard.id"
          class="study-card"
          :initial="{ opacity: 0, x: 18 }"
          :animate="{ opacity: 1, x: 0 }"
          :exit="{ opacity: 0, x: -14 }"
          :transition="cardTransition"
          :on-animation-complete="focusCurrentState"
        >
          <header class="study-card__header">
            <div>
              <span class="study-card__eyebrow">Recall</span>
              <h2>{{ currentCard.conceptTitle }}</h2>
            </div>

            <UButton
              :to="{
                name: 'concept-edit',
                params: { conceptId: currentCard.conceptId }
              }"
              leading-icon="i-lucide-pencil"
              color="neutral"
              variant="ghost"
              size="sm"
            >
              Edit concept
            </UButton>
          </header>

          <div class="study-card__body">
            <section
              class="study-document"
              aria-labelledby="study-prompt-heading"
            >
              <h3 id="study-prompt-heading">Prompt</h3>

              <RichContentRenderer
                :document="currentCard.content.prompt"
                :label="`Prompt for ${ currentCard.conceptTitle }`"
              />
            </section>

            <AnimatePresence :initial="false">
              <m.section
                v-if="answerRevealed"
                class="study-document study-document--answer"
                aria-labelledby="study-answer-heading"
                :initial="{ opacity: 0, y: 10 }"
                :animate="{ opacity: 1, y: 0 }"
                :exit="{ opacity: 0, y: -6 }"
                :transition="cardTransition"
              >
                <h3
                  id="study-answer-heading"
                  ref="answerHeading"
                  tabindex="-1"
                >
                  Answer
                </h3>

                <RichContentRenderer
                  :document="currentCard.content.answer"
                  :label="`Answer for ${ currentCard.conceptTitle }`"
                />
              </m.section>
            </AnimatePresence>
          </div>

          <footer class="study-card__footer">
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
                <p>Attempt the prompt before revealing the answer.</p>

                <UButton
                  ref="revealButton"
                  leading-icon="i-lucide-eye"
                  size="lg"
                  @click="showAnswer"
                >
                  Reveal answer
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
                    :aria-keyshortcuts="option.shortcut"
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

          <div class="study-complete__actions">
            <UButton
              :to="{ name: 'library' }"
              leading-icon="i-lucide-library"
              color="neutral"
              variant="soft"
              size="lg"
            >
              Open library
            </UButton>
          </div>
        </m.section>
      </AnimatePresence>
    </div>
  </div>
</template>
