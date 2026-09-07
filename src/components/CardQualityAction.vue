<script setup>
import { computed, onBeforeUnmount, ref, useId, watch } from 'vue';

import {
  cardQualityKinds,
  maximumCardQualityNoteLength
} from '../card-quality/options';
import { collectClozeGroups } from '../cloze/documents';
import { useCardQuality } from '../composables/useCardQuality';
import { conceptLibraryErrorMessage } from '../composables/useConceptLibrary';
import { collectImageOcclusionGroups } from '../image-occlusion/documents';

const props = defineProps({
  card: {
    type: Object,
    required: true
  },
  disabled: {
    type: Boolean,
    default: false
  },
  formName: {
    type: String,
    required: true
  }
});

const { flagCard, getQueue } = useCardQuality();
const formId = useId();
const existingConcern = ref( null );
const kind = ref( 'ambiguous' );
const note = ref( '' );
const open = ref( false );
const saved = ref( false );
const saveError = ref( '' );
const saving = ref( false );
const target = ref( null );
let requestSequence = 0;

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

watch( open, ( isOpen ) => {
  requestSequence += 1;

  if ( !isOpen ) {
    return;
  }

  target.value = {
    id: props.card.id,
    description: `${ props.card.conceptTitle } · ${ retrievalFormName() }`
  };
  kind.value = 'ambiguous';
  note.value = '';
  saved.value = false;
  saveError.value = '';
  existingConcern.value = null;
});

watch( kind, () => {
  saveError.value = '';
  existingConcern.value = null;
});

watch( () => props.card.id, () => {
  requestSequence += 1;
  open.value = false;
  saved.value = false;
  saving.value = false;
});

onBeforeUnmount( () => {
  requestSequence += 1;
});

function retrievalFormName() {
  const card = props.card;
  let groupIndex = -1;

  if ( card.cloze ) {
    groupIndex = collectClozeGroups( card.content.prompt )
      .findIndex( ( group ) => group.id === card.cloze.groupId );
  } else if ( card.imageOcclusion ) {
    groupIndex = collectImageOcclusionGroups( card.content.prompt )
      .findIndex( ( group ) => group.id === card.imageOcclusion.groupId );
  }

  return groupIndex < 0 ? props.formName : `${ props.formName } ${ groupIndex + 1 }`;
}

async function saveIssue() {
  if ( saving.value || !open.value || !target.value || noteError.value ) {
    return;
  }

  const request = ++requestSequence;
  const cardId = target.value.id;
  const requestedKind = kind.value;

  saving.value = true;
  saveError.value = '';
  existingConcern.value = null;

  try {
    await flagCard( cardId, requestedKind, note.value );

    if ( request === requestSequence ) {
      saved.value = true;
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
    :description="target?.description"
    :dismissible="!saving"
    :close="!saving"
    :ui="{ content: 'card-quality-dialog rounded-md', description: 'wrap-anywhere' }"
    scrollable
  >
    <UButton
      :leading-icon="saved ? 'i-lucide-check' : 'i-lucide-flag'"
      color="neutral"
      variant="link"
      size="sm"
      class="study-card-issue"
      :disabled="disabled"
      :title="saved ? 'Issue saved. Report another card issue.' : undefined"
    >
      Card issue
    </UButton>

    <template #body>
      <form
        :id="formId"
        class="card-quality-form"
        novalidate
        @submit.prevent="saveIssue"
      >
        <UFormField label="Issue" required>
          <USelect
            v-model="kind"
            :items="cardQualityKinds"
            :disabled="saving"
            class="w-full"
            autofocus
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
          :disabled="Boolean( noteError )"
          :loading="saving"
        >
          Save issue
        </UButton>
      </div>
    </template>
  </UModal>

  <span
    role="status"
    class="sr-only"
  >
    {{ saved ? 'Card issue saved.' : '' }}
  </span>
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
