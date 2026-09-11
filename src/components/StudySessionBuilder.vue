<script setup>
import { computed, ref, watch } from 'vue';

import { conceptLibraryErrorMessage, useConceptLibrary } from '../composables/useConceptLibrary';
import { libraryCardTypeOptions, libraryStateOptions } from '../library/search';
import { emptyStudySelection } from '../study/selection';

const props = defineProps({
  selection: { type: Object, required: true },
  disabled: { type: Boolean, default: false },
  replacing: { type: Boolean, default: false },
  startSession: { type: Function, required: true }
});
const open = defineModel( 'open', { type: Boolean, default: false });
const { getLibraryOrganizations } = useConceptLibrary();
const input = ref( emptyStudySelection() );
const limit = ref( '' );
const organizations = ref({ decks: [], tags: [] });
const loading = ref( false );
const starting = ref( false );
const error = ref( '' );
const catalogError = ref( '' );
const catalogAttempt = ref( 0 );
const stateOptions = libraryStateOptions.filter( ( option ) => option.value !== 'due' );
const selectUi = { content: 'z-80' };

const deckOptions = computed( () => [
  { label: 'All decks', value: 'all' },
  ...organizations.value.decks.map( ( deck ) => ({ label: deck.name, value: deck.id }) )
]);
const tagOptions = computed( () => [
  { label: 'All tags', value: 'all' },
  ...organizations.value.tags.map( ( tag ) => ({ label: tag.name, value: tag.id }) )
]);
const cardLimit = computed( () => String( limit.value ).trim() === '' ? null : Number( limit.value ) );
const limitError = computed( () => cardLimit.value !== null && (
  !Number.isInteger( cardLimit.value ) || cardLimit.value < 1 || cardLimit.value > 4_294_967_295
) ? 'Enter a positive whole number, or leave this blank.' : '' );
const selectionError = computed( () => {
  if ( loading.value || catalogError.value ) {
    return '';
  }

  for ( const [ field, kind ] of [[ 'deckId', 'decks' ], [ 'tagId', 'tags' ]]) {
    if ( input.value[ field ] && !organizations.value[ kind ].some( ( item ) => item.id === input.value[ field ]) ) {
      return 'A selected deck or tag is no longer available. Choose another or clear the filters.';
    }
  }

  return '';
});

watch([ open, catalogAttempt, () => props.selection ], async ([ isOpen ], previous, onCleanup ) => {
  let active = true;

  onCleanup( () => {
    active = false;
  });

  if ( !isOpen ) {
    return;
  }

  if ( !previous?.[ 0 ] || previous[ 2 ] !== props.selection ) {
    input.value = { ...props.selection };
    limit.value = props.selection.cardLimit ?? '';
  }

  error.value = '';
  catalogError.value = '';
  loading.value = true;

  try {
    const catalog = await getLibraryOrganizations();

    if ( active ) {
      organizations.value = catalog;
    }
  } catch ( cause ) {
    if ( active ) {
      catalogError.value = conceptLibraryErrorMessage( cause );
    }
  } finally {
    if ( active ) {
      loading.value = false;
    }
  }
}, { immediate: true });

function clearFilters() {
  input.value = emptyStudySelection();
  limit.value = '';
  error.value = '';
}

async function start() {
  if ( starting.value || loading.value || props.disabled || limitError.value || selectionError.value || catalogError.value ) {
    return;
  }

  starting.value = true;
  error.value = '';

  try {
    const started = await props.startSession({ ...input.value, cardLimit: cardLimit.value });

    if ( started ) {
      open.value = false;
    }
  } catch ( cause ) {
    error.value = conceptLibraryErrorMessage( cause );
  } finally {
    starting.value = false;
  }
}
</script>

<template>
  <UModal
    v-model:open="open"
    title="Build a study session"
    description="Only active cards that are due now are included. All selected filters must match."
    :dismissible="!starting"
    :close="!starting"
    :ui="{ overlay: 'z-70', content: 'z-71 rounded-md', body: 'min-h-0' }"
  >
    <template #body>
      <form
        class="session-builder"
        data-twill-session-builder
        novalidate
        @submit.prevent="start"
      >
        <UAlert
          v-if="error || catalogError || selectionError"
          :description="error || catalogError || selectionError"
          color="error"
          variant="subtle"
          role="alert"
        />

        <UButton
          v-if="catalogError"
          color="neutral"
          variant="subtle"
          @click="catalogAttempt += 1"
        >
          Retry choices
        </UButton>

        <fieldset :disabled="starting || loading">
          <div class="session-builder__search">
            <label for="session-query">Search titles and content</label>
            <UInput
              id="session-query"
              v-model="input.query"
              class="w-full"
              type="search"
              :maxlength="250"
            />
          </div>

          <div class="session-builder__fields">
            <div>
              <label for="session-deck">Deck</label>
              <USelect
                id="session-deck"
                class="w-full"
                :model-value="input.deckId ?? 'all'"
                :items="deckOptions"
                :ui="selectUi"
                variant="subtle"
                @update:model-value="input.deckId = $event === 'all' ? null : $event"
              />
            </div>

            <div>
              <label for="session-tag">Tag</label>
              <USelect
                id="session-tag"
                class="w-full"
                :model-value="input.tagId ?? 'all'"
                :items="tagOptions"
                :ui="selectUi"
                variant="subtle"
                @update:model-value="input.tagId = $event === 'all' ? null : $event"
              />
            </div>

            <div>
              <label for="session-card-type">Card type</label>
              <USelect
                id="session-card-type"
                v-model="input.cardType"
                class="w-full"
                :items="libraryCardTypeOptions"
                :ui="selectUi"
                variant="subtle"
              />
            </div>

            <div>
              <label for="session-state">Learning state</label>
              <USelect
                id="session-state"
                v-model="input.state"
                class="w-full"
                :items="stateOptions"
                :ui="selectUi"
                variant="subtle"
              />
            </div>
          </div>

          <div class="session-builder__limit">
            <label for="session-card-limit">Card limit</label>
            <UInput
              id="session-card-limit"
              v-model="limit"
              class="w-full"
              type="number"
              :min="1"
              :step="1"
              :max="4_294_967_295"
              placeholder="All matching due cards"
              :aria-invalid="Boolean(limitError)"
              aria-describedby="session-limit-help"
            />
            <p id="session-limit-help" :class="{ 'text-error': limitError }">
              {{ limitError || 'Leave blank for all. The earliest due cards are selected before mixing.' }}
            </p>
          </div>
        </fieldset>

        <p v-if="replacing" class="session-builder__notice">
          Starting replaces the current queue and unsaved responses. Completed reviews remain saved.
        </p>

        <div class="session-builder__actions">
          <UButton
            color="neutral"
            variant="link"
            :disabled="starting || loading"
            @click="clearFilters"
          >
            Clear filters
          </UButton>
          <UButton
            color="neutral"
            variant="subtle"
            :disabled="starting"
            @click="open = false"
          >
            Cancel
          </UButton>
          <UButton
            type="submit"
            variant="subtle"
            :loading="starting || loading"
            :disabled="disabled || starting || loading || Boolean(limitError || selectionError || catalogError)"
          >
            Start session
          </UButton>
        </div>
      </form>
    </template>
  </UModal>
</template>

<style scoped>
.session-builder,
.session-builder fieldset {
  display: grid;
  gap: 1.5rem;
  min-width: 0;
}

.session-builder__fields {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 1rem;
}

.session-builder label {
  display: block;
  margin-bottom: 0.5rem;
  font-weight: 500;
}

.session-builder p {
  margin-top: 0.5rem;
  color: var(--ui-text-muted);
  font-size: 0.875rem;
}

.session-builder__actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 0.75rem;
}

.session-builder p.text-error {
  color: var(--ui-error);
}

@media (max-width: 480px) {
  .session-builder__fields {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
