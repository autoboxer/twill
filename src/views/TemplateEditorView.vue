<script setup>
import {
  computed,
  onBeforeUnmount,
  onMounted,
  reactive,
  ref,
  watch
} from 'vue';
import {
  onBeforeRouteLeave,
  onBeforeRouteUpdate,
  useRoute,
  useRouter
} from 'vue-router';

import ContentState from '../components/ContentState.vue';
import PageHeader from '../components/PageHeader.vue';
import TemplateMarkupEditor from '../components/TemplateMarkupEditor.vue';
import TemplatePreview from '../components/TemplatePreview.vue';
import TemplateVisualSideEditor from '../components/TemplateVisualSideEditor.vue';
import { COMMAND_IDS } from '../commands/registry';
import { useAuthoringDraft } from '../composables/useAuthoringDraft';
import { useCommandHandler } from '../composables/useCommands';
import { conceptLibraryErrorMessage } from '../composables/useConceptLibrary';
import { useTemplateLibrary } from '../composables/useTemplateLibrary';
import {
  cloneTemplateEditorState,
  createTemplateEditorState,
  templateEditorStateKey
} from '../drafts/templateDraft';
import {
  cloneTemplateContent,
  createDefaultTemplateContent,
  MAXIMUM_TEMPLATE_BLOCKS_PER_SIDE
} from '../templates/defaults';

const route = useRoute();
const router = useRouter();
const {
  clearError,
  createTemplate,
  error,
  getTemplate,
  isPending,
  updateTemplate
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
} = useAuthoringDraft( 'template' );

const form = reactive({
  content: createDefaultTemplateContent(),
  name: ''
});
const conflictMessage = ref( '' );
const draftCleanupError = ref( '' );
const editorResolved = ref( false );
const initialLoading = ref( true );
const leaveDialogOpen = ref( false );
const leaveError = ref( '' );
const leaveLoading = ref( false );
const loadError = ref( '' );
const recoveryBusy = ref( false );
const recoveryDraft = ref( null );
const recoveryError = ref( '' );
const recoveryOpen = ref( false );
const saveAsCopy = ref( false );
const saveAttempted = ref( false );
const saveInProgress = ref( false );
const savedTemplate = ref( null );
const savedSnapshot = ref( '' );
const targetUnavailable = ref( false );
let allowNavigation = false;
let canonicalBaseChangeId = null;
let leaveResolution = null;
let loadRequestSequence = 0;

const templateId = computed( () => route.params.templateId ?? '' );
const isEditing = computed( () => Boolean( templateId.value ) );
const savesExistingTemplate = computed( () => isEditing.value && !saveAsCopy.value );
const pageTitle = computed( () => {
  if ( saveAsCopy.value ) {
    return 'Create template copy';
  }

  return isEditing.value ? 'Edit template' : 'New template';
});
const submitLabel = computed( () => (
  savesExistingTemplate.value ? 'Save template' : 'Create template'
) );
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
    return 'Template no longer exists';
  }

  if ( recoveryChanged.value ) {
    return 'Saved template changed';
  }

  return 'Restore template draft?';
});
const recoveryDescription = computed( () => {
  if ( recoveryMissing.value ) {
    return 'Restore this draft as a new template, or discard it.';
  }

  if ( recoveryChanged.value ) {
    return 'This draft began before the saved template changed. Restore it as a new template to preserve both versions, or discard it and edit the saved version.';
  }

  return 'Twill found unfinished work saved on this device.';
});
const restoreLabel = computed( () => (
  recoveryChanged.value || recoveryMissing.value ? 'Restore as new' : 'Restore draft'
) );

const nameError = computed( () => {
  if ( !saveAttempted.value ) {
    return '';
  }

  if ( !form.name.trim() ) {
    return 'Template name cannot be empty.';
  }

  return '';
});

const visualFrontError = computed( () => visualSideError( form.content.visual.front.blocks ) );
const visualAnswerError = computed( () => visualSideError( form.content.visual.answer.blocks ) );
const customFrontError = computed( () => customSideError( form.content.custom.frontHtml ) );
const customAnswerError = computed( () => customSideError( form.content.custom.answerHtml ) );
const validationError = computed( () => {
  if ( !saveAttempted.value ) {
    return '';
  }

  return nameError.value
    || visualFrontError.value
    || visualAnswerError.value
    || customFrontError.value
    || customAnswerError.value;
});
const formValid = computed( () => {
  return Boolean( form.name.trim() )
    && !visualFrontError.value
    && !visualAnswerError.value
    && !customFrontError.value
    && !customAnswerError.value;
});
const hasChanges = computed( () => {
  return templateEditorStateKey( form ) !== savedSnapshot.value;
});
const saveCommand = useCommandHandler( COMMAND_IDS.templateSave, {
  enabled: computed( () => (
    !initialLoading.value
    && !loadError.value
    && !isPending.value
    && !saveInProgress.value
    && !recoveryOpen.value
    && !savedTemplate.value
    && hasChanges.value
  ) ),
  execute: saveTemplate
});

watch( templateId, loadTemplate, { immediate: true });
watch( form, templateStateChanged, { deep: true });
onBeforeRouteLeave( protectNavigation );
onBeforeRouteUpdate( protectNavigation );

onMounted( () => {
  window.addEventListener( 'beforeunload', warnBeforeWindowClose );
  document.addEventListener( 'visibilitychange', flushHiddenDraft );
});

onBeforeUnmount( () => {
  loadRequestSequence += 1;
  window.removeEventListener( 'beforeunload', warnBeforeWindowClose );
  document.removeEventListener( 'visibilitychange', flushHiddenDraft );

  if ( leaveResolution ) {
    leaveResolution( false );
  }
});

async function loadTemplate() {
  const request = ++loadRequestSequence;
  const requestedTemplateId = templateId.value;

  allowNavigation = false;
  clearError();
  conflictMessage.value = '';
  draftCleanupError.value = '';
  editorResolved.value = false;
  initialLoading.value = true;
  loadError.value = '';
  recoveryDraft.value = null;
  recoveryError.value = '';
  recoveryOpen.value = false;
  saveAsCopy.value = false;
  saveAttempted.value = false;
  savedTemplate.value = null;
  targetUnavailable.value = false;

  try {
    const templateRequest = requestedTemplateId
      ? getTemplate( requestedTemplateId )
        .then( ( value ) => ({ value }) )
        .catch( ( cause ) => ({ cause }) )
      : Promise.resolve({ value: null });
    const [ templateResult, existingDraft ] = await Promise.all([
      templateRequest,
      loadDraft( requestedTemplateId || null )
    ]);

    if ( request !== loadRequestSequence ) {
      return;
    }

    if ( templateResult.cause && existingDraft?.targetStatus !== 'missing' ) {
      throw templateResult.cause;
    }

    const template = templateResult.value ?? null;
    const canonicalState = createTemplateEditorState( template );

    canonicalBaseChangeId = template?.lastChangeId ?? null;
    targetUnavailable.value = Boolean( templateResult.cause );
    savedSnapshot.value = templateEditorStateKey( canonicalState );
    applyEditorState( canonicalState );
    startDraft({
      targetId: requestedTemplateId || null,
      baseChangeId: existingDraft?.baseChangeId
        ?? template?.lastChangeId
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

function applyEditorState( state ) {
  const normalizedState = cloneTemplateEditorState( state );

  form.name = normalizedState.name;
  form.content = normalizedState.content;
}

async function saveTemplate() {
  if ( saveInProgress.value ) {
    return;
  }

  if ( savedTemplate.value ) {
    await finishSavedTemplate();
    return;
  }

  saveAttempted.value = true;
  clearError();
  conflictMessage.value = '';

  if ( !formValid.value || isPending.value ) {
    return;
  }

  saveInProgress.value = true;

  try {
    await flushDraft();
  } catch {
    saveInProgress.value = false;
    return;
  }

  const input = {
    name: form.name,
    content: cloneTemplateContent( form.content )
  };

  try {
    if ( savesExistingTemplate.value && hasChanges.value ) {
      const currentDraft = await refreshDraft();

      if ( currentDraft?.targetStatus !== 'current' ) {
        saveAsCopy.value = true;
        savedSnapshot.value = templateEditorStateKey( createTemplateEditorState() );
        conflictMessage.value = currentDraft?.targetStatus === 'missing'
          ? 'The saved template was removed while you were editing. Create this draft as a new template to preserve it.'
          : 'The saved template changed while you were editing. Create this draft as a new template to preserve both versions.';

        return;
      }
    }

    const saved = savesExistingTemplate.value
      ? await updateTemplate({ id: templateId.value, ...input })
      : await createTemplate( input );

    savedTemplate.value = saved;
    await finishSavedTemplate();
  } catch {
    // Error state is handled by the composable.
  } finally {
    saveInProgress.value = false;
  }
}

async function finishSavedTemplate() {
  if ( !savedTemplate.value ) {
    return;
  }

  draftCleanupError.value = '';

  try {
    await discardDraft();
  } catch {
    draftCleanupError.value = 'The template was saved, but its local draft could not be cleared.';
    return;
  }

  const saved = savedTemplate.value;
  const changedIdentity = saved.id !== templateId.value;

  saveAttempted.value = false;
  savedTemplate.value = null;

  if ( changedIdentity ) {
    allowNavigation = true;

    await router.replace({
      name: 'template-edit',
      params: { templateId: saved.id }
    });
    return;
  }

  savedSnapshot.value = templateEditorStateKey( createTemplateEditorState( saved ) );
  applyEditorState( saved );
  canonicalBaseChangeId = saved.lastChangeId;
  saveAsCopy.value = false;
  conflictMessage.value = '';
  startDraft({
    targetId: saved.id,
    baseChangeId: saved.lastChangeId
  });
}

function templateStateChanged() {
  if ( !editorResolved.value || savedTemplate.value ) {
    return;
  }

  const state = cloneTemplateEditorState( form );

  if ( templateEditorStateKey( state ) !== savedSnapshot.value ) {
    scheduleDraftSave( state );
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
    savedSnapshot.value = templateEditorStateKey( createTemplateEditorState() );
  }

  applyEditorState( recoveryDraft.value.payload );
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
      await router.replace({ name: 'templates' });
      return;
    }

    startDraft({
      targetId: templateId.value || null,
      baseChangeId: canonicalBaseChangeId
    });
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

  if ( !hasChanges.value && !hasPendingPersistence.value ) {
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
  if ( !hasChanges.value && !hasPendingPersistence.value ) {
    return;
  }

  event.preventDefault();
  event.returnValue = '';
}

function flushHiddenDraft() {
  if ( document.visibilityState === 'hidden' && hasChanges.value ) {
    void flushDraft().catch( () => undefined );
  }
}

function visualSideError( blocks ) {
  if ( blocks.length > MAXIMUM_TEMPLATE_BLOCKS_PER_SIDE ) {
    return `Each side can contain up to ${ MAXIMUM_TEMPLATE_BLOCKS_PER_SIDE } blocks.`;
  }

  if ( !blocks.length ) {
    return 'Each side needs at least one field.';
  }

  if ( !blocks.some( ( block ) => block.type === 'field' ) ) {
    return 'Each side needs at least one field.';
  }

  if ( blocks.some( ( block ) => block.type === 'text' && !block.text.trim() ) ) {
    return 'Text blocks cannot be empty.';
  }

  return '';
}

function customSideError( source ) {
  if ( !source.trim() ) {
    return 'Each custom side needs HTML and at least one concept field.';
  }

  if ( !/{{\s*(title|prompt|answer)\s*}}/.test( source ) ) {
    return 'Each custom side needs at least one concept field.';
  }

  return '';
}

function cancel() {
  router.push({ name: 'templates' });
}
</script>

<template>
  <div
    class="page template-editor-page"
    data-twill-page="template-editor"
  >
    <PageHeader :title="pageTitle">
      <template #actions>
        <UButton
          :to="{ name: 'templates' }"
          leading-icon="i-lucide-arrow-left"
          color="neutral"
          variant="link"
        >
          Templates
        </UButton>
      </template>
    </PageHeader>

    <ContentState
      v-if="initialLoading"
      kind="loading"
      title="Loading template editor"
    />

    <ContentState
      v-else-if="loadError"
      kind="error"
      title="Template could not be loaded"
      :description="loadError"
    >
      <template #actions>
        <UButton
          leading-icon="i-lucide-refresh-cw"
          @click="loadTemplate"
        >
          Retry
        </UButton>

        <UButton
          :to="{ name: 'templates' }"
          color="neutral"
          variant="link"
        >
          Back to templates
        </UButton>
      </template>
    </ContentState>

    <div
      v-if="!initialLoading && !loadError && ( draftStatusMessage || draftError || conflictMessage || draftCleanupError )"
      class="draft-persistence"
      aria-live="polite"
    >
      <UAlert
        v-if="conflictMessage"
        :description="conflictMessage"
        title="Saved template changed"
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
            @click="savedTemplate ? finishSavedTemplate() : retryDraft()"
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

    <form
      v-if="!initialLoading && !loadError"
      class="template-editor"
      novalidate
      @submit.prevent="saveTemplate"
    >
      <UAlert
        v-if="error || validationError"
        :description="error || validationError"
        icon="i-lucide-circle-alert"
        color="error"
        variant="soft"
      />

      <fieldset
        class="template-editor__fields"
        :disabled="isPending || saveInProgress"
      >
        <section
          class="editor-section template-editor__basics"
          data-twill-editor-section="template-basics"
        >
          <div class="editor-section__heading">
            <div>
              <h2>Template</h2>
              <p>Name the layout and choose how it is edited.</p>
            </div>
          </div>

          <UFormField
            label="Name"
            :error="nameError"
            :hint="`${ form.name.length } / 80`"
            required
          >
            <UInput
              v-model="form.name"
              placeholder="Template name"
              :maxlength="80"
              autocomplete="off"
              autofocus
              class="w-full"
              size="xl"
            />
          </UFormField>

          <div>
            <span class="template-mode-label">Editor</span>

            <div
              class="segmented-control template-mode-control"
              aria-label="Template editor mode"
            >
              <button
                type="button"
                class="segmented-control__button"
                :class="{
                  'segmented-control__button--active': form.content.mode === 'visual'
                }"
                :aria-pressed="form.content.mode === 'visual'"
                @click="form.content.mode = 'visual'"
              >
                <UIcon name="i-lucide-layout-template" />
                Visual
              </button>

              <button
                type="button"
                class="segmented-control__button"
                :class="{
                  'segmented-control__button--active': form.content.mode === 'custom'
                }"
                :aria-pressed="form.content.mode === 'custom'"
                @click="form.content.mode = 'custom'"
              >
                <UIcon name="i-lucide-code-xml" />
                HTML & CSS
              </button>
            </div>

            <p class="template-mode-description">
              Both drafts are retained when you switch modes.
            </p>
          </div>
        </section>

        <div class="template-workspace">
          <section
            class="editor-section template-design"
            data-twill-editor-section="template-design"
          >
            <div class="editor-section__heading">
              <div>
                <h2>{{ form.content.mode === 'visual' ? 'Layout' : 'Markup' }}</h2>
                <p v-if="form.content.mode === 'visual'">
                  Arrange concept fields and optional text for each side.
                </p>
                <p v-else>
                  Use HTML for structure and CSS for presentation.
                </p>
              </div>
            </div>

            <template v-if="form.content.mode === 'visual'">
              <div class="template-visual-settings">
                <div>
                  <span>Alignment</span>

                  <div
                    class="segmented-control template-alignment-control"
                    aria-label="Template text alignment"
                  >
                    <button
                      type="button"
                      class="segmented-control__button"
                      :class="{
                        'segmented-control__button--active': form.content.visual.appearance.alignment === 'left'
                      }"
                      :aria-pressed="form.content.visual.appearance.alignment === 'left'"
                      @click="form.content.visual.appearance.alignment = 'left'"
                    >
                      Left
                    </button>

                    <button
                      type="button"
                      class="segmented-control__button"
                      :class="{
                        'segmented-control__button--active': form.content.visual.appearance.alignment === 'center'
                      }"
                      :aria-pressed="form.content.visual.appearance.alignment === 'center'"
                      @click="form.content.visual.appearance.alignment = 'center'"
                    >
                      Center
                    </button>
                  </div>
                </div>

                <label class="template-label-toggle">
                  <USwitch v-model="form.content.visual.appearance.showFieldLabels" />
                  <span>Show field labels</span>
                </label>
              </div>

              <div class="template-side-grid">
                <TemplateVisualSideEditor
                  v-model="form.content.visual.front.blocks"
                  label="Front"
                  :error="saveAttempted ? visualFrontError : ''"
                />

                <TemplateVisualSideEditor
                  v-model="form.content.visual.answer.blocks"
                  label="Answer"
                  :error="saveAttempted ? visualAnswerError : ''"
                />
              </div>
            </template>

            <template v-else>
              <UAlert
                title="Restricted markup"
                description="Unsupported tags and attributes are removed. JavaScript and external resource loading are not allowed."
                icon="i-lucide-shield-check"
                color="neutral"
                variant="soft"
              />

              <div class="template-markup-grid">
                <TemplateMarkupEditor
                  v-model="form.content.custom.frontHtml"
                  label="Front HTML"
                  description="Insert at least one concept field."
                  :error="saveAttempted ? customFrontError : ''"
                />

                <TemplateMarkupEditor
                  v-model="form.content.custom.answerHtml"
                  label="Answer HTML"
                  description="Insert at least one concept field."
                  :error="saveAttempted ? customAnswerError : ''"
                />

                <TemplateMarkupEditor
                  v-model="form.content.custom.css"
                  label="CSS"
                  description="Styles are isolated to the card preview."
                  :rows="12"
                  :show-fields="false"
                  class="template-css-editor"
                />
              </div>
            </template>
          </section>

          <TemplatePreview :content="form.content" />
        </div>
      </fieldset>

      <footer class="editor-actions template-editor__actions">
        <UButton
          type="button"
          color="neutral"
          variant="link"
          :disabled="isPending || saveInProgress"
          @click="cancel"
        >
          Cancel
        </UButton>

        <UButton
          type="submit"
          leading-icon="i-lucide-check"
          :disabled="!hasChanges || Boolean( savedTemplate )"
          :loading="isPending || saveInProgress"
          :aria-keyshortcuts="saveCommand.ariaKeyshortcuts"
          :title="saveCommand.tooltip"
          size="lg"
        >
          {{ submitLabel }}
        </UButton>
      </footer>
    </form>

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
      title="Leave template editor?"
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
