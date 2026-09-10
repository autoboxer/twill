<script setup>
import { AnimatePresence, m } from 'motion-v';
import { computed, ref } from 'vue';

import ContentState from '../components/ContentState.vue';
import OrganizationManager from '../components/OrganizationManager.vue';
import PageHeader from '../components/PageHeader.vue';
import { useLibrarySearch } from '../composables/useLibrarySearch';

const {
  activeFilter,
  clearFilters,
  includeArchived,
  library,
  loadError,
  loading,
  organizations,
  page,
  query,
  refresh,
  refreshOrganizations
} = useLibrarySearch();

const organizationManagerOpen = ref( false );

const activeFilterName = computed( () => {
  if ( activeFilter.value.kind === 'all' ) {
    return 'All concepts';
  }

  const items = activeFilter.value.kind === 'deck'
    ? organizations.value.decks
    : organizations.value.tags;

  return items.find( ( item ) => item.id === activeFilter.value.id )?.name ?? 'Concepts';
});

function selectFilter( kind, id = '' ) {
  activeFilter.value = { id, kind };
}

function filterIsActive( kind, id = '' ) {
  return activeFilter.value.kind === kind && activeFilter.value.id === id;
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
          to="/create"
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

        <div class="library-results__heading">
          <div>
            <h2>{{ activeFilterName }}</h2>
            <p aria-live="polite">
              {{ loading ? 'Searching…' : library.totalCount }}
              {{ loading ? '' : library.totalCount === 1 ? 'concept' : 'concepts' }}
            </p>
          </div>

          <nav
            v-if="library.totalCount > library.pageSize"
            class="library-pagination"
            aria-label="Library pages"
          >
            <UButton
              leading-icon="i-lucide-chevron-left"
              color="neutral"
              variant="subtle"
              :disabled="loading || library.page <= 1"
              @click="page = library.page - 1"
            >
              Previous
            </UButton>

            <span>Page {{ library.page }} of {{ Math.ceil( library.totalCount / library.pageSize ) }}</span>

            <UButton
              trailing-icon="i-lucide-chevron-right"
              color="neutral"
              variant="subtle"
              :disabled="loading || library.page * library.pageSize >= library.totalCount"
              @click="page = library.page + 1"
            >
              Next
            </UButton>
          </nav>
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
          :title="query.trim() || activeFilter.kind !== 'all'
            ? 'No matching concepts'
            : library.archivedCount && !includeArchived ? 'No active concepts' : 'No concepts yet'"
          :description="query.trim() || activeFilter.kind !== 'all'
            ? 'Try different words or clear the current filters.'
            : 'Create a concept or show archived concepts.'"
        >
          <template #actions>
            <UButton
              v-if="query || activeFilter.kind !== 'all'"
              color="neutral"
              variant="subtle"
              @click="clearFilters"
            >
              Clear filters
            </UButton>

            <UButton
              v-else-if="library.archivedCount && !includeArchived"
              color="neutral"
              variant="subtle"
              @click="includeArchived = true"
            >
              Show archived
            </UButton>

            <UButton
              v-else
              to="/create"
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
      :decks="organizations.decks"
      :tags="organizations.tags"
      @changed="refreshOrganizations"
    />
  </div>
</template>
