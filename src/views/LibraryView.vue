<script setup>
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';

import ContentState from '../components/ContentState.vue';
import LibraryConceptRow from '../components/LibraryConceptRow.vue';
import LibraryPagination from '../components/LibraryPagination.vue';
import OrganizationManager from '../components/OrganizationManager.vue';
import PageHeader from '../components/PageHeader.vue';
import { useLibraryReturnFocus } from '../composables/useLibraryReturnFocus';
import { useLibrarySearch } from '../composables/useLibrarySearch';
import { useLibraryViewPreferences } from '../composables/useLibraryViewPreferences';
import { useStartupReady } from '../composables/useStartupReady';
import { libraryCardTypeOptions, librarySortOptions, libraryStateOptions } from '../library/search';
import { studyBuilderLocation } from '../study/selection';

const {
  cardType,
  clearFilters,
  deckId,
  hasFilters,
  includeArchived,
  library,
  loadError,
  loading,
  navigationQuery,
  organizations,
  page,
  query,
  refresh,
  refreshOrganizations,
  sort,
  state,
  tagId
} = useLibrarySearch();
const { rememberConcept } = useLibraryReturnFocus( loading, navigationQuery );
const {
  preferences: viewPreferences,
  ready: viewReady,
  loading: viewLoading,
  error: viewError,
  load: loadViewPreferences,
  setOption
} = useLibraryViewPreferences();

const viewOptions = computed( () => [
  [ 'showTags', 'Tags' ],
  [ 'showDecks', 'Decks' ],
  [ 'showPromptPreview', 'Prompt preview' ]
].map( ([ name, label ]) => ({
  label,
  type: 'checkbox',
  checked: viewPreferences.value[ name ],
  onSelect: ( event ) => event.preventDefault(),
  onUpdateChecked: ( checked ) => setOption( name, checked )
}) ) );

const organizationManagerOpen = ref( false );
const resultsHeading = ref( null );
const currentTime = ref( Date.now() );
const filtersOpen = ref( cardType.value !== 'all' || state.value !== 'all'
  || includeArchived.value || sort.value !== 'relevance' );
let timeUpdateTimer;

const filterCount = computed( () => [
  cardType.value !== 'all',
  state.value !== 'all',
  includeArchived.value
].filter( Boolean ).length );
const deckOptions = computed( () => [
  { label: 'All decks', value: 'all' },
  ...organizations.value.decks.map( ( deck ) => ({ label: deck.name, value: deck.id }) )
]);
const tagOptions = computed( () => [
  { label: 'All tags', value: 'all' },
  ...organizations.value.tags.map( ( tag ) => ({ label: tag.name, value: tag.id }) )
]);
const libraryActions = [
  { label: 'Card quality', icon: 'i-lucide-flag', to: { name: 'card-quality' } },
  { label: 'Templates', icon: 'i-lucide-layout-template', to: { name: 'templates' } },
  { label: 'Organize', icon: 'i-lucide-settings-2', onSelect: () => {
    organizationManagerOpen.value = true;
  } }
];

function changePage( nextPage ) {
  page.value = nextPage;
  resultsHeading.value?.focus({ preventScroll: true });
  resultsHeading.value?.scrollIntoView({ block: 'start' });
}

watch( library, () => {
  currentTime.value = Date.now();
});

onMounted( () => {
  timeUpdateTimer = window.setInterval( () => {
    currentTime.value = Date.now();
  }, 60_000 );
});

onBeforeUnmount( () => window.clearInterval( timeUpdateTimer ) );

useStartupReady( loading );
</script>

<template>
  <div
    class="page library-page"
    data-twill-page="library"
  >
    <PageHeader title="Library">
      <template #actions>
        <UButton
          :to="{ name: 'create', query: navigationQuery }"
          leading-icon="i-lucide-plus"
        >
          New concept
        </UButton>

        <UDropdownMenu
          :items="libraryActions"
          :content="{ align: 'end' }"
        >
          <UButton
            aria-label="Library actions"
            icon="i-lucide-ellipsis"
            color="neutral"
            variant="ghost"
          />
        </UDropdownMenu>
      </template>
    </PageHeader>

    <div class="library-toolbar">
      <div class="library-search">
        <label
          class="sr-only"
          for="library-query"
        >Search library</label>

        <UInput
          id="library-query"
          v-model="query"
          type="search"
          leading-icon="i-lucide-search"
          placeholder="Search titles and content"
          :maxlength="250"
          aria-describedby="library-search-help"
        />

        <span
          id="library-search-help"
          class="sr-only"
        >Use words or word prefixes. All terms must match.</span>
      </div>

      <USelect
        id="library-deck"
        :model-value="deckId ?? 'all'"
        :items="deckOptions"
        aria-label="Deck"
        @update:model-value="deckId = $event === 'all' ? null : $event"
      />

      <USelect
        id="library-tag"
        :model-value="tagId ?? 'all'"
        :items="tagOptions"
        aria-label="Tag"
        @update:model-value="tagId = $event === 'all' ? null : $event"
      />

      <UButton
        leading-icon="i-lucide-list-filter"
        color="neutral"
        :variant="filtersOpen ? 'subtle' : 'ghost'"
        :aria-expanded="filtersOpen"
        aria-controls="library-filter-options"
        @click="filtersOpen = !filtersOpen"
      >
        Filters{{ filterCount ? ` (${filterCount})` : '' }}
      </UButton>
    </div>

    <div
      v-show="filtersOpen"
      id="library-filter-options"
      class="library-search-options"
    >
      <div>
        <label for="library-card-type">Card type</label>
        <USelect
          id="library-card-type"
          v-model="cardType"
          :items="libraryCardTypeOptions"
        />
      </div>

      <div>
        <label for="library-state">Learning state</label>
        <USelect
          id="library-state"
          v-model="state"
          :items="libraryStateOptions"
        />
      </div>

      <div>
        <label for="library-sort">Sort by</label>
        <USelect
          id="library-sort"
          v-model="sort"
          :items="librarySortOptions"
        />
      </div>

      <label class="archive-toggle">
        <USwitch v-model="includeArchived" />
        <span>Show archived</span>
      </label>
    </div>

    <div
      v-if="viewError"
      class="library-view-error"
      role="alert"
    >
      <span>{{ viewError }}</span>
      <UButton
        v-if="!viewReady"
        :loading="viewLoading"
        color="neutral"
        variant="link"
        @click="loadViewPreferences"
      >
        Retry View options
      </UButton>
    </div>

    <section
      class="library-results"
      :aria-busy="loading"
      aria-label="Library results"
    >
      <div
        ref="resultsHeading"
        class="library-results__heading"
        tabindex="-1"
      >
        <div class="library-results__summary">
          <p aria-live="polite">
            {{ loading ? 'Searching…' : `${library.totalCount} ${library.totalCount === 1 ? 'concept' : 'concepts'}` }}
          </p>

          <UButton
            v-if="hasFilters"
            color="neutral"
            variant="link"
            @click="clearFilters"
          >
            Clear filters
          </UButton>
        </div>

        <div class="library-results__actions">
          <UDropdownMenu
            :items="viewOptions"
            :content="{ align: 'end' }"
            :ui="{
              content: 'w-44 max-w-[calc(100vw-1rem)]',
              itemTrailing: 'order-first ms-0 size-3.5 shrink-0 self-center justify-center',
              itemTrailingIcon: 'size-3.5'
            }"
          >
            <UButton
              :disabled="!viewReady"
              leading-icon="i-lucide-sliders-horizontal"
              color="neutral"
              variant="ghost"
            >
              View
            </UButton>
          </UDropdownMenu>

          <UButton
            :to="studyBuilderLocation(navigationQuery)"
            leading-icon="i-lucide-book-open"
            color="neutral"
            variant="ghost"
          >
            Study results
          </UButton>

          <LibraryPagination
            :library="library"
            :loading="loading"
            @change="changePage"
          />
        </div>
      </div>

      <ContentState
        v-if="loading"
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
            @click="refresh"
          >
            Retry
          </UButton>
        </template>
      </ContentState>

      <ContentState
        v-else-if="!library.concepts.length"
        :title="hasFilters
          ? 'No matching concepts'
          : library.archivedCount && !includeArchived ? 'No active concepts' : 'No concepts yet'"
        :description="hasFilters
          ? 'Try different words or clear the current filters.'
          : 'Create a concept or show archived concepts.'"
      >
        <template #actions>
          <UButton
            v-if="!hasFilters && library.archivedCount && !includeArchived"
            color="neutral"
            @click="includeArchived = true"
          >
            Show archived
          </UButton>

          <UButton
            v-else-if="!hasFilters"
            :to="{ name: 'create', query: navigationQuery }"
            leading-icon="i-lucide-plus"
          >
            New concept
          </UButton>
        </template>
      </ContentState>

      <div
        v-else
        class="concept-list"
      >
        <div
          class="concept-list__columns"
          aria-hidden="true"
        >
          <span>Concept</span>
          <span>Cards</span>
          <span>Due</span>
        </div>

        <LibraryConceptRow
          v-for="concept in library.concepts"
          :key="concept.id"
          :concept="concept"
          :navigation-query="navigationQuery"
          :now="currentTime"
          :view-preferences="viewPreferences"
          @open="rememberConcept( concept.id )"
        />
      </div>

      <LibraryPagination
        v-if="!loading && !loadError && library.concepts.length"
        class="library-pagination--bottom"
        :library="library"
        @change="changePage"
      />
    </section>

    <OrganizationManager
      v-model:open="organizationManagerOpen"
      :decks="organizations.decks"
      :tags="organizations.tags"
      @changed="refreshOrganizations"
    />
  </div>
</template>
