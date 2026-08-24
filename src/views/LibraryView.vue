<script setup>
import { AnimatePresence, m } from 'motion-v';
import { computed, ref, watch } from 'vue';

import ContentState from '../components/ContentState.vue';
import OrganizationManager from '../components/OrganizationManager.vue';
import PageHeader from '../components/PageHeader.vue';
import {
  conceptLibraryErrorMessage,
  useConceptLibrary
} from '../composables/useConceptLibrary';

const { clearError, getLibrary } = useConceptLibrary();

const activeFilter = ref({ id: '', kind: 'all' });
const includeArchived = ref( false );
const initialLoading = ref( true );
const library = ref({
  archivedCount: 0,
  concepts: [],
  decks: [],
  tags: []
});

const loadError = ref( '' );
const organizationManagerOpen = ref( false );
let loadRequestSequence = 0;

const filteredConcepts = computed( () => {
  if ( activeFilter.value.kind === 'deck' ) {
    return library.value.concepts.filter( ( concept ) => concept.decks.some(
      ( deck ) => deck.id === activeFilter.value.id
    ) );
  }

  if ( activeFilter.value.kind === 'tag' ) {
    return library.value.concepts.filter( ( concept ) => concept.tags.some(
      ( tag ) => tag.id === activeFilter.value.id
    ) );
  }

  return library.value.concepts;
});

const activeFilterName = computed( () => {
  if ( activeFilter.value.kind === 'all' ) {
    return 'All concepts';
  }

  const items = activeFilter.value.kind === 'deck'
    ? library.value.decks
    : library.value.tags;

  return items.find( ( item ) => item.id === activeFilter.value.id )?.name ?? 'Concepts';
});

watch( includeArchived, () => loadLibrary(), { immediate: true });

function selectFilter( kind, id = '' ) {
  activeFilter.value = { id, kind };
}

function filterIsActive( kind, id = '' ) {
  return activeFilter.value.kind === kind && activeFilter.value.id === id;
}

function ensureFilterExists() {
  if ( activeFilter.value.kind === 'deck' ) {
    const exists = library.value.decks.some( ( deck ) => deck.id === activeFilter.value.id );

    if ( !exists ) {
      selectFilter( 'all' );
    }
  }

  if ( activeFilter.value.kind === 'tag' ) {
    const exists = library.value.tags.some( ( tag ) => tag.id === activeFilter.value.id );

    if ( !exists ) {
      selectFilter( 'all' );
    }
  }
}

async function loadLibrary( showLoading = true ) {
  const request = ++loadRequestSequence;
  const requestedIncludeArchived = includeArchived.value;

  clearError();
  loadError.value = '';

  if ( showLoading ) {
    initialLoading.value = true;
  }

  try {
    const snapshot = await getLibrary( requestedIncludeArchived );

    if ( request !== loadRequestSequence ) {
      return;
    }

    library.value = snapshot;
    ensureFilterExists();
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

function visibleConceptCount( kind, id ) {
  return library.value.concepts.filter( ( concept ) => concept[ kind ].some(
    ( item ) => item.id === id
  ) ).length;
}

function formattedDate( timestamp ) {
  return new Intl.DateTimeFormat( undefined, {
    day: 'numeric',
    month: 'short',
    year: 'numeric'
  }).format( new Date( timestamp ) );
}
</script>

<template>
  <div class="page library-page">
    <PageHeader title="Library">
      <template #actions>
        <UButton
          :to="{ name: 'templates' }"
          leading-icon="i-lucide-layout-template"
          color="neutral"
          variant="link"
        >
          Templates
        </UButton>

        <UButton
          leading-icon="i-lucide-settings-2"
          color="neutral"
          variant="subtle"
          @click="organizationManagerOpen = true"
        >
          Organize
        </UButton>

        <UButton
          to="/create"
          leading-icon="i-lucide-plus"
        >
          New concept
        </UButton>
      </template>
    </PageHeader>

    <ContentState
      v-if="initialLoading"
      kind="loading"
      title="Loading library"
    />

    <ContentState
      v-else-if="loadError"
      kind="error"
      title="Library could not be loaded"
      :description="loadError"
    >
      <template #actions>
        <UButton
          leading-icon="i-lucide-refresh-cw"
          @click="loadLibrary"
        >
          Retry
        </UButton>
      </template>
    </ContentState>

    <ContentState
      v-else-if="!library.concepts.length"
      :title="library.archivedCount && !includeArchived
        ? 'No active concepts'
        : 'No concepts yet'"
      :description="library.archivedCount && !includeArchived
        ? 'Archived concepts are hidden from the current view.'
        : 'Create a concept to begin building the library.'"
    >
      <template #actions>
        <UButton
          v-if="library.archivedCount && !includeArchived"
          leading-icon="i-lucide-archive"
          color="neutral"
          variant="subtle"
          @click="includeArchived = true"
        >
          Show archived
        </UButton>

        <UButton
          to="/create"
          leading-icon="i-lucide-plus"
        >
          New concept
        </UButton>
      </template>
    </ContentState>

    <div
      v-else
      class="library-layout"
    >
      <aside
        class="library-filters"
        aria-label="Library filters"
      >
        <div class="library-filter-group">
          <button
            type="button"
            class="library-filter"
            :class="{ 'library-filter--active': filterIsActive( 'all' ) }"
            @click="selectFilter( 'all' )"
          >
            <UIcon name="i-lucide-layers-3" />
            <span>All concepts</span>
            <small>{{ library.concepts.length }}</small>
          </button>
        </div>

        <div
          v-if="library.decks.length"
          class="library-filter-group"
        >
          <h2>Decks</h2>

          <button
            v-for="deck in library.decks"
            :key="deck.id"
            type="button"
            class="library-filter"
            :class="{ 'library-filter--active': filterIsActive( 'deck', deck.id ) }"
            @click="selectFilter( 'deck', deck.id )"
          >
            <UIcon name="i-lucide-folder" />
            <span>{{ deck.name }}</span>
            <small>{{ visibleConceptCount( 'decks', deck.id ) }}</small>
          </button>
        </div>

        <div
          v-if="library.tags.length"
          class="library-filter-group"
        >
          <h2>Tags</h2>

          <button
            v-for="tag in library.tags"
            :key="tag.id"
            type="button"
            class="library-filter"
            :class="{ 'library-filter--active': filterIsActive( 'tag', tag.id ) }"
            @click="selectFilter( 'tag', tag.id )"
          >
            <UIcon name="i-lucide-tag" />
            <span>{{ tag.name }}</span>
            <small>{{ visibleConceptCount( 'tags', tag.id ) }}</small>
          </button>
        </div>

        <label class="archive-toggle">
          <USwitch v-model="includeArchived" />
          <span>
            Show archived
            <small v-if="library.archivedCount">{{ library.archivedCount }}</small>
          </span>
        </label>
      </aside>

      <section class="library-results">
        <div class="library-results__heading">
          <div>
            <h2>{{ activeFilterName }}</h2>
            <p>
              {{ filteredConcepts.length }}
              {{ filteredConcepts.length === 1 ? 'concept' : 'concepts' }}
            </p>
          </div>
        </div>

        <ContentState
          v-if="!filteredConcepts.length"
          title="No concepts in this view"
          description="Choose another deck or tag, or clear the current filter."
        >
          <template #actions>
            <UButton
              color="neutral"
              variant="subtle"
              @click="selectFilter( 'all' )"
            >
              Clear filter
            </UButton>
          </template>
        </ContentState>

        <div
          v-else
          class="concept-list"
        >
          <AnimatePresence :initial="false">
            <m.article
              v-for="( concept, index ) in filteredConcepts"
              :key="concept.id"
              class="concept-card"
              layout
              :initial="{ opacity: 0, y: 8 }"
              :animate="{ opacity: 1, y: 0 }"
              :exit="{ opacity: 0, scale: 0.98 }"
              :transition="{ delay: Math.min( index * 0.025, 0.15 ) }"
            >
              <RouterLink
                :to="{
                  name: 'concept-detail',
                  params: { conceptId: concept.id }
                }"
                class="concept-card__link"
              >
                <div class="concept-card__content">
                  <div class="concept-card__title-row">
                    <h3>{{ concept.title }}</h3>

                    <UBadge
                      v-if="concept.archived"
                      label="Archived"
                      color="neutral"
                      variant="soft"
                      size="sm"
                    />
                  </div>

                  <div
                    v-if="concept.decks.length || concept.tags.length"
                    class="concept-labels"
                  >
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
                  </div>

                  <div class="concept-card__meta">
                    <span>
                      {{ concept.cardCount }}
                      {{ concept.cardCount === 1 ? 'retrieval form' : 'retrieval forms' }}
                    </span>
                    <span aria-hidden="true">·</span>
                    <span>Updated {{ formattedDate( concept.updatedAt ) }}</span>
                  </div>
                </div>

                <UIcon
                  name="i-lucide-chevron-right"
                  class="concept-card__arrow"
                  aria-hidden="true"
                />
              </RouterLink>
            </m.article>
          </AnimatePresence>
        </div>
      </section>
    </div>

    <OrganizationManager
      v-model:open="organizationManagerOpen"
      :decks="library.decks"
      :tags="library.tags"
      @changed="loadLibrary( false )"
    />
  </div>
</template>
