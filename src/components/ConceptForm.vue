<script setup>
import { computed, reactive, ref, watch } from 'vue';

import ClozePreview from './ClozePreview.vue';
import ImageOcclusionPreview from './ImageOcclusionPreview.vue';
import RichContentEditor from './RichContentEditor.vue';
import {
  collectClozeGroups,
  removeAllClozeMarks
} from '../cloze/documents';
import {
  collectImageOcclusionGroups,
  removeAllImageOcclusionRegions
} from '../image-occlusion/documents';
import {
  cloneConceptContent,
  createEmptyConceptContent
} from '../rich-content/schema';
import {
  cloneConceptEditorState,
  conceptRetrievalFormId,
  createConceptEditorState
} from '../drafts/conceptDraft';

const CLOZE_ID = 'cloze';
const IMAGE_OCCLUSION_ID = 'image-occlusion';
const STANDARD_RECALL_ID = 'standard-recall';
const TYPE_ANSWER_ID = 'type-answer';
const MAXIMUM_ACCEPTED_ANSWERS = 20;
const MAXIMUM_ACCEPTED_ANSWER_LENGTH = 500;

const props = defineProps({
  concept: {
    type: Object,
    default: null
  },
  decks: {
    type: Array,
    default: () => []
  },
  disabled: {
    type: Boolean,
    default: false
  },
  error: {
    type: String,
    default: ''
  },
  editorState: {
    type: Object,
    default: null
  },
  loading: {
    type: Boolean,
    default: false
  },
  mode: {
    type: String,
    default: 'create',
    validator: ( value ) => [ 'create', 'edit' ].includes( value )
  },
  saveCommand: {
    type: Object,
    required: true
  },
  tags: {
    type: Array,
    default: () => []
  },
  templates: {
    type: Array,
    default: () => []
  }
});

const emit = defineEmits([ 'cancel', 'change', 'manage', 'submit' ]);

const form = reactive({
  content: createEmptyConceptContent(),
  deckIds: [],
  retrievalFormIds: [ STANDARD_RECALL_ID ],
  tagIds: [],
  typeAnswerAcceptedAnswers: [ '' ],
  title: ''
});

const submitted = ref( false );

const deckItems = computed( () => props.decks.map( ( deck ) => ({
  label: deck.name,
  value: deck.id
}) ) );

const tagItems = computed( () => props.tags.map( ( tag ) => ({
  label: tag.name,
  value: tag.id
}) ) );

const retrievalFormItems = computed( () => [
  {
    description: 'Shows the prompt first and the answer after reveal.',
    label: 'Standard recall',
    value: STANDARD_RECALL_ID
  },
  {
    description: 'Requires a typed response before the answer is revealed.',
    label: 'Type answer',
    value: TYPE_ANSWER_ID
  },
  {
    description: 'Hides marked Prompt passages and reveals them in context.',
    label: 'Cloze',
    value: CLOZE_ID
  },
  {
    description: 'Hides selected regions of a Prompt image.',
    label: 'Image occlusion',
    value: IMAGE_OCCLUSION_ID
  },

  ...props.templates.map( ( template ) => ({
    description: template.mode === 'custom' ? 'HTML & CSS template' : 'Visual template',
    label: template.name,
    value: template.id
  }) )
]);

const titleError = computed( () => {
  if ( !submitted.value || form.title.trim() ) {
    return '';
  }

  return 'Concept title cannot be empty.';
});

const titleLength = computed( () => Array.from( form.title ).length );
const clozeGroups = computed( () => collectClozeGroups( form.content.prompt ) );
const clozeSelected = computed( () => form.retrievalFormIds.includes( CLOZE_ID ) );
const imageOcclusionGroups = computed( () => (
  collectImageOcclusionGroups( form.content.prompt )
) );
const imageOcclusionSelected = computed( () => (
  form.retrievalFormIds.includes( IMAGE_OCCLUSION_ID )
) );
const typeAnswerSelected = computed( () => form.retrievalFormIds.includes( TYPE_ANSWER_ID ) );
const atAcceptedAnswerLimit = computed( () => (
  form.typeAnswerAcceptedAnswers.length >= MAXIMUM_ACCEPTED_ANSWERS
) );
const acceptedAnswerErrors = computed( () => {
  if ( !submitted.value || !typeAnswerSelected.value ) {
    return form.typeAnswerAcceptedAnswers.map( () => '' );
  }

  const normalizedAnswers = form.typeAnswerAcceptedAnswers.map( normalizeAcceptedAnswer );

  return normalizedAnswers.map( ( answer, index ) => {
    if ( !answer ) {
      return 'Enter an accepted answer.';
    }

    if ( Array.from( answer ).length > MAXIMUM_ACCEPTED_ANSWER_LENGTH ) {
      return `Accepted answers cannot exceed ${ MAXIMUM_ACCEPTED_ANSWER_LENGTH } characters.`;
    }

    const comparisonAnswer = answer.toLowerCase();
    const duplicateIndex = normalizedAnswers.findIndex( ( candidate ) => (
      candidate.toLowerCase() === comparisonAnswer
    ) );

    if ( duplicateIndex !== index ) {
      return 'Accepted answers must be unique.';
    }

    return '';
  });
});
const acceptedAnswersValid = computed( () => (
  acceptedAnswerErrors.value.every( ( error ) => !error )
) );

const retrievalFormsError = computed( () => {
  if ( !submitted.value || form.retrievalFormIds.length ) {
    return '';
  }

  return 'Select at least one retrieval form.';
});

const clozeError = computed( () => {
  if ( !submitted.value || !clozeSelected.value || clozeGroups.value.length ) {
    return '';
  }

  return 'Mark at least one Prompt passage as an omission.';
});

const imageOcclusionError = computed( () => {
  if (
    !submitted.value
    || !imageOcclusionSelected.value
    || imageOcclusionGroups.value.length
  ) {
    return '';
  }

  return 'Add at least one mask to a Prompt image.';
});

const removedRetrievalForms = computed( () => {
  if ( props.mode !== 'edit' || !props.concept ) {
    return [];
  }

  return props.concept.cards.filter( ( card ) => {
    const id = conceptRetrievalFormId( card );

    return !form.retrievalFormIds.includes( id );
  });
});

const submitLabel = computed( () => props.mode === 'edit' ? 'Save changes' : 'Create concept' );

watch([ () => props.concept, () => props.editorState ], ([ concept, editorState ]) => {
  const state = editorState
    ? cloneConceptEditorState( editorState )
    : createConceptEditorState( concept );

  form.content = state.content;
  form.title = state.title;
  form.deckIds = state.deckIds;
  form.retrievalFormIds = state.retrievalFormIds;
  form.tagIds = state.tagIds;
  form.typeAnswerAcceptedAnswers = state.typeAnswerAcceptedAnswers;
  submitted.value = false;
}, { immediate: true });

watch( form, () => {
  emit( 'change', cloneConceptEditorState( form ) );
}, { deep: true });

function updateRetrievalForms( retrievalFormIds ) {
  if (
    clozeSelected.value
    && !retrievalFormIds.includes( CLOZE_ID )
  ) {
    form.content.prompt = removeAllClozeMarks( form.content.prompt );
  }

  if (
    imageOcclusionSelected.value
    && !retrievalFormIds.includes( IMAGE_OCCLUSION_ID )
  ) {
    form.content.prompt = removeAllImageOcclusionRegions( form.content.prompt );
  }

  form.retrievalFormIds = retrievalFormIds;
}

function normalizeAcceptedAnswer( answer ) {
  return answer.trim().replace( /\s+/gu, ' ' );
}

function acceptedAnswerLength( answer ) {
  return Array.from( normalizeAcceptedAnswer( answer ) ).length;
}

function addAcceptedAnswer() {
  if ( atAcceptedAnswerLimit.value ) {
    return;
  }

  form.typeAnswerAcceptedAnswers.push( '' );
}

function removeAcceptedAnswer( index ) {
  if ( form.typeAnswerAcceptedAnswers.length === 1 ) {
    return;
  }

  form.typeAnswerAcceptedAnswers.splice( index, 1 );
}

function submit() {
  if ( props.disabled ) {
    return;
  }

  submitted.value = true;

  if (
    !form.title.trim()
    || !form.retrievalFormIds.length
    || Boolean( clozeError.value )
    || Boolean( imageOcclusionError.value )
    || !acceptedAnswersValid.value
  ) {
    return;
  }

  const typeAnswer = typeAnswerSelected.value
    ? {
      acceptedAnswers: form.typeAnswerAcceptedAnswers.map( normalizeAcceptedAnswer )
    }
    : null;

  emit( 'submit', {
    content: cloneConceptContent( form.content ),
    deckIds: [ ...form.deckIds ],
    includeStandardRecall: form.retrievalFormIds.includes( STANDARD_RECALL_ID ),
    tagIds: [ ...form.tagIds ],
    templateIds: form.retrievalFormIds.filter( ( id ) => (
      id !== CLOZE_ID
      && id !== IMAGE_OCCLUSION_ID
      && id !== STANDARD_RECALL_ID
      && id !== TYPE_ANSWER_ID
    ) ),
    typeAnswer,
    title: form.title
  });
}

defineExpose({ submit });
</script>

<template>
  <form
    class="concept-editor"
    @submit.prevent="submit"
  >
    <UAlert
      v-if="error"
      :description="error"
      icon="i-lucide-circle-alert"
      color="error"
      variant="soft"
    />

    <section
      class="editor-section"
      data-twill-editor-section="concept-basics"
    >
      <div class="editor-section__heading">
        <div>
          <h2>Concept</h2>
          <p>Use a short, recognizable title.</p>
        </div>
      </div>

      <UFormField
        label="Title"
        :error="titleError || false"
        :hint="`${ titleLength } / 200`"
        required
      >
        <UInput
          v-model="form.title"
          placeholder="Concept title"
          :maxlength="200"
          autocomplete="off"
          autofocus
          class="w-full"
          size="xl"
          :disabled="disabled"
        />
      </UFormField>
    </section>

    <section
      class="editor-section"
      data-twill-editor-section="concept-content"
    >
      <div class="editor-section__heading">
        <div>
          <h2>Content</h2>
          <p>Write the prompt and answer for this concept.</p>
        </div>
      </div>

      <div class="concept-content-editors">
        <RichContentEditor
          v-model="form.content.prompt"
          label="Prompt"
          placeholder="Write a prompt"
          :cloze-enabled="clozeSelected"
          :disabled="disabled"
          :image-occlusion-enabled="imageOcclusionSelected"
        />

        <RichContentEditor
          v-model="form.content.answer"
          label="Answer"
          placeholder="Write an answer"
          :disabled="disabled"
        />

        <ClozePreview
          v-if="clozeSelected"
          :document="form.content.prompt"
        />

        <ImageOcclusionPreview
          v-if="imageOcclusionSelected"
          :document="form.content.prompt"
        />
      </div>
    </section>

    <section
      class="editor-section"
      data-twill-editor-section="concept-retrieval-forms"
    >
      <div class="editor-section__heading">
        <div>
          <h2>Retrieval forms</h2>
          <p>Choose how this concept is studied. Each form keeps its own schedule.</p>
        </div>
      </div>

      <UCheckboxGroup
        :model-value="form.retrievalFormIds"
        :items="retrievalFormItems"
        legend="Retrieval forms"
        value-key="value"
        variant="card"
        class="retrieval-form-options"
        :ui="{ legend: 'sr-only' }"
        :disabled="disabled"
        required
        @update:model-value="updateRetrievalForms"
      />

      <p
        v-if="retrievalFormsError"
        class="editor-field-error"
      >
        {{ retrievalFormsError }}
      </p>

      <p
        v-if="clozeError"
        class="editor-field-error"
      >
        {{ clozeError }}
      </p>

      <p
        v-if="imageOcclusionError"
        class="editor-field-error"
      >
        {{ imageOcclusionError }}
      </p>

      <div
        v-if="typeAnswerSelected"
        class="type-answer-settings"
      >
        <div class="type-answer-settings__heading">
          <div>
            <h3>Accepted answers</h3>
            <p>Add alternatives that should count as an exact match.</p>
          </div>

          <span>
            {{ form.typeAnswerAcceptedAnswers.length }} / {{ MAXIMUM_ACCEPTED_ANSWERS }}
          </span>
        </div>

        <div class="accepted-answer-list">
          <UFormField
            v-for="( answer, index ) in form.typeAnswerAcceptedAnswers"
            :key="index"
            :label="index === 0 ? 'Answer' : `Alternative ${ index }`"
            :error="acceptedAnswerErrors[ index ] || false"
            :hint="`${ acceptedAnswerLength( answer ) } / ${ MAXIMUM_ACCEPTED_ANSWER_LENGTH }`"
            required
          >
            <div class="accepted-answer-row">
              <UInput
                v-model="form.typeAnswerAcceptedAnswers[ index ]"
                :placeholder="index === 0 ? 'Accepted answer' : 'Alternative answer'"
                autocomplete="off"
                class="accepted-answer-row__input"
                size="lg"
                :disabled="disabled"
              />

              <UButton
                type="button"
                icon="i-lucide-x"
                :aria-label="`Remove ${ index === 0 ? 'answer' : `alternative ${ index }` }`"
                color="neutral"
                variant="ghost"
                size="lg"
                square
                :disabled="disabled || form.typeAnswerAcceptedAnswers.length === 1"
                @click="removeAcceptedAnswer( index )"
              />
            </div>
          </UFormField>
        </div>

        <UButton
          type="button"
          label="Add alternative"
          leading-icon="i-lucide-plus"
          color="neutral"
          variant="subtle"
          :disabled="disabled || atAcceptedAnswerLimit"
          class="type-answer-settings__add"
          @click="addAcceptedAnswer"
        />
      </div>

      <UAlert
        v-if="removedRetrievalForms.length"
        title="Scheduling progress will be removed"
        description="Saving will remove the deselected forms and their scheduling progress. Adding a form again starts it as new."
        icon="i-lucide-calendar-x-2"
        color="warning"
        variant="soft"
      />
    </section>

    <section
      class="editor-section"
      data-twill-editor-section="concept-organization"
    >
      <div class="editor-section__heading">
        <div>
          <h2>Organization</h2>
          <p>Use decks for broader groups and tags for labels that can cross decks.</p>
        </div>

        <UButton
          type="button"
          leading-icon="i-lucide-settings-2"
          color="neutral"
          variant="subtle"
          :disabled="disabled"
          @click="emit( 'manage' )"
        >
          Manage
        </UButton>
      </div>

      <div class="editor-selection-grid">
        <div class="editor-selection">
          <h3>Decks</h3>

          <UCheckboxGroup
            v-if="deckItems.length"
            v-model="form.deckIds"
            :items="deckItems"
            value-key="value"
            variant="card"
            class="editor-checkboxes"
            :disabled="disabled"
          />

          <p
            v-else
            class="editor-selection__empty"
          >
            No decks yet.
          </p>
        </div>

        <div class="editor-selection">
          <h3>Tags</h3>

          <UCheckboxGroup
            v-if="tagItems.length"
            v-model="form.tagIds"
            :items="tagItems"
            value-key="value"
            variant="card"
            class="editor-checkboxes"
            :disabled="disabled"
          />

          <p
            v-else
            class="editor-selection__empty"
          >
            No tags yet.
          </p>
        </div>
      </div>
    </section>

    <footer class="editor-actions">
      <UButton
        type="button"
        color="neutral"
        variant="link"
        :disabled="disabled"
        @click="emit( 'cancel' )"
      >
        Cancel
      </UButton>

      <UButton
        type="submit"
        leading-icon="i-lucide-check"
        :disabled="disabled"
        :loading="loading"
        :aria-keyshortcuts="saveCommand.ariaKeyshortcuts"
        :title="saveCommand.tooltip"
        size="lg"
      >
        {{ submitLabel }}
      </UButton>
    </footer>
  </form>
</template>
