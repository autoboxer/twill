<script setup>
import { computed, ref, watch } from 'vue';

import { useConceptLibrary } from '../composables/useConceptLibrary';
import ConfirmDialog from './ConfirmDialog.vue';

const props = defineProps({
  decks: {
    type: Array,
    default: () => []
  },
  open: {
    type: Boolean,
    required: true
  },
  tags: {
    type: Array,
    default: () => []
  }
});

const emit = defineEmits([ 'changed', 'update:open' ]);

const {
  clearError,
  createDeck,
  createTag,
  deleteDeck,
  deleteTag,
  error,
  isPending,
  renameDeck,
  renameTag
} = useConceptLibrary();

const activeKind = ref( 'deck' );
const deleteTarget = ref( null );
const editingId = ref( '' );
const editingName = ref( '' );
const newName = ref( '' );
const validationError = ref( '' );

const isOpen = computed({
  get: () => props.open,
  set: ( value ) => emit( 'update:open', value )
});

const deleteDialogOpen = computed({
  get: () => Boolean( deleteTarget.value ),
  set: ( value ) => {
    if ( !value ) {
      deleteTarget.value = null;
    }
  }
});

const deleteTargetKind = computed( () => deleteTarget.value?.kind ?? activeKind.value );
const items = computed( () => activeKind.value === 'deck' ? props.decks : props.tags );
const itemLabel = computed( () => activeKind.value === 'deck' ? 'deck' : 'tag' );
const itemLabelTitle = computed( () => activeKind.value === 'deck' ? 'Deck' : 'Tag' );
const visibleError = computed( () => validationError.value || error.value );

watch( () => props.open, ( open ) => {
  if ( !open ) {
    cancelEditing();
    deleteTarget.value = null;
    newName.value = '';
  }

  clearMessages();
});

watch( activeKind, () => {
  cancelEditing();
  clearMessages();
});

function clearMessages() {
  validationError.value = '';
  clearError();
}

function startEditing( item ) {
  clearMessages();
  editingId.value = item.id;
  editingName.value = item.name;
}

function cancelEditing() {
  editingId.value = '';
  editingName.value = '';
}

function requestDelete( item ) {
  deleteTarget.value = {
    id: item.id,
    kind: activeKind.value,
    name: item.name
  };
}

async function createItem() {
  clearMessages();

  if ( !newName.value.trim() ) {
    validationError.value = `${ itemLabelTitle.value } name cannot be empty.`;
    return;
  }

  try {
    if ( activeKind.value === 'deck' ) {
      await createDeck( newName.value );
    } else {
      await createTag( newName.value );
    }

    newName.value = '';
    emit( 'changed' );
  } catch {
    // Error state is handled by the composable.
  }
}

async function saveEditing() {
  clearMessages();

  if ( !editingName.value.trim() ) {
    validationError.value = `${ itemLabelTitle.value } name cannot be empty.`;
    return;
  }

  try {
    if ( activeKind.value === 'deck' ) {
      await renameDeck( editingId.value, editingName.value );
    } else {
      await renameTag( editingId.value, editingName.value );
    }

    cancelEditing();
    emit( 'changed' );
  } catch {
    // Error state is handled by the composable.
  }
}

async function confirmDelete() {
  const target = deleteTarget.value;

  if ( !target ) {
    return;
  }

  clearMessages();

  try {
    if ( target.kind === 'deck' ) {
      await deleteDeck( target.id );
    } else {
      await deleteTag( target.id );
    }

    if ( editingId.value === target.id ) {
      cancelEditing();
    }

    deleteTarget.value = null;
    emit( 'changed' );
  } catch {
    // Error state is handled by the composable.
  }
}
</script>

<template>
  <UModal
    v-model:open="isOpen"
    title="Organize library"
    description="Create and manage the decks and tags used by concepts."
    scrollable
  >
    <template #body>
      <div class="organization-manager">
        <div
          class="segmented-control"
          aria-label="Organization type"
        >
          <button
            type="button"
            :class="{ 'segmented-control__button--active': activeKind === 'deck' }"
            class="segmented-control__button"
            @click="activeKind = 'deck'"
          >
            Decks
          </button>

          <button
            type="button"
            :class="{ 'segmented-control__button--active': activeKind === 'tag' }"
            class="segmented-control__button"
            @click="activeKind = 'tag'"
          >
            Tags
          </button>
        </div>

        <UAlert
          v-if="visibleError"
          :description="visibleError"
          icon="i-lucide-circle-alert"
          color="error"
          variant="soft"
        />

        <form
          class="organization-create"
          @submit.prevent="createItem"
        >
          <UFormField
            :label="`New ${ itemLabel }`"
            class="organization-create__field"
          >
            <UInput
              v-model="newName"
              :placeholder="`${ itemLabelTitle } name`"
              :maxlength="80"
              autocomplete="off"
              class="w-full"
              size="lg"
            />
          </UFormField>

          <UButton
            type="submit"
            leading-icon="i-lucide-plus"
            :loading="isPending"
          >
            Add
          </UButton>
        </form>

        <div
          v-if="items.length"
          class="organization-list"
        >
          <div
            v-for="item in items"
            :key="item.id"
            class="organization-row"
          >
            <form
              v-if="editingId === item.id"
              class="organization-row__edit"
              @submit.prevent="saveEditing"
            >
              <UInput
                v-model="editingName"
                :aria-label="`Rename ${ item.name }`"
                :maxlength="80"
                autocomplete="off"
                autofocus
                class="w-full"
              />

              <UButton
                type="submit"
                icon="i-lucide-check"
                :aria-label="`Save ${ item.name }`"
                :loading="isPending"
                square
              />

              <UButton
                type="button"
                icon="i-lucide-x"
                :aria-label="`Cancel renaming ${ item.name }`"
                color="neutral"
                variant="ghost"
                square
                @click="cancelEditing"
              />
            </form>

            <template v-else>
              <div class="organization-row__copy">
                <strong>{{ item.name }}</strong>
                <span>
                  {{ item.conceptCount }}
                  {{ item.conceptCount === 1 ? 'concept' : 'concepts' }}
                </span>
              </div>

              <div class="organization-row__actions">
                <UButton
                  icon="i-lucide-pencil"
                  :aria-label="`Rename ${ item.name }`"
                  color="neutral"
                  variant="ghost"
                  square
                  @click="startEditing( item )"
                />

                <UButton
                  icon="i-lucide-trash-2"
                  :aria-label="`Delete ${ item.name }`"
                  color="error"
                  variant="ghost"
                  square
                  @click="requestDelete( item )"
                />
              </div>
            </template>
          </div>
        </div>

        <div
          v-else
          class="organization-empty"
        >
          No {{ activeKind === 'deck' ? 'decks' : 'tags' }} yet.
        </div>
      </div>
    </template>
  </UModal>

  <ConfirmDialog
    v-model:open="deleteDialogOpen"
    :title="`Delete ${ deleteTargetKind }?`"
    :description="deleteTarget
      ? `${ deleteTarget.name } will be removed from every concept that uses it.`
      : ''"
    :confirm-label="`Delete ${ deleteTargetKind }`"
    :loading="isPending"
    @confirm="confirmDelete"
  />
</template>
