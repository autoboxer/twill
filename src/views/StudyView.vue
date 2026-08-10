<script setup>
import { AnimatePresence, m } from 'motion-v';
import { nextTick, onBeforeUnmount, onMounted, ref } from 'vue';

import ContentState from '../components/ContentState.vue';
import PageHeader from '../components/PageHeader.vue';
import RichContentRenderer from '../components/RichContentRenderer.vue';
import {
  conceptLibraryErrorMessage,
  useConceptLibrary
} from '../composables/useConceptLibrary';
import { useRecallSession } from '../composables/useRecallSession';

const { clearError, getStudyCards } = useConceptLibrary();
const {
  answerRevealed,
  assess,
  begin,
  completedCount,
  currentCard,
  hasCards,
  isComplete,
  needsWorkCount,
  position,
  progress,
  recalledCount,
  restart,
  revealAnswer,
  totalCards
} = useRecallSession();

const answerHeading = ref( null );
const completionHeading = ref( null );
const initialLoading = ref( true );
const loadError = ref( '' );
const revealButton = ref( null );
let loadRequestSequence = 0;

const cardTransition = {
  duration: 0.22,
  ease: [ 0.22, 1, 0.36, 1 ]
};

onMounted( loadStudyCards );
onBeforeUnmount( () => {
  loadRequestSequence += 1;
});

async function loadStudyCards() {
  const request = ++loadRequestSequence;

  clearError();
  loadError.value = '';
  initialLoading.value = true;

  try {
    const cards = await getStudyCards();

    if ( request !== loadRequestSequence ) {
      return;
    }

    begin( cards );
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

async function recordAssessment( outcome ) {
  if ( !assess( outcome ) ) {
    return;
  }

  await nextTick();

  focusCurrentState();
}

async function studyAgain() {
  restart();
  await nextTick();
  focusCurrentState();
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
          @click="loadStudyCards"
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
      v-else-if="!hasCards"
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
                    leading-icon="i-lucide-rotate-ccw"
                    color="neutral"
                    variant="soft"
                    size="lg"
                    @click="recordAssessment( 'needsWork' )"
                  >
                    Needs work
                  </UButton>

                  <UButton
                    leading-icon="i-lucide-check"
                    size="lg"
                    @click="recordAssessment( 'recalled' )"
                  >
                    Recalled
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
            <p>These results are not saved yet.</p>
          </div>

          <dl class="study-results">
            <div>
              <dt>Recalled</dt>
              <dd>{{ recalledCount }}</dd>
            </div>
            <div>
              <dt>Needs work</dt>
              <dd>{{ needsWorkCount }}</dd>
            </div>
          </dl>

          <div class="study-complete__actions">
            <UButton
              leading-icon="i-lucide-refresh-cw"
              size="lg"
              @click="studyAgain"
            >
              Study again
            </UButton>

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
