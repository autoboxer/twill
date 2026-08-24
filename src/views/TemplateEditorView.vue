<script setup>
import { computed, onBeforeUnmount, reactive, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';

import ContentState from '../components/ContentState.vue';
import PageHeader from '../components/PageHeader.vue';
import TemplateMarkupEditor from '../components/TemplateMarkupEditor.vue';
import TemplatePreview from '../components/TemplatePreview.vue';
import TemplateVisualSideEditor from '../components/TemplateVisualSideEditor.vue';
import { COMMAND_IDS } from '../commands/registry';
import { useCommandHandler } from '../composables/useCommands';
import { conceptLibraryErrorMessage } from '../composables/useConceptLibrary';
import { useTemplateLibrary } from '../composables/useTemplateLibrary';
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

const form = reactive({
  content: createDefaultTemplateContent(),
  name: ''
});
const initialLoading = ref( true );
const loadError = ref( '' );
const saveAttempted = ref( false );
const savedSnapshot = ref( '' );
let loadRequestSequence = 0;
let saveRequestSequence = 0;

const templateId = computed( () => route.params.templateId ?? '' );
const isEditing = computed( () => Boolean( templateId.value ) );
const pageTitle = computed( () => isEditing.value ? 'Edit template' : 'New template' );

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
  return JSON.stringify({ name: form.name, content: form.content }) !== savedSnapshot.value;
});
const saveCommand = useCommandHandler( COMMAND_IDS.templateSave, {
  enabled: computed( () => (
    !initialLoading.value
    && !loadError.value
    && !isPending.value
    && hasChanges.value
  ) ),
  execute: saveTemplate
});

watch( templateId, loadTemplate, { immediate: true });

onBeforeUnmount( () => {
  loadRequestSequence += 1;
  saveRequestSequence += 1;
});

async function loadTemplate() {
  const request = ++loadRequestSequence;
  const requestedTemplateId = templateId.value;

  saveRequestSequence += 1;
  clearError();
  initialLoading.value = true;
  loadError.value = '';
  saveAttempted.value = false;

  if ( !requestedTemplateId ) {
    applyTemplate({
      name: '',
      content: createDefaultTemplateContent()
    });
    savedSnapshot.value = '';
    initialLoading.value = false;
    return;
  }

  try {
    const template = await getTemplate( requestedTemplateId );

    if ( request !== loadRequestSequence ) {
      return;
    }

    applyTemplate( template );
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

function applyTemplate( template ) {
  form.name = template.name;
  form.content = cloneTemplateContent( template.content );
  savedSnapshot.value = JSON.stringify({
    name: form.name,
    content: form.content
  });
}

async function saveTemplate() {
  saveAttempted.value = true;
  clearError();

  if ( !formValid.value || isPending.value ) {
    return;
  }

  const input = {
    name: form.name,
    content: cloneTemplateContent( form.content )
  };
  const request = ++saveRequestSequence;

  try {
    const saved = isEditing.value
      ? await updateTemplate({ id: templateId.value, ...input })
      : await createTemplate( input );

    if ( request !== saveRequestSequence ) {
      return;
    }

    applyTemplate( saved );
    saveAttempted.value = false;

    if ( !isEditing.value ) {
      await router.replace({
        name: 'template-edit',
        params: { templateId: saved.id }
      });
    }
  } catch {
    // Error state is handled by the composable.
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

    <form
      v-else
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
        :disabled="isPending"
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
          :disabled="isPending"
          @click="cancel"
        >
          Cancel
        </UButton>

        <UButton
          type="submit"
          leading-icon="i-lucide-check"
          :disabled="!hasChanges"
          :loading="isPending"
          :aria-keyshortcuts="saveCommand.ariaKeyshortcuts"
          :title="saveCommand.tooltip"
          size="lg"
        >
          {{ isEditing ? 'Save template' : 'Create template' }}
        </UButton>
      </footer>
    </form>
  </div>
</template>
