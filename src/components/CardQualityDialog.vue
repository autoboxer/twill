<script setup>
import { computed, onBeforeUnmount, ref, useId, watch } from 'vue';

import {
  cardQualityKinds,
  maximumCardQualityNoteLength
} from '../card-quality/options';
import { useCardQuality } from '../composables/useCardQuality';
import { conceptLibraryErrorMessage } from '../composables/useConceptLibrary';

const props = defineProps({
  cards: {
    type: Array,
    required: true
  },
  conceptTitle: {
    type: String,
    required: true
  },
  loading: {
    type: Boolean,
    default: false
  },
  loadError: {
    type: String,
    default: ''
  }
});

const open = defineModel( 'open', { type: Boolean, default: false });
const emit = defineEmits([ 'closeAutoFocus', 'retry', 'saved' ]);
const { flagCard, getQueue } = useCardQuality();
const formId = useId();
const selectedCardId = ref();
const existingConcern = ref( null );
const kind = ref( 'ambiguous' );
const note = ref( '' );
const saveError = ref( '' );
const saving = ref( false );
let requestSequence = 0;

const selectedCard = computed( () => props.cards.find( ( card ) => card.id === selectedCardId.value ) );
const cardOptions = computed( () => props.cards.map( ( card ) => ({
  label: card.label,
  value: card.id
}) ) );
const description = computed( () => props.cards.length === 1
  ? `${ props.conceptTitle } · ${ props.cards[ 0 ].label }`
  : props.conceptTitle
);
const noteLength = computed( () => Array.from( note.value.trim() ).length );
const noteError = computed( () => {
  if ( noteLength.value > maximumCardQualityNoteLength ) {
    return `Notes cannot exceed ${ maximumCardQualityNoteLength } characters.`;
  }

  if ( note.value.includes( '\0' ) ) {
    return 'Remove the invalid character from the note.';
  }

  return '';
});

watch([ open, () => props.cards ], () => {
  requestSequence += 1;
  saving.value = false;

  if ( !open.value ) {
    return;
  }

  selectedCardId.value = props.cards.length === 1 ? props.cards[ 0 ].id : undefined;
  kind.value = 'ambiguous';
  note.value = '';
  saveError.value = '';
  existingConcern.value = null;
}, { immediate: true, flush: 'sync' });

watch([ kind, selectedCardId ], () => {
  requestSequence += 1;
  saveError.value = '';
  existingConcern.value = null;
}, { flush: 'sync' });

watch( selectedCardId, () => {
  note.value = '';
}, { flush: 'sync' });

onBeforeUnmount( () => {
  requestSequence += 1;
});

async function saveIssue() {
  if ( saving.value || !open.value || props.loading || props.loadError
    || !selectedCard.value || noteError.value ) {
    return;
  }

  const request = ++requestSequence;
  const cardId = selectedCard.value.id;
  const requestedKind = kind.value;

  saving.value = true;
  saveError.value = '';
  existingConcern.value = null;

  try {
    await flagCard( cardId, requestedKind, note.value );

    if ( request === requestSequence ) {
      emit( 'saved' );
      open.value = false;
    }
  } catch ( cause ) {
    if ( request !== requestSequence ) {
      return;
    }

    saveError.value = conceptLibraryErrorMessage( cause );

    if ( cause?.code === 'conflict' ) {
      saveError.value = 'This card already has an open issue of this kind. Your new note has not been saved.';

      try {
        const queue = await getQueue();

        if ( request === requestSequence ) {
          existingConcern.value = queue.items.find( ( item ) => (
            item.cardId === cardId
            && item.source === 'manual'
            && item.kind === requestedKind
          ) ) ?? null;

          if ( !existingConcern.value ) {
            saveError.value = 'The existing issue has changed. Try saving your note again.';
          }
        }
      } catch {
        if ( request === requestSequence ) {
          saveError.value += ' The existing note could not be loaded. Try again.';
        }
      }
    }
  } finally {
    if ( request === requestSequence ) {
      saving.value = false;
    }
  }
}
</script>

<template>
  <UModal
    v-model:open="open"
    title="Card issue"
    :description="description"
    :dismissible="!saving"
    :close="!saving"
    :content="{ onCloseAutoFocus: ( event ) => emit( 'closeAutoFocus', event ) }"
    :ui="{ content: 'card-quality-dialog rounded-md', description: 'wrap-anywhere' }"
    scrollable
  >
    <template v-if="$slots.default" #default>
      <slot />
    </template>

    <template #body>
      <p v-if="loading" role="status">Loading cards…</p>

      <UAlert
        v-else-if="loadError"
        role="alert"
        :description="loadError"
        color="error"
        variant="subtle"
        :actions="[{ label: 'Retry', onClick: () => emit( 'retry' ) }]"
      />

      <form
        v-else
        :id="formId"
        class="card-quality-form"
        novalidate
        @submit.prevent="saveIssue"
      >
        <UFormField v-if="cards.length > 1" label="Card" required>
          <USelect
            v-model="selectedCardId"
            :items="cardOptions"
            placeholder="Choose a card"
            :disabled="saving"
            class="w-full"
            :ui="{ itemLabel: 'whitespace-normal wrap-anywhere' }"
          />
        </UFormField>

        <UFormField label="Issue" required>
          <USelect
            v-model="kind"
            :items="cardQualityKinds"
            :disabled="saving"
            class="w-full"
          />
        </UFormField>

        <UFormField
          label="Note (optional)"
          :hint="`${ noteLength } / ${ maximumCardQualityNoteLength }`"
          :error="noteError || undefined"
        >
          <UTextarea
            v-model="note"
            :rows="3"
            :disabled="saving"
            class="w-full"
          />
        </UFormField>

        <UAlert
          v-if="saveError"
          role="alert"
          :description="saveError"
          icon="i-lucide-circle-alert"
          color="error"
          variant="subtle"
        />

        <div
          v-if="existingConcern"
          class="card-quality-existing"
          role="status"
        >
          <h3>Existing note</h3>
          <p>{{ existingConcern.note || 'No note was added.' }}</p>
        </div>
      </form>
    </template>

    <template #footer>
      <div class="dialog-actions">
        <UButton
          color="neutral"
          variant="link"
          :disabled="saving"
          @click="open = false"
        >
          Cancel
        </UButton>

        <UButton
          type="submit"
          :form="formId"
          leading-icon="i-lucide-check"
          variant="subtle"
          :disabled="loading || Boolean( loadError ) || !selectedCard || Boolean( noteError )"
          :loading="saving"
        >
          Save issue
        </UButton>
      </div>
    </template>
  </UModal>
</template>

<style scoped>
.card-quality-form {
  display: grid;
  gap: 1.25rem;
}

.card-quality-existing {
  padding: 1rem;
  background: var(--ui-bg-muted);
  border: 1px solid var(--ui-border);
  border-radius: var(--ui-radius);
  overflow-wrap: anywhere;
}

.card-quality-existing h3 {
  margin: 0 0 0.5rem;
  font-weight: 600;
}

.card-quality-existing p {
  margin: 0;
  white-space: pre-wrap;
}
</style>
