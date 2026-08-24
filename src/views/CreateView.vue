<script setup>
import {
  computed,
  onBeforeUnmount,
  onMounted,
  ref,
  watch
} from 'vue';
import {
  onBeforeRouteLeave,
  onBeforeRouteUpdate,
  useRoute,
  useRouter
} from 'vue-router';

import ConceptForm from '../components/ConceptForm.vue';
import ContentState from '../components/ContentState.vue';
import OrganizationManager from '../components/OrganizationManager.vue';
import PageHeader from '../components/PageHeader.vue';
import { COMMAND_IDS } from '../commands/registry';
import { useAuthoringDraft } from '../composables/useAuthoringDraft';
import { useCommandHandler } from '../composables/useCommands';
import { useDeferredEdits } from '../composables/useDeferredEdits';
import {
  conceptLibraryErrorMessage,
  useConceptLibrary
} from '../composables/useConceptLibrary';
import { useTemplateLibrary } from '../composables/useTemplateLibrary';
import {
  cloneConceptEditorState,
  conceptDraftMediaIds,
  conceptEditorStateKey,
  createConceptEditorState
} from '../drafts/conceptDraft';
import { markStudyConceptChanged } from '../study/resume';

const route = useRoute();
const router = useRouter();
const {
  clearError,
  createConcept,
  error,
  getLibrary,
  isPending,
  updateConcept
} = useConceptLibrary();
const {
  clearError: clearLoadError,
  getConcept: loadConcept,
  getLibrary: loadLibrary
} = useConceptLibrary();
const {
  clearError: clearTemplateLoadError,
  getTemplates
} = useTemplateLibrary();
const {
  discard: discardDraft,
  draft,
  error: draftError,
  flush: flushDraft,
  hasPendingPersistence,
  load: loadDraft,
  refresh: refreshDraft,
  retry: retryDraft,
  scheduleDelete: scheduleDraftDelete,
  scheduleSave: scheduleDraftSave,
  start: startDraft,
  status: draftStatus
} = useAuthoringDraft( 'concept' );
const {
  getDeferredEdits,
  removeDeferredEdit
} = useDeferredEdits();

const concept = ref( null );
const conceptForm = ref( null );
const conflictMessage = ref( '' );
const draftCleanupError = ref( '' );
const deferredEditItem = ref( null );
const deferredEditQueue = ref([]);
const deferredWorkflowError = ref( '' );
const deferredWorkflowPending = ref( false );
const editorResolved = ref( false );
const editorState = ref( null );
const initialLoading = ref( true );
const isModified = ref( false );
const leaveDialogOpen = ref( false );
const leaveError = ref( '' );
const leaveLoading = ref( false );
const loadError = ref( '' );
const library = ref({
  archivedCount: 0,
  concepts: [],
  decks: [],
  tags: []
});
const organizationManagerOpen = ref( false );
const recoveryBusy = ref( false );
const recoveryDraft = ref( null );
const recoveryError = ref( '' );
const recoveryOpen = ref( false );
const saveAsCopy = ref( false );
const saveInProgress = ref( false );
const savedConcept = ref( null );
const targetUnavailable = ref( false );
const templates = ref([]);
let allowNavigation = false;
let canonicalEditorState = createConceptEditorState();
let canonicalStateKey = conceptEditorStateKey( canonicalEditorState );
let leaveResolution = null;
let loadRequestSequence = 0;

const conceptId = computed( () => route.params.conceptId ?? '' );
const isEditing = computed( () => Boolean( conceptId.value ) );
const isDeferredEdit = computed( () => (
  isEditing.value && route.query.deferred === '1'
) );
const deferredTargetUnavailable = computed( () => (
  isDeferredEdit.value
  && deferredEditItem.value
  && deferredEditItem.value.targetStatus !== 'current'
) );
const savesExistingConcept = computed( () => isEditing.value && !saveAsCopy.value );
const editorDisabled = computed( () => (
  isPending.value
  || saveInProgress.value
  || deferredWorkflowPending.value
  || Boolean( savedConcept.value )
) );
const pageTitle = computed( () => {
  if ( saveAsCopy.value ) {
    return 'Create concept copy';
  }

  if ( isDeferredEdit.value ) {
    return 'Edit queued concept';
  }

  return isEditing.value ? 'Edit concept' : 'Create concept';
});
const deferredProgressLabel = computed( () => {
  const position = deferredEditQueue.value.findIndex( ( item ) => (
    item.conceptId === conceptId.value
  ) );

  if ( position < 0 ) {
    return 'Queued edit';
  }

  return `Queued edit ${ position + 1 } of ${ deferredEditQueue.value.length }`;
});
const deferredUnavailableDescription = computed( () => ({
  changed: 'This concept changed after it was queued, so Twill did not open it for editing.',
  archived: 'This concept was archived after it was queued, so Twill did not open it for editing.',
  missing: 'This concept was removed after it was queued, so Twill did not open it for editing.'
})[ deferredEditItem.value?.targetStatus ] ?? 'This queued concept is no longer available for editing.' );
const draftStatusMessage = computed( () => {
  if ( draftStatus.value === 'dirty' ) {
    return 'Waiting to save draft…';
  }

  if ( draftStatus.value === 'saving' ) {
    return 'Saving draft…';
  }

  if ( draftStatus.value === 'saved' ) {
    return 'Draft saved locally.';
  }

  return '';
});
const recoveryChanged = computed( () => (
  recoveryDraft.value?.targetStatus === 'changed'
) );
const recoveryMissing = computed( () => (
  recoveryDraft.value?.targetStatus === 'missing'
) );
const recoveryTitle = computed( () => {
  if ( recoveryMissing.value ) {
    return 'Concept no longer exists';
  }

  if ( recoveryChanged.value ) {
    return 'Saved concept changed';
  }

  return 'Restore concept draft?';
});
const recoveryDescription = computed( () => {
  if ( recoveryMissing.value ) {
    return 'Restore this draft as a new concept, or discard it.';
  }

  if ( recoveryChanged.value ) {
    return 'This draft began before the saved concept changed. Restore it as a new concept to preserve both versions, or discard it and edit the saved version.';
  }

  return 'Twill found unfinished work saved on this device.';
});
const restoreLabel = computed( () => (
  recoveryChanged.value || recoveryMissing.value ? 'Restore as new' : 'Restore draft'
) );
const saveCommand = useCommandHandler( COMMAND_IDS.conceptSave, {
  enabled: computed( () => (
    !initialLoading.value
    && !loadError.value
    && !deferredTargetUnavailable.value
    && !isPending.value
    && !saveInProgress.value
    && !deferredWorkflowPending.value
    && !recoveryOpen.value
    && !savedConcept.value
  ) ),
  execute: () => conceptForm.value?.submit()
});

watch( conceptId, loadData, { immediate: true });
onBeforeRouteLeave( protectNavigation );
onBeforeRouteUpdate( protectNavigation );

onMounted( () => {
  window.addEventListener( 'beforeunload', warnBeforeWindowClose );
  document.addEventListener( 'visibilitychange', flushHiddenDraft );
});

onBeforeUnmount( () => {
  window.removeEventListener( 'beforeunload', warnBeforeWindowClose );
  document.removeEventListener( 'visibilitychange', flushHiddenDraft );

  if ( leaveResolution ) {
    leaveResolution( false );
  }
});

async function loadData() {
  const request = ++loadRequestSequence;
  const requestedConceptId = conceptId.value;
  const requestedDeferredEdit = Boolean(
    requestedConceptId && route.query.deferred === '1'
  );

  allowNavigation = false;
  clearError();
  clearLoadError();
  clearTemplateLoadError();
  conflictMessage.value = '';
  draftCleanupError.value = '';
  deferredEditItem.value = null;
  deferredEditQueue.value = [];
  deferredWorkflowError.value = '';
  deferredWorkflowPending.value = false;
  editorResolved.value = false;
  isModified.value = false;
  initialLoading.value = true;
  loadError.value = '';
  recoveryDraft.value = null;
  recoveryError.value = '';
  recoveryOpen.value = false;
  saveAsCopy.value = false;
  savedConcept.value = null;
  targetUnavailable.value = false;

  try {
    const conceptRequest = requestedConceptId
      ? loadConcept( requestedConceptId )
        .then( ( value ) => ({ value }) )
        .catch( ( cause ) => ({ cause }) )
      : Promise.resolve({ value: null });
    const deferredQueueRequest = requestedDeferredEdit
      ? getDeferredEdits()
      : Promise.resolve({ items: [] });
    const [
      snapshot,
      conceptResult,
      templateCatalog,
      existingDraft,
      queuedEdits
    ] = await Promise.all([
      loadLibrary( false ),
      conceptRequest,
      getTemplates(),
      loadDraft( requestedConceptId || null ),
      deferredQueueRequest
    ]);

    if ( request !== loadRequestSequence ) {
      return;
    }

    if ( requestedDeferredEdit ) {
      deferredEditQueue.value = queuedEdits.items;
      deferredEditItem.value = queuedEdits.items.find( ( item ) => (
        item.conceptId === requestedConceptId
      ) ) ?? null;

      if ( !deferredEditItem.value ) {
        allowNavigation = true;
        await router.replace({ name: 'study' });
        return;
      }

      if ( deferredEditItem.value.targetStatus !== 'current' ) {
        return;
      }
    }

    if ( conceptResult.cause && existingDraft?.targetStatus !== 'missing' ) {
      throw conceptResult.cause;
    }

    library.value = snapshot;
    concept.value = conceptResult.value ?? null;
    templates.value = templateCatalog.templates;
    targetUnavailable.value = Boolean( conceptResult.cause );

    canonicalEditorState = createConceptEditorState( concept.value );
    canonicalStateKey = conceptEditorStateKey( canonicalEditorState );
    editorState.value = cloneConceptEditorState( canonicalEditorState );

    startDraft({
      targetId: requestedConceptId || null,
      baseChangeId: existingDraft?.baseChangeId
        ?? concept.value?.lastChangeId
        ?? null
    }, existingDraft );

    if ( existingDraft ) {
      recoveryDraft.value = existingDraft;
      recoveryOpen.value = true;
    } else {
      editorResolved.value = true;
    }
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

async function refreshOrganizations() {
  try {
    library.value = await getLibrary( false );
  } catch {
    // Error state is handled by the composable.
  }
}

async function saveConcept( input ) {
  if ( saveInProgress.value ) {
    return;
  }

  if ( savedConcept.value ) {
    await finishSavedConcept();
    return;
  }

  saveInProgress.value = true;
  clearError();
  conflictMessage.value = '';

  try {
    await flushDraft();
  } catch {
    saveInProgress.value = false;
    return;
  }

  try {
    if ( savesExistingConcept.value && isModified.value ) {
      const currentDraft = await refreshDraft();

      if ( currentDraft?.targetStatus !== 'current' ) {
        saveAsCopy.value = true;
        canonicalStateKey = conceptEditorStateKey( createConceptEditorState() );
        conflictMessage.value = currentDraft?.targetStatus === 'missing'
          ? 'The saved concept was removed while you were editing. Create this draft as a new concept to preserve it.'
          : 'The saved concept changed while you were editing. Create this draft as a new concept to preserve both versions.';

        return;
      }
    }

    const saved = savesExistingConcept.value
      ? await updateConcept({ id: conceptId.value, ...input })
      : await createConcept( input );

    if ( isDeferredEdit.value && savesExistingConcept.value ) {
      markStudyConceptChanged( conceptId.value );
    }

    savedConcept.value = saved;
    await finishSavedConcept();
  } catch {
    // Error state is handled by the composable.
  } finally {
    saveInProgress.value = false;
  }
}

async function finishSavedConcept() {
  if ( !savedConcept.value ) {
    return;
  }

  draftCleanupError.value = '';

  try {
    await discardDraft();
  } catch {
    draftCleanupError.value = 'The concept was saved, but its local draft could not be cleared.';
    return;
  }

  isModified.value = false;

  if ( isDeferredEdit.value ) {
    await continueDeferredEditing();
    return;
  }

  allowNavigation = true;

  await router.replace({
    name: 'concept-detail',
    params: { conceptId: savedConcept.value.id }
  });
}

async function continueDeferredEditing() {
  deferredWorkflowError.value = '';
  deferredWorkflowPending.value = true;

  try {
    await removeDeferredEdit( conceptId.value );

    const queue = await getDeferredEdits();
    const nextItem = queue.items[ 0 ];

    allowNavigation = true;

    if ( nextItem?.targetStatus === 'current' ) {
      await router.replace({
        name: 'concept-edit',
        params: { conceptId: nextItem.conceptId },
        query: { deferred: '1' }
      });
    } else {
      await router.replace({ name: 'study' });
    }
  } catch ( cause ) {
    allowNavigation = false;
    deferredWorkflowError.value = conceptLibraryErrorMessage( cause );
  } finally {
    deferredWorkflowPending.value = false;
  }
}

async function skipDeferredEdit() {
  if ( deferredWorkflowPending.value ) {
    return;
  }

  deferredWorkflowError.value = '';
  deferredWorkflowPending.value = true;

  try {
    if ( isModified.value || hasPendingPersistence.value ) {
      await flushDraft();
    }

    deferredWorkflowPending.value = false;
    await continueDeferredEditing();
  } catch ( cause ) {
    deferredWorkflowError.value = cause.message
      || 'The queued edit could not be skipped safely.';
    deferredWorkflowPending.value = false;
  }
}

function conceptStateChanged( state ) {
  if ( !editorResolved.value || savedConcept.value ) {
    return;
  }

  const normalizedState = cloneConceptEditorState( state );
  const modified = conceptEditorStateKey( normalizedState ) !== canonicalStateKey;

  isModified.value = modified;

  if ( modified ) {
    scheduleDraftSave( normalizedState, conceptDraftMediaIds( normalizedState ) );
  } else if ( draft.value || hasPendingPersistence.value ) {
    scheduleDraftDelete();
  }
}

function restoreRecoveryDraft() {
  if ( !recoveryDraft.value ) {
    return;
  }

  saveAsCopy.value = recoveryChanged.value || recoveryMissing.value;

  if ( saveAsCopy.value ) {
    canonicalStateKey = conceptEditorStateKey( createConceptEditorState() );
  }

  editorState.value = cloneConceptEditorState( recoveryDraft.value.payload );
  editorResolved.value = true;
  recoveryOpen.value = false;
}

async function discardRecoveryDraft() {
  recoveryBusy.value = true;
  recoveryError.value = '';

  try {
    await discardDraft();

    if ( targetUnavailable.value ) {
      allowNavigation = true;
      await router.replace({ name: 'library' });
      return;
    }

    startDraft({
      targetId: conceptId.value || null,
      baseChangeId: concept.value?.lastChangeId ?? null
    });
    editorState.value = cloneConceptEditorState( canonicalEditorState );
    editorResolved.value = true;
    recoveryOpen.value = false;
  } catch ( cause ) {
    recoveryError.value = cause.message || 'The draft could not be discarded.';
  } finally {
    recoveryBusy.value = false;
  }
}

function protectNavigation() {
  if ( allowNavigation || !editorResolved.value ) {
    return recoveryOpen.value ? false : true;
  }

  if ( saveInProgress.value ) {
    return false;
  }

  if ( !isModified.value && !hasPendingPersistence.value ) {
    return true;
  }

  if ( leaveResolution ) {
    leaveResolution( false );
  }

  leaveError.value = '';
  leaveDialogOpen.value = true;

  return new Promise( ( resolve ) => {
    leaveResolution = resolve;
  });
}

function stayInEditor() {
  leaveDialogOpen.value = false;

  if ( leaveResolution ) {
    leaveResolution( false );
    leaveResolution = null;
  }
}

async function leaveEditor() {
  leaveLoading.value = true;
  leaveError.value = '';

  try {
    await flushDraft();
  } catch {
    leaveError.value = 'The latest changes could not be saved. Retry or stay in the editor.';
    leaveLoading.value = false;
    return;
  }

  leaveLoading.value = false;
  leaveDialogOpen.value = false;

  if ( leaveResolution ) {
    leaveResolution( true );
    leaveResolution = null;
  }
}

function warnBeforeWindowClose( event ) {
  if ( !isModified.value && !hasPendingPersistence.value ) {
    return;
  }

  event.preventDefault();
  event.returnValue = '';
}

function flushHiddenDraft() {
  if ( document.visibilityState === 'hidden' && isModified.value ) {
    void flushDraft().catch( () => undefined );
  }
}

function cancel() {
  if ( isDeferredEdit.value ) {
    void skipDeferredEdit();
    return;
  }

  if ( isEditing.value ) {
    router.push({
      name: 'concept-detail',
      params: { conceptId: conceptId.value }
    });
    return;
  }

  router.push({ name: 'library' });
}
</script>

<template>
  <div
    class="page editor-page"
    data-twill-page="concept-editor"
  >
    <PageHeader :title="pageTitle">
      <template #actions>
        <UButton
          leading-icon="i-lucide-arrow-left"
          color="neutral"
          variant="link"
          :loading="deferredWorkflowPending"
          :disabled="deferredWorkflowPending"
          @click="cancel"
        >
          {{ isDeferredEdit ? 'Skip' : 'Back' }}
        </UButton>
      </template>
    </PageHeader>

    <ContentState
      v-if="initialLoading"
      kind="loading"
      title="Loading concept editor"
    />

    <ContentState
      v-else-if="deferredTargetUnavailable"
      :title="`${ deferredEditItem.conceptTitle } was skipped`"
      :description="deferredUnavailableDescription"
    >
      <template #actions>
        <UButton
          leading-icon="i-lucide-list-restart"
          :loading="deferredWorkflowPending"
          @click="skipDeferredEdit"
        >
          Remove and continue
        </UButton>
      </template>
    </ContentState>

    <ContentState
      v-else-if="loadError"
      kind="error"
      :title="isEditing ? 'Concept could not be loaded' : 'Editor could not be loaded'"
      :description="loadError"
    >
      <template #actions>
        <UButton
          leading-icon="i-lucide-refresh-cw"
          @click="loadData"
        >
          Retry
        </UButton>
      </template>
    </ContentState>

    <UAlert
      v-if="!initialLoading && !loadError && isDeferredEdit && !deferredTargetUnavailable"
      class="deferred-edit-context"
      :title="deferredProgressLabel"
      description="Saving or skipping continues to the next queued concept."
      icon="i-lucide-list-ordered"
      color="primary"
      variant="subtle"
    />

    <UAlert
      v-if="deferredWorkflowError"
      class="deferred-edit-context"
      title="Queued editing needs attention"
      :description="deferredWorkflowError"
      icon="i-lucide-circle-alert"
      color="error"
      variant="subtle"
    >
      <template
        v-if="savedConcept"
        #actions
      >
        <UButton
          color="error"
          variant="subtle"
          size="sm"
          :loading="deferredWorkflowPending"
          @click="finishSavedConcept"
        >
          Retry
        </UButton>
      </template>
    </UAlert>

    <div
      v-if="!initialLoading
        && !loadError
        && !deferredTargetUnavailable
        && ( draftStatusMessage || draftError || conflictMessage || draftCleanupError )"
      class="draft-persistence"
      aria-live="polite"
    >
      <UAlert
        v-if="conflictMessage"
        :description="conflictMessage"
        title="Saved concept changed"
        icon="i-lucide-copy-plus"
        color="warning"
        variant="subtle"
      />

      <UAlert
        v-if="draftError || draftCleanupError"
        :description="draftCleanupError || draftError"
        title="Draft needs attention"
        icon="i-lucide-cloud-alert"
        color="error"
        variant="subtle"
      >
        <template #actions>
          <UButton
            size="sm"
            color="error"
            variant="subtle"
            @click="savedConcept ? finishSavedConcept() : retryDraft()"
          >
            Retry
          </UButton>
        </template>
      </UAlert>

      <p
        v-else-if="draftStatusMessage"
        class="draft-persistence__status"
      >
        <UIcon
          :name="draftStatus === 'saved' ? 'i-lucide-check' : 'i-lucide-loader-circle'"
          aria-hidden="true"
        />
        {{ draftStatusMessage }}
      </p>
    </div>

    <ConceptForm
      v-if="!initialLoading && !loadError && !deferredTargetUnavailable"
      ref="conceptForm"
      :mode="savesExistingConcept ? 'edit' : 'create'"
      :concept="concept"
      :editor-state="editorState"
      :disabled="editorDisabled"
      :decks="library.decks"
      :tags="library.tags"
      :templates="templates"
      :error="error"
      :loading="isPending || saveInProgress"
      :save-command="saveCommand"
      @cancel="cancel"
      @change="conceptStateChanged"
      @manage="organizationManagerOpen = true"
      @submit="saveConcept"
    />

    <OrganizationManager
      v-model:open="organizationManagerOpen"
      :decks="library.decks"
      :tags="library.tags"
      @changed="refreshOrganizations"
    />

    <UModal
      v-model:open="recoveryOpen"
      :title="recoveryTitle"
      :description="recoveryDescription"
      :dismissible="false"
      :close="false"
    >
      <template
        v-if="recoveryError"
        #body
      >
        <UAlert
          :description="recoveryError"
          icon="i-lucide-circle-alert"
          color="error"
          variant="subtle"
        />
      </template>

      <template #footer>
        <div class="dialog-actions dialog-actions--split">
          <UButton
            color="neutral"
            variant="link"
            :disabled="recoveryBusy"
            @click="discardRecoveryDraft"
          >
            Discard draft
          </UButton>

          <UButton
            leading-icon="i-lucide-rotate-ccw"
            :disabled="recoveryBusy"
            @click="restoreRecoveryDraft"
          >
            {{ restoreLabel }}
          </UButton>
        </div>
      </template>
    </UModal>

    <UModal
      v-model:open="leaveDialogOpen"
      title="Leave concept editor?"
      description="Your unfinished changes will remain saved as a draft on this device."
      :dismissible="!leaveLoading"
      @update:open="( open ) => { if ( !open && !leaveLoading ) stayInEditor() }"
    >
      <template
        v-if="leaveError"
        #body
      >
        <UAlert
          :description="leaveError"
          icon="i-lucide-circle-alert"
          color="error"
          variant="subtle"
        />
      </template>

      <template #footer>
        <div class="dialog-actions">
          <UButton
            color="neutral"
            variant="link"
            :disabled="leaveLoading"
            @click="stayInEditor"
          >
            Stay
          </UButton>

          <UButton
            leading-icon="i-lucide-log-out"
            :loading="leaveLoading"
            @click="leaveEditor"
          >
            Leave and keep draft
          </UButton>
        </div>
      </template>
    </UModal>
  </div>
</template>
