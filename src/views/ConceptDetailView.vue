<script setup>
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';

import ConfirmDialog from '../components/ConfirmDialog.vue';
import ContentState from '../components/ContentState.vue';
import PageHeader from '../components/PageHeader.vue';
import RichContentRenderer from '../components/RichContentRenderer.vue';
import { collectClozeGroups } from '../cloze/documents';
import {
  conceptLibraryErrorMessage,
  useConceptLibrary
} from '../composables/useConceptLibrary';
import { collectImageOcclusionGroups } from '../image-occlusion/documents';
import { libraryNavigationQuery } from '../library/search';
import { richDocumentHasContent } from '../rich-content/schema';
import { retrievalFormIcon, retrievalFormLabel } from '../retrieval-forms/catalog';

const route = useRoute();
const router = useRouter();
const {
  deleteConcept,
  getConcept,
  setConceptArchived
} = useConceptLibrary();

const actionError = ref( '' );
const concept = ref( null );
const currentTime = ref( Date.now() );
const deleteTarget = ref( null );
const initialLoading = ref( true );
const loadError = ref( '' );
const pendingAction = ref( '' );
let loadRequestSequence = 0;
let timeUpdateTimer = null;

const conceptId = computed( () => route.params.conceptId ?? '' );
const libraryQuery = computed( () => libraryNavigationQuery( route.query ) );
const detailQuery = computed( () => libraryNavigationQuery( route.query, true ) );
const selectedCardId = computed( () => detailQuery.value.card ?? '' );
const selectedCardMissing = computed( () => selectedCardId.value
  && concept.value && !concept.value.cards.some( ( card ) => card.id === selectedCardId.value )
);
const isPending = computed( () => Boolean( pendingAction.value ) );
const deleteDialogOpen = computed({
  get: () => Boolean( deleteTarget.value ),
  set: ( open ) => {
    if ( !open ) {
      deleteTarget.value = null;
    }
  }
});
const archiveLabel = computed( () => concept.value?.archived ? 'Restore' : 'Archive' );
const archiveIcon = computed( () => concept.value?.archived
  ? 'i-lucide-archive-restore'
  : 'i-lucide-archive'
);
const retrievalProgress = computed( () => {
  const cards = concept.value?.cards ?? [];
  const started = cards.filter( ( card ) => card.reviewCount > 0 ).length;
  const due = concept.value?.archived
    ? 0
    : cards.filter( ( card ) => card.dueAt <= currentTime.value ).length;

  return { due, started, total: cards.length };
});
const retrievalProgressLabel = computed( () => {
  const { due, started, total } = retrievalProgress.value;

  return `${ started } of ${ total } started · ${ due } due`;
});
const clozeGroups = computed( () => collectClozeGroups(
  concept.value?.content.prompt ?? { content: [] }
) );
const clozeGroupsById = computed( () => new Map(
  clozeGroups.value.map( ( group, index ) => [ group.id, { group, index }])
) );
const imageOcclusionGroups = computed( () => collectImageOcclusionGroups(
  concept.value?.content.prompt ?? { content: [] }
) );
const imageOcclusionGroupsById = computed( () => new Map(
  imageOcclusionGroups.value.map( ( group, index ) => [ group.id, { group, index }])
) );
const answerFeedbackDocuments = computed( () => [
  {
    document: concept.value?.content.feedback?.explanation,
    id: 'explanation',
    label: 'Explanation and context'
  },
  {
    document: concept.value?.content.feedback?.commonMistakes,
    id: 'common-mistakes',
    label: 'Common mistakes'
  }
].filter( ( item ) => richDocumentHasContent( item.document ) ) );

watch( () => route.fullPath, loadConcept, { immediate: true, flush: 'sync' });

watch([ initialLoading, selectedCardId, () => concept.value?.id ], () => {
  if ( initialLoading.value || !selectedCardId.value || selectedCardMissing.value
    || route.name !== 'concept-detail' ) {
    return;
  }

  const element = document.getElementById( `retrieval-form-${ selectedCardId.value }` );

  element?.focus({ preventScroll: true });
  element?.scrollIntoView({ block: 'center' });
}, { flush: 'post' });

onMounted( () => {
  timeUpdateTimer = window.setInterval( () => {
    currentTime.value = Date.now();
  }, 60_000 );
});

onBeforeUnmount( () => {
  loadRequestSequence += 1;
  window.clearInterval( timeUpdateTimer );
});

async function loadConcept() {
  const request = ++loadRequestSequence;
  const requestedConceptId = conceptId.value;

  concept.value = null;
  deleteTarget.value = null;
  pendingAction.value = '';
  actionError.value = '';
  initialLoading.value = true;
  loadError.value = '';

  if ( route.name !== 'concept-detail' ) {
    return;
  }

  try {
    const loadedConcept = await getConcept( requestedConceptId );

    if ( request !== loadRequestSequence ) {
      return;
    }

    concept.value = loadedConcept;
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

async function toggleArchived() {
  const target = captureActionTarget();

  if ( !target ) {
    return;
  }

  const archived = !concept.value.archived;

  actionError.value = '';
  pendingAction.value = 'archive';

  try {
    const updatedConcept = await setConceptArchived( target.id, archived );

    if ( isCurrentTarget( target ) ) {
      concept.value = updatedConcept;
    }
  } catch ( cause ) {
    if ( isCurrentTarget( target ) ) {
      actionError.value = conceptLibraryErrorMessage( cause );
    }
  } finally {
    if ( isCurrentTarget( target ) ) {
      pendingAction.value = '';
    }
  }
}

function requestDelete() {
  const target = captureActionTarget();

  if ( target ) {
    actionError.value = '';
    deleteTarget.value = target;
  }
}

async function confirmDelete() {
  const target = deleteTarget.value;

  if ( !target || !isCurrentTarget( target ) || isPending.value ) {
    return;
  }

  actionError.value = '';
  pendingAction.value = 'delete';

  try {
    await deleteConcept( target.id );

    if ( isCurrentTarget( target ) ) {
      deleteTarget.value = null;
      await router.replace({ name: 'library', query: libraryQuery.value });
    }
  } catch ( cause ) {
    if ( isCurrentTarget( target ) ) {
      actionError.value = conceptLibraryErrorMessage( cause );
    }
  } finally {
    if ( isCurrentTarget( target ) ) {
      pendingAction.value = '';
    }
  }
}

function captureActionTarget() {
  if ( initialLoading.value || loadError.value || isPending.value
    || route.name !== 'concept-detail' || concept.value?.id !== conceptId.value ) {
    return null;
  }

  return {
    id: concept.value.id,
    title: concept.value.title,
    request: loadRequestSequence
  };
}

function isCurrentTarget( target ) {
  return target.request === loadRequestSequence
    && target.id === conceptId.value
    && route.name === 'concept-detail';
}

function formattedDate( timestamp ) {
  return new Intl.DateTimeFormat( undefined, {
    day: 'numeric',
    month: 'long',
    year: 'numeric'
  }).format( new Date( timestamp ) );
}

function formattedDueDate( timestamp ) {
  if ( concept.value?.archived ) {
    return 'Paused while archived';
  }

  if ( timestamp <= currentTime.value ) {
    return 'Due now';
  }

  return `Due ${ new Intl.DateTimeFormat( undefined, {
    dateStyle: 'medium',
    timeStyle: 'short'
  }).format( new Date( timestamp ) ) }`;
}

function retrievalFormName( card ) {
  const label = retrievalFormLabel( card );

  if ( card.retrievalKind === 'cloze' ) {
    const group = clozeGroupDetails( card );

    return group ? `${ label } ${ group.index + 1 }` : label;
  }

  if ( card.retrievalKind === 'imageOcclusion' ) {
    const group = imageOcclusionGroupDetails( card );

    return group ? `${ label } ${ group.index + 1 }` : label;
  }

  return label;
}

function retrievalFormDescription( card ) {
  if ( card.retrievalKind === 'cloze' ) {
    const count = clozeGroupDetails( card )?.group.passages.length ?? 0;

    return `${ count } hidden ${ count === 1 ? 'passage' : 'passages' }`;
  }

  if ( card.retrievalKind === 'typeAnswer' ) {
    const count = card.typeAnswer.acceptedAnswers.length;

    return `${ count } accepted ${ count === 1 ? 'answer' : 'answers' }`;
  }

  if ( card.retrievalKind === 'explain' ) {
    const count = card.explain.keyPoints.length;

    return `${ explainFocusLabel( card.explain.focus ) } · ${ count } key ${ count === 1
      ? 'point'
      : 'points' }`;
  }

  if ( card.retrievalKind === 'problem' ) {
    const count = card.problem.checkpoints.length;

    return `${ count } ${ count === 1 ? 'checkpoint' : 'checkpoints' }`;
  }

  if ( card.retrievalKind === 'imageOcclusion' ) {
    const count = imageOcclusionGroupDetails( card )?.group.regions.length ?? 0;

    return `${ count } masked ${ count === 1 ? 'region' : 'regions' }`;
  }

  return card.template ? 'Template recall' : 'Built-in layout';
}

function clozeGroupDetails( card ) {
  return clozeGroupsById.value.get( card.cloze?.groupId );
}

function imageOcclusionGroupDetails( card ) {
  return imageOcclusionGroupsById.value.get( card.imageOcclusion?.groupId );
}

function explainFocusLabel( focus ) {
  return {
    causeAndEffect: 'Cause and effect',
    compareAndContrast: 'Compare and contrast',
    how: 'How',
    why: 'Why'
  }[ focus ] ?? 'Explain';
}

function clozePassageLabel( passage ) {
  const normalized = passage.trim().replace( /\s+/gu, ' ' );
  const characters = Array.from( normalized );

  return characters.length > 120
    ? `${ characters.slice( 0, 119 ).join( '' ) }…`
    : normalized;
}

function reviewCountLabel( count ) {
  if ( count === 0 ) {
    return 'Not studied yet';
  }

  return `${ count } ${ count === 1 ? 'review' : 'reviews' }`;
}

function schedulingStateDetails( state ) {
  return {
    learning: { color: 'warning', label: 'Learning' },
    new: { color: 'neutral', label: 'New' },
    relearning: { color: 'error', label: 'Relearning' },
    review: { color: 'primary', label: 'Review' }
  }[ state ] ?? { color: 'neutral', label: state };
}
</script>

<template>
  <div
    class="page concept-detail-page"
    data-twill-page="concept-detail"
  >
    <PageHeader :title="concept?.title ?? 'Concept'">
      <template #leading>
        <UButton
          :to="{ name: 'library', query: libraryQuery }"
          leading-icon="i-lucide-arrow-left"
          color="neutral"
          variant="link"
        >
          Library
        </UButton>
      </template>

      <template #actions>
        <UButton
          v-if="concept"
          :to="{
            name: 'concept-edit',
            params: { conceptId: concept.id },
            query: detailQuery
          }"
          leading-icon="i-lucide-pencil"
          color="neutral"
          variant="subtle"
        >
          Edit
        </UButton>
      </template>
    </PageHeader>

    <ContentState
      v-if="initialLoading"
      kind="loading"
      title="Loading concept"
    />

    <ContentState
      v-else-if="loadError"
      kind="error"
      title="Concept could not be loaded"
      :description="loadError"
    >
      <template #actions>
        <UButton
          leading-icon="i-lucide-refresh-cw"
          @click="loadConcept"
        >
          Retry
        </UButton>

        <UButton
          :to="{ name: 'library', query: libraryQuery }"
          color="neutral"
          variant="link"
        >
          Back to library
        </UButton>
      </template>
    </ContentState>

    <div
      v-else-if="concept"
      class="concept-detail-layout"
    >
      <UAlert
        v-if="actionError && !deleteDialogOpen"
        role="alert"
        :description="actionError"
        icon="i-lucide-circle-alert"
        color="error"
        variant="soft"
      />

      <UAlert
        v-if="concept.archived"
        title="Archived"
        description="This concept is hidden from the active library."
        icon="i-lucide-archive"
        color="neutral"
        variant="soft"
      />

      <section class="concept-detail-panel concept-detail-summary">
        <div class="concept-labels">
          <UBadge
            v-for="deck in concept.decks"
            :key="`deck-${ deck.id }`"
            :label="deck.name"
            leading-icon="i-lucide-folder"
            color="primary"
            variant="subtle"
          />

          <UBadge
            v-for="tag in concept.tags"
            :key="`tag-${ tag.id }`"
            :label="tag.name"
            leading-icon="i-lucide-tag"
            color="neutral"
            variant="soft"
          />

          <span
            v-if="!concept.decks.length && !concept.tags.length"
            class="concept-detail-summary__unfiled"
          >
            No decks or tags
          </span>
        </div>

        <dl class="concept-detail-dates">
          <div>
            <dt>Created</dt>
            <dd>{{ formattedDate( concept.createdAt ) }}</dd>
          </div>

          <div>
            <dt>Updated</dt>
            <dd>{{ formattedDate( concept.updatedAt ) }}</dd>
          </div>
        </dl>
      </section>

      <section class="concept-detail-panel concept-content-panel">
        <div class="concept-content-document">
          <h2>Prompt</h2>

          <RichContentRenderer
            :document="concept.content.prompt"
            label="Prompt"
          />
        </div>

        <div class="concept-content-document">
          <h2>Answer</h2>

          <RichContentRenderer
            :document="concept.content.answer"
            label="Answer"
          />
        </div>
      </section>

      <section
        v-if="answerFeedbackDocuments.length"
        class="concept-detail-panel concept-feedback-panel"
        data-twill-concept-feedback
      >
        <div class="concept-detail-panel__heading">
          <div>
            <h2>Answer feedback</h2>
            <p>Shown after the expected answer during study.</p>
          </div>
        </div>

        <div class="concept-feedback-documents">
          <div
            v-for="item in answerFeedbackDocuments"
            :key="item.id"
            class="concept-feedback-document"
          >
            <h3>{{ item.label }}</h3>

            <RichContentRenderer
              :document="item.document"
              :label="item.label"
            />
          </div>
        </div>
      </section>

      <section class="concept-detail-panel retrieval-forms">
        <div class="concept-detail-panel__heading">
          <div>
            <h2>Retrieval forms</h2>
            <p>{{ retrievalProgressLabel }}</p>
          </div>
        </div>

        <p
          v-if="selectedCardMissing"
          class="retrieval-forms__empty"
          role="status"
        >
          The selected retrieval form is no longer available.
        </p>

        <ol
          v-if="concept.cards.length"
          class="retrieval-form-list"
        >
          <li
            v-for="card in concept.cards"
            :id="`retrieval-form-${ card.id }`"
            :key="card.id"
            :class="{ 'retrieval-form-list__selected': card.id === selectedCardId }"
            tabindex="-1"
          >
            <span
              class="retrieval-form-list__icon"
              aria-hidden="true"
            >
              <UIcon :name="retrievalFormIcon( card )" />
            </span>

            <div class="retrieval-form-list__copy">
              <strong>{{ retrievalFormName( card ) }}</strong>
              <span>
                {{ retrievalFormDescription( card ) }}
                · {{ reviewCountLabel( card.reviewCount ) }}
              </span>

              <div
                v-if="card.typeAnswer"
                class="retrieval-form-list__answers"
              >
                <span>{{ card.typeAnswer.acceptedAnswers.length === 1
                  ? 'Accepted answer'
                  : 'Accepted answers' }}</span>

                <ul>
                  <li
                    v-for="answer in card.typeAnswer.acceptedAnswers"
                    :key="answer"
                  >
                    {{ answer }}
                  </li>
                </ul>
              </div>

              <div
                v-if="card.explain"
                class="retrieval-form-list__answers"
              >
                <span>Key points</span>

                <ul>
                  <li
                    v-for="keyPoint in card.explain.keyPoints"
                    :key="keyPoint"
                  >
                    {{ keyPoint }}
                  </li>
                </ul>
              </div>

              <div
                v-if="card.problem"
                class="retrieval-form-list__answers"
              >
                <span>Solution checkpoints</span>

                <ol>
                  <li
                    v-for="checkpoint in card.problem.checkpoints"
                    :key="checkpoint"
                  >
                    <span>{{ checkpoint }}</span>
                  </li>
                </ol>
              </div>

              <div
                v-if="card.cloze"
                class="retrieval-form-list__answers"
              >
                <span>{{ clozeGroupDetails( card )?.group.passages.length === 1
                  ? 'Hidden passage'
                  : 'Hidden passages' }}</span>

                <ul>
                  <li
                    v-for="( passage, index ) in clozeGroupDetails( card )?.group.passages ?? []"
                    :key="`${ card.cloze.groupId }-${ index }`"
                  >
                    {{ clozePassageLabel( passage ) }}
                  </li>
                </ul>
              </div>

              <div
                v-if="card.imageOcclusion"
                class="retrieval-form-list__answers"
              >
                <span>Source image</span>

                <ul>
                  <li>
                    {{ imageOcclusionGroupDetails( card )?.group.image.alt
                      || 'Prompt image' }}
                  </li>
                </ul>
              </div>
            </div>

            <div class="retrieval-form-list__schedule">
              <UBadge
                :label="schedulingStateDetails( card.schedulingState ).label"
                :color="schedulingStateDetails( card.schedulingState ).color"
                variant="soft"
              />

              <span>{{ formattedDueDate( card.dueAt ) }}</span>
            </div>
          </li>
        </ol>

        <div
          v-else
          class="retrieval-forms__empty"
        >
          No retrieval forms yet.
        </div>
      </section>

      <footer class="concept-detail-actions">
        <UButton
          :leading-icon="archiveIcon"
          color="neutral"
          variant="subtle"
          :loading="pendingAction === 'archive'"
          :disabled="isPending"
          @click="toggleArchived"
        >
          {{ archiveLabel }}
        </UButton>

        <UButton
          leading-icon="i-lucide-trash-2"
          color="error"
          variant="ghost"
          :disabled="isPending"
          @click="requestDelete"
        >
          Delete
        </UButton>
      </footer>
    </div>

    <ConfirmDialog
      v-model:open="deleteDialogOpen"
      title="Delete concept?"
      :description="deleteTarget
        ? `“${ deleteTarget.title }” and its retrieval forms will be removed from this device. The deletion is retained for later synchronization.`
        : ''"
      confirm-label="Delete concept"
      :error="actionError"
      :loading="pendingAction === 'delete'"
      @confirm="confirmDelete"
    />
  </div>
</template>
