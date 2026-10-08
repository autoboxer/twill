<script setup>
import { computed, onBeforeUnmount, ref, watch } from 'vue';

import ConfirmDialog from './ConfirmDialog.vue';
import { useActionNotifications } from '../composables/useActionNotifications';
import { conceptLibraryErrorMessage, useConceptLibrary } from '../composables/useConceptLibrary';
import { usePracticeLinks } from '../composables/usePracticeLinks';

const props = defineProps({
  conceptId: { type: String, required: true },
  archived: { type: Boolean, default: false },
  navigationQuery: { type: Object, default: () => ({}) }
});

const {
  actionError, createLink, links, load, loadError, loading,
  pending, removeLink, updateLink
} = usePracticeLinks( () => props.conceptId );
const { getLibrary } = useConceptLibrary();
const { notifySuccess } = useActionNotifications();
const open = ref( false );
const editTarget = ref( null );
const removeTarget = ref( null );
const objective = ref( '' );
const query = ref( '' );
const page = ref( 1 );
const selected = ref( null );
const candidates = ref( null );
const searchLoading = ref( false );
const searchError = ref( '' );
let searchSequence = 0;
let searchTimer;

const available = computed( () => {
  const excluded = new Set([ props.conceptId, ...links.value.map( ( link ) => link.conceptId ) ]);

  return candidates.value?.concepts.filter( ( item ) => !excluded.has( item.id ) ) ?? [];
});
const objectiveValid = computed( () => {
  const length = Array.from( objective.value.trim() ).length;

  return length > 0 && length <= 300;
});
const canSave = computed( () => !pending.value && !loading.value && !loadError.value
  && objectiveValid.value && Boolean( editTarget.value || selected.value )
);
const removeOpen = computed({
  get: () => Boolean( removeTarget.value ),
  set: ( value ) => {
    if ( !value && !pending.value ) {
      removeTarget.value = null;
    }
  }
});

watch( query, () => {
  page.value = 1;
}, { flush: 'sync' });
watch([ open, query, page ], scheduleSearch, { flush: 'sync' });

onBeforeUnmount( () => {
  clearTimeout( searchTimer );
  searchSequence += 1;
});

function begin( link = null ) {
  actionError.value = '';
  editTarget.value = link;
  selected.value = null;
  objective.value = link?.objective ?? '';
  query.value = '';
  page.value = 1;
  candidates.value = null;
  open.value = true;
}

function scheduleSearch() {
  const request = ++searchSequence;

  clearTimeout( searchTimer );
  searchError.value = '';
  candidates.value = null;
  searchLoading.value = false;

  if ( !open.value || editTarget.value ) {
    return;
  }

  searchLoading.value = true;
  searchTimer = setTimeout( () => search( request ), 180 );
}

async function search( request ) {
  try {
    const result = await getLibrary({ query: query.value, page: page.value, sort: 'title' });

    if ( request === searchSequence ) {
      candidates.value = result;
    }
  } catch ( cause ) {
    if ( request === searchSequence ) {
      searchError.value = conceptLibraryErrorMessage( cause );
    }
  } finally {
    if ( request === searchSequence ) {
      searchLoading.value = false;
    }
  }
}

async function save() {
  if ( !canSave.value ) {
    return;
  }

  const editing = Boolean( editTarget.value );
  const saved = editing
    ? await updateLink( editTarget.value, objective.value )
    : await createLink( selected.value.id, objective.value );

  if ( saved ) {
    open.value = false;
    notifySuccess( editing ? 'Link updated' : 'Concept linked' );
  }
}

function requestRemove( link ) {
  actionError.value = '';
  removeTarget.value = link;
}

async function confirmRemove() {
  if ( removeTarget.value && await removeLink( removeTarget.value ) ) {
    removeTarget.value = null;
    notifySuccess( 'Link removed' );
  }
}
</script>

<template>
  <section class="concept-detail-panel linked-practice" data-twill-practice-links>
    <div class="concept-detail-panel__heading">
      <h2>Linked practice</h2>

      <UButton
        leading-icon="i-lucide-link"
        color="neutral"
        variant="subtle"
        :disabled="archived || loading || pending || Boolean( loadError )"
        @click="begin()"
      >
        Link concept
      </UButton>
    </div>

    <div v-if="loadError" role="alert" class="linked-practice__error">
      <p>{{ loadError }}</p>
      <UButton
        color="neutral"
        variant="link"
        :disabled="pending"
        @click="load"
      >
        Retry links
      </UButton>
    </div>
    <p v-else-if="loading" role="status">Loading links</p>
    <ul v-else-if="links.length" class="linked-practice__list">
      <li v-for="link in links" :key="link.id">
        <div class="linked-practice__copy">
          <RouterLink :to="{ name: 'concept-detail', params: { conceptId: link.conceptId }, query: navigationQuery }">
            {{ link.title }}
          </RouterLink>
          <span v-if="link.archived" class="linked-practice__archived">Archived</span>
          <p>{{ link.objective }}</p>
        </div>

        <div class="linked-practice__actions">
          <UTooltip text="Edit learning objective">
            <UButton
              :aria-label="`Edit link to ${ link.title }`"
              icon="i-lucide-pencil"
              color="neutral"
              variant="ghost"
              :disabled="pending"
              @click="begin( link )"
            />
          </UTooltip>
          <UTooltip text="Remove link">
            <UButton
              :aria-label="`Remove link to ${ link.title }`"
              icon="i-lucide-unlink"
              color="neutral"
              variant="ghost"
              :disabled="pending"
              @click="requestRemove( link )"
            />
          </UTooltip>
        </div>
      </li>
    </ul>
    <p v-else class="linked-practice__empty">Link a concept that applies a shared principle or helps distinguish when to use it.</p>

    <UModal
      v-model:open="open"
      :title="editTarget ? 'Edit learning objective' : 'Link concept'"
      :description="editTarget ? editTarget.title : 'Choose another case and describe what connects the questions.'"
      :dismissible="!pending"
      :close="!pending"
      :ui="{ content: 'rounded-md', description: 'break-words' }"
    >
      <template #body>
        <form id="practice-link-form" class="practice-link-form" @submit.prevent="save">
          <UAlert
            v-if="actionError"
            :description="actionError"
            color="error"
            variant="subtle"
            role="alert"
          />

          <div>
            <label for="practice-objective">Learning objective</label>
            <UTextarea
              id="practice-objective"
              v-model="objective"
              class="w-full"
              :rows="2"
              :disabled="pending"
              placeholder="For example, choose a transport based on reliability and latency."
            />
            <p v-if="Array.from( objective.trim() ).length > 300" role="alert">Use no more than 300 characters.</p>
          </div>

          <div v-if="!editTarget">
            <div v-if="selected" class="practice-link-form__selection">
              <span>{{ selected.title }}</span>
              <UButton
                color="neutral"
                variant="link"
                :disabled="pending"
                @click="selected = null"
              >
                Change
              </UButton>
            </div>

            <template v-else>
              <label for="practice-link-search">Find a concept</label>
              <UInput
                id="practice-link-search"
                v-model="query"
                class="w-full"
                type="search"
                :maxlength="250"
                :disabled="pending"
              />
              <p v-if="searchLoading" role="status">Searching</p>
              <div v-else-if="searchError" role="alert">
                <p>{{ searchError }}</p>
                <UButton color="neutral" variant="link" @click="scheduleSearch">Retry search</UButton>
              </div>
              <ul v-else class="practice-link-form__candidates" aria-label="Available concepts">
                <li v-for="item in available" :key="item.id">
                  <UButton
                    color="neutral"
                    variant="ghost"
                    :disabled="pending"
                    @click="selected = item"
                  >
                    {{ item.title }}
                  </UButton>
                </li>
                <li v-if="!available.length">No available concepts on this page. Try another search or page.</li>
              </ul>

              <div v-if="candidates && candidates.totalCount > candidates.pageSize" class="practice-link-form__pages">
                <UButton
                  aria-label="Previous concept page"
                  icon="i-lucide-chevron-left"
                  color="neutral"
                  variant="subtle"
                  :disabled="pending || page <= 1"
                  @click="page -= 1"
                />
                <span>Page {{ candidates.page }} of {{ Math.ceil( candidates.totalCount / candidates.pageSize ) }}</span>
                <UButton
                  aria-label="Next concept page"
                  icon="i-lucide-chevron-right"
                  color="neutral"
                  variant="subtle"
                  :disabled="pending || page >= Math.ceil( candidates.totalCount / candidates.pageSize )"
                  @click="page += 1"
                />
              </div>
            </template>
          </div>
        </form>
      </template>

      <template #footer>
        <div class="dialog-actions">
          <UButton
            color="neutral"
            variant="link"
            :disabled="pending"
            @click="open = false"
          >
            Cancel
          </UButton>
          <UButton
            type="submit"
            form="practice-link-form"
            variant="subtle"
            :loading="pending"
            :disabled="!canSave"
          >
            {{ editTarget ? 'Save changes' : 'Link' }}
          </UButton>
        </div>
      </template>
    </UModal>

    <ConfirmDialog
      v-model:open="removeOpen"
      title="Remove link?"
      confirm-label="Remove link"
      :description="`Remove the connection to ${ removeTarget?.title ?? 'this concept' }? Both concepts will stay in your library.`"
      :error="actionError"
      :loading="pending"
      @confirm="confirmRemove"
    />
  </section>
</template>

<style scoped>
.linked-practice__list,
.practice-link-form__candidates {
  list-style: none;
  margin: 0;
  padding: 0;
}

.linked-practice__list li {
  display: flex;
  align-items: start;
  gap: 0.75rem;
  padding-block: 0.6rem;
  border-bottom: 1px solid var(--ui-border);
}

.linked-practice__list li:last-child {
  border-bottom: 0;
}

.linked-practice__copy {
  flex: 1;
  min-width: 0;
  overflow-wrap: anywhere;
}

.linked-practice__copy a {
  color: var(--ui-primary);
}

.linked-practice__copy a:hover {
  text-decoration: underline;
}

.linked-practice__copy p,
.linked-practice__empty,
.linked-practice__archived {
  color: var(--ui-text-muted);
  font-size: 0.85rem;
}

.linked-practice__copy p {
  margin: 0.2rem 0 0;
}

.linked-practice__archived {
  margin-left: 0.5rem;
}

.linked-practice__actions,
.practice-link-form__pages,
.practice-link-form__selection {
  display: flex;
  align-items: center;
  gap: 0.4rem;
}

.linked-practice__actions {
  flex: none;
}

.practice-link-form {
  display: grid;
  gap: 1rem;
}

.practice-link-form label {
  display: block;
  margin-bottom: 0.4rem;
}

.practice-link-form__candidates {
  max-height: 14rem;
  overflow-y: auto;
  margin-block: 0.5rem;
}

.practice-link-form__candidates button {
  width: 100%;
  justify-content: start;
  white-space: normal;
  overflow-wrap: anywhere;
  text-align: left;
}

.practice-link-form__selection {
  justify-content: space-between;
  overflow-wrap: anywhere;
}

.practice-link-form__selection span {
  flex: 1;
  min-width: 0;
}

.practice-link-form__selection button {
  flex: none;
  align-self: start;
  white-space: nowrap;
}
</style>
