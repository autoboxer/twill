<script setup>
import { computed, nextTick, reactive, ref } from 'vue';

import { conceptLibraryErrorMessage } from '../composables/useConceptLibrary';
import { useCssSnippets } from '../composables/useCssSnippets';
import ConfirmDialog from './ConfirmDialog.vue';

const MAXIMUM_NAME_LENGTH = 80;
const MAXIMUM_SOURCE_BYTES = 100_000;

const {
  createSnippet,
  deleteSnippet,
  disableAllSnippets,
  enabledCount,
  initializeCssSnippets,
  isPending,
  loadError,
  loading,
  ready,
  safeMode,
  setSnippetEnabled,
  snippets,
  updateSnippet
} = useCssSnippets();

const actionError = ref( '' );
const actionPending = ref( '' );
const actionStatus = ref( '' );
const deleteTarget = ref( null );
const editorAttempted = ref( false );
const editorError = ref( '' );
const editorErrorNotice = ref( null );
const editorOpen = ref( false );
const editorPending = ref( false );
const editingId = ref( '' );
const form = reactive({
  name: '',
  source: ''
});

const deleteDialogOpen = computed({
  get: () => Boolean( deleteTarget.value ),
  set: ( value ) => {
    if ( !value ) {
      deleteTarget.value = null;
    }
  }
});
const editorNameError = computed( () => {
  if ( !editorAttempted.value ) {
    return '';
  }

  const name = form.name.trim();

  if ( !name ) {
    return 'Enter a snippet name.';
  }

  if ( Array.from( name ).length > MAXIMUM_NAME_LENGTH ) {
    return `Names cannot exceed ${ MAXIMUM_NAME_LENGTH } characters.`;
  }

  return '';
});
const editorSourceError = computed( () => {
  if ( !editorAttempted.value ) {
    return '';
  }

  if ( !form.source.trim() ) {
    return 'Enter CSS to save this snippet.';
  }

  if ( sourceBytes.value > MAXIMUM_SOURCE_BYTES ) {
    return 'CSS cannot exceed 100 KB.';
  }

  return '';
});
const editorTitle = computed( () => editingId.value ? 'Edit snippet' : 'New snippet' );
const formValid = computed( () => (
  Boolean( form.name.trim() )
  && Array.from( form.name.trim() ).length <= MAXIMUM_NAME_LENGTH
  && Boolean( form.source.trim() )
  && sourceBytes.value <= MAXIMUM_SOURCE_BYTES
) );
const sourceBytes = computed( () => new TextEncoder().encode( form.source ).length );

function openCreateEditor() {
  clearFeedback();
  editingId.value = '';
  form.name = '';
  form.source = '';
  editorAttempted.value = false;
  editorError.value = '';
  editorOpen.value = true;
}

function openEditEditor( snippet ) {
  clearFeedback();
  editingId.value = snippet.id;
  form.name = snippet.name;
  form.source = snippet.content.source;
  editorAttempted.value = false;
  editorError.value = '';
  editorOpen.value = true;
}

function closeEditor() {
  if ( !editorPending.value ) {
    editorOpen.value = false;
  }
}

async function saveSnippet() {
  editorAttempted.value = true;
  editorError.value = '';

  if ( !formValid.value || editorPending.value ) {
    return;
  }

  editorPending.value = true;

  try {
    if ( editingId.value ) {
      await updateSnippet( editingId.value, form.name, form.source );
      actionStatus.value = 'Snippet saved.';
    } else {
      await createSnippet( form.name, form.source );
      actionStatus.value = 'Snippet created. Enable it when ready.';
    }

    editorOpen.value = false;
  } catch ( cause ) {
    editorError.value = conceptLibraryErrorMessage( cause );
    await nextTick();
    editorErrorNotice.value?.$el?.scrollIntoView({ block: 'nearest' });
  } finally {
    editorPending.value = false;
  }
}

async function toggleSnippet( snippet, enabled ) {
  if ( isPending.value ) {
    return;
  }

  clearFeedback();
  actionPending.value = snippet.id;

  try {
    await setSnippetEnabled( snippet.id, enabled );
    actionStatus.value = enabled
      ? `${ snippet.name } enabled.`
      : `${ snippet.name } disabled.`;
  } catch ( cause ) {
    actionError.value = conceptLibraryErrorMessage( cause );
  } finally {
    actionPending.value = '';
  }
}

async function disableAll() {
  if ( isPending.value || !enabledCount.value ) {
    return;
  }

  clearFeedback();
  actionPending.value = 'disable-all';

  try {
    await disableAllSnippets();
    actionStatus.value = 'All snippets disabled.';
  } catch ( cause ) {
    actionError.value = conceptLibraryErrorMessage( cause );
  } finally {
    actionPending.value = '';
  }
}

function requestDelete( snippet ) {
  clearFeedback();
  deleteTarget.value = {
    id: snippet.id,
    name: snippet.name
  };
}

async function confirmDelete() {
  const target = deleteTarget.value;

  if ( !target || isPending.value ) {
    return;
  }

  clearFeedback();
  actionPending.value = target.id;

  try {
    await deleteSnippet( target.id );
    deleteTarget.value = null;
    actionStatus.value = `${ target.name } deleted.`;
  } catch ( cause ) {
    actionError.value = conceptLibraryErrorMessage( cause );
  } finally {
    actionPending.value = '';
  }
}

function clearFeedback() {
  actionError.value = '';
  actionStatus.value = '';
}

function formatSourceSize( source ) {
  const bytes = new TextEncoder().encode( source ).length;

  if ( bytes < 1_000 ) {
    return `${ bytes } ${ bytes === 1 ? 'byte' : 'bytes' }`;
  }

  return `${ ( bytes / 1_000 ).toFixed( bytes < 10_000 ? 1 : 0 ) } KB`;
}
</script>

<template>
  <div class="css-snippet-manager">
    <div class="css-snippet-manager__toolbar">
      <p>
        New snippets start disabled. Enable a snippet after reviewing its effect.
      </p>

      <UButton
        type="button"
        leading-icon="i-lucide-plus"
        :disabled="loading || !ready || Boolean( loadError ) || isPending"
        @click="openCreateEditor"
      >
        New snippet
      </UButton>
    </div>

    <UAlert
      description="External resources and executable CSS are rejected. Use documented data-twill-* targets and --twill-* variables."
      icon="i-lucide-shield-check"
      color="neutral"
      variant="subtle"
    />

    <UAlert
      v-if="safeMode"
      description="Enabled snippets are not being applied for this launch. Saved enablement has not changed."
      title="Safe mode is active"
      icon="i-lucide-shield-alert"
      color="warning"
      variant="subtle"
      class="css-snippet-manager__safe-mode"
    />

    <div
      v-if="loading || !ready"
      class="css-snippet-manager__state"
      role="status"
    >
      <UIcon name="i-lucide-loader-circle" />
      Loading snippets
    </div>

    <div
      v-else-if="loadError"
      class="css-snippet-manager__load-error"
    >
      <UAlert
        :description="loadError"
        icon="i-lucide-circle-alert"
        color="error"
        variant="subtle"
      />

      <UButton
        type="button"
        leading-icon="i-lucide-refresh-cw"
        color="neutral"
        variant="subtle"
        @click="initializeCssSnippets({ force: true })"
      >
        Retry
      </UButton>
    </div>

    <p
      v-else-if="!snippets.length"
      class="css-snippet-manager__empty"
    >
      No CSS snippets.
    </p>

    <div
      v-else
      class="css-snippet-list"
    >
      <div
        v-for="snippet in snippets"
        :key="snippet.id"
        class="css-snippet-row"
      >
        <USwitch
          :model-value="snippet.enabled"
          :disabled="isPending"
          :loading="actionPending === snippet.id"
          :aria-label="`${ snippet.enabled ? 'Disable' : 'Enable' } ${ snippet.name }`"
          @update:model-value="toggleSnippet( snippet, $event )"
        />

        <div class="css-snippet-row__copy">
          <strong>{{ snippet.name }}</strong>
          <span>
            {{ snippet.enabled ? 'Enabled' : 'Disabled' }}
            <span aria-hidden="true"> · </span>
            {{ formatSourceSize( snippet.content.source ) }}
          </span>
        </div>

        <div class="css-snippet-row__actions">
          <UButton
            type="button"
            color="neutral"
            variant="link"
            :disabled="isPending"
            @click="openEditEditor( snippet )"
          >
            Edit
          </UButton>

          <UButton
            type="button"
            color="error"
            variant="ghost"
            :disabled="isPending"
            @click="requestDelete( snippet )"
          >
            Delete
          </UButton>
        </div>
      </div>
    </div>

    <UAlert
      v-if="actionError"
      :description="actionError"
      icon="i-lucide-circle-alert"
      color="error"
      variant="subtle"
    />

    <footer class="settings-section-actions">
      <p
        class="settings-save-status"
        aria-live="polite"
      >
        {{ actionStatus }}
      </p>

      <UButton
        type="button"
        color="neutral"
        variant="link"
        :disabled="!enabledCount || isPending"
        :loading="actionPending === 'disable-all'"
        @click="disableAll"
      >
        Disable all
      </UButton>
    </footer>

    <UModal
      v-model:open="editorOpen"
      :title="editorTitle"
      description="CSS is validated by Twill before it is saved."
      :dismissible="!editorPending"
      :close="!editorPending"
      class="css-snippet-dialog"
      :ui="{
        overlay: 'css-snippet-dialog-overlay'
      }"
    >
      <template #body>
        <form
          id="css-snippet-editor"
          class="css-snippet-editor"
          novalidate
          @submit.prevent="saveSnippet"
        >
          <UFormField
            label="Name"
            :error="editorNameError"
            :hint="`${ Array.from( form.name ).length } / ${ MAXIMUM_NAME_LENGTH }`"
            required
          >
            <UInput
              v-model="form.name"
              placeholder="Snippet name"
              :maxlength="MAXIMUM_NAME_LENGTH"
              autocomplete="off"
              autofocus
              class="w-full"
              @input="editorError = ''"
            />
          </UFormField>

          <UFormField
            label="CSS"
            :error="editorSourceError"
            :hint="`${ formatSourceSize( form.source ) } / 100 KB`"
            required
          >
            <UTextarea
              v-model="form.source"
              placeholder=":root {&#10;  --twill-accent: #8b7cf6;&#10;}"
              :maxlength="MAXIMUM_SOURCE_BYTES"
              :rows="14"
              spellcheck="false"
              class="w-full css-snippet-editor__source"
              @input="editorError = ''"
            />
          </UFormField>

          <UAlert
            v-if="editorError"
            ref="editorErrorNotice"
            :description="editorError"
            icon="i-lucide-circle-alert"
            color="error"
            variant="subtle"
          />
        </form>
      </template>

      <template #footer>
        <div class="dialog-actions">
          <UButton
            type="button"
            color="neutral"
            variant="link"
            :disabled="editorPending"
            @click="closeEditor"
          >
            Cancel
          </UButton>

          <UButton
            type="submit"
            form="css-snippet-editor"
            leading-icon="i-lucide-check"
            :loading="editorPending"
          >
            {{ editingId ? 'Save changes' : 'Create snippet' }}
          </UButton>
        </div>
      </template>
    </UModal>

    <ConfirmDialog
      v-model:open="deleteDialogOpen"
      title="Delete CSS snippet?"
      :description="deleteTarget
        ? `${ deleteTarget.name } will be removed from this library.`
        : 'This snippet will be removed from this library.'"
      confirm-label="Delete snippet"
      :loading="Boolean( deleteTarget && actionPending === deleteTarget.id )"
      @confirm="confirmDelete"
    />
  </div>
</template>
