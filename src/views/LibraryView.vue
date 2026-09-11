<script setup>
import { AnimatePresence, m } from 'motion-v';
import { computed, ref } from 'vue';

import ContentState from '../components/ContentState.vue';
import LibraryPagination from '../components/LibraryPagination.vue';
import OrganizationManager from '../components/OrganizationManager.vue';
import PageHeader from '../components/PageHeader.vue';
import { useLibrarySearch } from '../composables/useLibrarySearch';
import { libraryCardTypeOptions, librarySortOptions, libraryStateOptions } from '../library/search';
import { retrievalFormIcon, retrievalFormLabel } from '../retrieval-forms/catalog';
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
  tagId,
  update
} = useLibrarySearch();

const organizationManagerOpen = ref( false );
const resultsHeading = ref( null );

const activeFilterName = computed( () => {
  const names = [
    organizations.value.decks.find( ( item ) => item.id === deckId.value )?.name,
    organizations.value.tags.find( ( item ) => item.id === tagId.value )?.name
  ].filter( Boolean );

  return names.length ? names.join( ' · ' ) : 'All concepts';
});

function selectFilter( kind, id = '' ) {
  if ( kind === 'all' ) {
    update({ deckId: null, tagId: null });
  } else {
    const field = kind === 'deck' ? deckId : tagId;

    field.value = field.value === id ? null : id;
  }
}

function filterIsActive( kind, id = '' ) {
  if ( kind === 'all' ) {
    return !deckId.value && !tagId.value;
  }

  return ( kind === 'deck' ? deckId.value : tagId.value ) === id;
}

function changePage( nextPage ) {
  page.value = nextPage;
  resultsHeading.value?.focus({ preventScroll: true });
  resultsHeading.value?.scrollIntoView({ block: 'start' });
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
  <div
    class="page library-page"
    data-twill-page="library"
  >
    <PageHeader title="Library">
      <template #actions>
        <UButton
          :to="{ name: 'card-quality' }"
          leading-icon="i-lucide-flag"
          color="neutral"
          variant="link"
        >
          Card quality
        </UButton>

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
          :to="{ name: 'create', query: navigationQuery }"
          leading-icon="i-lucide-plus"
        >
          New concept
        </UButton>
      </template>
    </PageHeader>

    <div class="library-layout">
      <aside
        class="library-filters"
        aria-label="Library filters"
      >
        <div class="library-filter-group">
          <button
            type="button"
            class="library-filter"
            :class="{ 'library-filter--active': filterIsActive( 'all' ) }"
            :aria-pressed="filterIsActive( 'all' )"
            @click="selectFilter( 'all' )"
          >
            <UIcon name="i-lucide-layers-3" />
            <span>All concepts</span>
            <small>{{ library.conceptCount }}</small>
          </button>
        </div>

        <div
          v-if="organizations.decks.length"
          class="library-filter-group"
        >
          <h2>Decks</h2>

          <button
            v-for="deck in organizations.decks"
            :key="deck.id"
            type="button"
            class="library-filter"
            :class="{ 'library-filter--active': filterIsActive( 'deck', deck.id ) }"
            :aria-pressed="filterIsActive( 'deck', deck.id )"
            @click="selectFilter( 'deck', deck.id )"
          >
            <UIcon name="i-lucide-folder" />
            <span>{{ deck.name }}</span>
            <small>{{ includeArchived ? deck.conceptCount : deck.activeConceptCount }}</small>
          </button>
        </div>

        <div
          v-if="organizations.tags.length"
          class="library-filter-group"
        >
          <h2>Tags</h2>

          <button
            v-for="tag in organizations.tags"
            :key="tag.id"
            type="button"
            class="library-filter"
            :class="{ 'library-filter--active': filterIsActive( 'tag', tag.id ) }"
            :aria-pressed="filterIsActive( 'tag', tag.id )"
            @click="selectFilter( 'tag', tag.id )"
          >
            <UIcon name="i-lucide-tag" />
            <span>{{ tag.name }}</span>
            <small>{{ includeArchived ? tag.conceptCount : tag.activeConceptCount }}</small>
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

      <section
        class="library-results"
        :aria-busy="loading"
      >
        <div class="library-search">
          <label for="library-query">Search library</label>

          <UInput
            id="library-query"
            v-model="query"
            type="search"
            leading-icon="i-lucide-search"
            placeholder="Search titles and content"
            :maxlength="250"
            aria-describedby="library-search-help"
          />

          <p id="library-search-help">Use words or word prefixes. All terms must match.</p>
        </div>

        <div class="library-search-options">
          <div>
            <label for="library-card-type">Card type</label>
            <USelect
              id="library-card-type"
              v-model="cardType"
              :items="libraryCardTypeOptions"
              variant="subtle"
            />
          </div>

          <div>
            <label for="library-state">Learning state</label>
            <USelect
              id="library-state"
              v-model="state"
              :items="libraryStateOptions"
              variant="subtle"
            />
          </div>

          <div>
            <label for="library-sort">Sort by</label>
            <USelect
              id="library-sort"
              v-model="sort"
              :items="librarySortOptions"
              variant="subtle"
            />
          </div>

          <UButton
            v-if="hasFilters"
            color="neutral"
            variant="link"
            @click="clearFilters"
          >
            Clear filters
          </UButton>
        </div>

        <div
          ref="resultsHeading"
          class="library-results__heading"
          tabindex="-1"
        >
          <div>
            <h2>{{ activeFilterName }}</h2>
            <p aria-live="polite">
              {{ loading ? 'Searching…' : library.totalCount }}
              {{ loading ? '' : library.totalCount === 1 ? 'concept' : 'concepts' }}
            </p>
          </div>

          <div class="library-results__actions">
            <UButton
              :to="studyBuilderLocation(navigationQuery)"
              leading-icon="i-lucide-list-filter"
              color="neutral"
              variant="subtle"
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
              variant="subtle"
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
          <AnimatePresence :initial="false">
            <m.article
              v-for="( concept, index ) in library.concepts"
              :key="concept.id"
              class="concept-card"
              data-twill-concept-card
              layout
              :initial="{ opacity: 0, y: 8 }"
              :animate="{ opacity: 1, y: 0 }"
              :exit="{ opacity: 0, scale: 0.98 }"
              :transition="{ delay: Math.min( index * 0.025, 0.15 ) }"
            >
              <RouterLink
                :to="{
                  name: 'concept-detail',
                  params: { conceptId: concept.id },
                  query: { ...navigationQuery, card: concept.matchingForm?.id }
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

                  <p
                    v-if="concept.excerpt"
                    class="concept-card__excerpt"
                  >
                    {{ concept.excerpt }}
                  </p>

                  <p
                    v-if="concept.matchingForm"
                    class="concept-card__match"
                  >
                    <UIcon :name="retrievalFormIcon( concept.matchingForm )" />
                    <span>View {{ retrievalFormLabel( concept.matchingForm ) }}</span>
                  </p>

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

        <LibraryPagination
          v-if="!loading && !loadError && library.concepts.length"
          class="library-pagination--bottom"
          :library="library"
          @change="changePage"
        />
      </section>
    </div>

    <OrganizationManager
      v-model:open="organizationManagerOpen"
      :decks="organizations.decks"
      :tags="organizations.tags"
      @changed="refreshOrganizations"
    />
  </div>
</template>
