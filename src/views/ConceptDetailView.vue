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

const route = useRoute();
const router = useRouter();
const {
  clearError,
  deleteConcept,
  error,
  isPending,
  setConceptArchived
} = useConceptLibrary();
const {
  clearError: clearLoadError,
  getConcept
} = useConceptLibrary();

const concept = ref( null );
const currentTime = ref( Date.now() );
const deleteDialogOpen = ref( false );
const initialLoading = ref( true );
const loadError = ref( '' );
let loadRequestSequence = 0;
let timeUpdateTimer = null;

const conceptId = computed( () => route.params.conceptId ?? '' );
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

watch( conceptId, loadConcept, { immediate: true });

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

  clearError();
  clearLoadError();
  initialLoading.value = true;
  loadError.value = '';

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
  clearError();

  try {
    concept.value = await setConceptArchived( conceptId.value, !concept.value.archived );
  } catch {
    // Error state is handled by the composable.
  }
}

async function confirmDelete() {
  clearError();

  try {
    await deleteConcept( conceptId.value );
    deleteDialogOpen.value = false;
    await router.replace({ name: 'library' });
  } catch {
    // Error state is handled by the composable.
  }
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
  if ( card.retrievalKind === 'cloze' ) {
    const group = clozeGroupDetails( card );

    return group ? `Cloze ${ group.index + 1 }` : 'Cloze';
  }

  if ( card.retrievalKind === 'typeAnswer' ) {
    return 'Type answer';
  }

  if ( card.retrievalKind === 'imageOcclusion' ) {
    const group = imageOcclusionGroupDetails( card );

    return group ? `Image occlusion ${ group.index + 1 }` : 'Image occlusion';
  }

  return card.template?.name ?? 'Standard recall';
}

function retrievalFormIcon( card ) {
  if ( card.retrievalKind === 'cloze' ) {
    return 'i-lucide-text-select';
  }

  if ( card.retrievalKind === 'typeAnswer' ) {
    return 'i-lucide-keyboard';
  }

  if ( card.retrievalKind === 'imageOcclusion' ) {
    return 'i-lucide-scan';
  }

  return card.template ? 'i-lucide-panels-top-left' : 'i-lucide-rotate-ccw';
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
  <div class="page concept-detail-page">
    <PageHeader :title="concept?.title ?? 'Concept'">
      <template #actions>
        <UButton
          :to="{ name: 'library' }"
          leading-icon="i-lucide-arrow-left"
          color="neutral"
          variant="link"
        >
          Library
        </UButton>

        <UButton
          v-if="concept"
          :to="{
            name: 'concept-edit',
            params: { conceptId: concept.id }
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
          :to="{ name: 'library' }"
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
        v-if="error"
        :description="error"
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

      <section class="concept-detail-panel retrieval-forms">
        <div class="concept-detail-panel__heading">
          <div>
            <h2>Retrieval forms</h2>
            <p>{{ retrievalProgressLabel }}</p>
          </div>
        </div>

        <ol
          v-if="concept.cards.length"
          class="retrieval-form-list"
        >
          <li
            v-for="card in concept.cards"
            :key="card.id"
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
          :loading="isPending"
          @click="toggleArchived"
        >
          {{ archiveLabel }}
        </UButton>

        <UButton
          leading-icon="i-lucide-trash-2"
          color="error"
          variant="ghost"
          :disabled="isPending"
          @click="deleteDialogOpen = true"
        >
          Delete
        </UButton>
      </footer>
    </div>

    <ConfirmDialog
      v-model:open="deleteDialogOpen"
      title="Delete concept?"
      description="This removes the concept and its retrieval forms from this device. The deletion is retained for later synchronization."
      confirm-label="Delete concept"
      :loading="isPending"
      @confirm="confirmDelete"
    />
  </div>
</template>
