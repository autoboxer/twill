<script setup>
import { computed, reactive, ref, watch } from 'vue';

import RichContentEditor from './RichContentEditor.vue';
import {
  cloneConceptContent,
  createEmptyConceptContent
} from '../rich-content/schema';

const STANDARD_RECALL_ID = 'standard-recall';

const props = defineProps({
  concept: {
    type: Object,
    default: null
  },
  decks: {
    type: Array,
    default: () => []
  },
  error: {
    type: String,
    default: ''
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
  tags: {
    type: Array,
    default: () => []
  },
  templates: {
    type: Array,
    default: () => []
  }
});

const emit = defineEmits([ 'cancel', 'manage', 'submit' ]);

const form = reactive({
  content: createEmptyConceptContent(),
  deckIds: [],
  retrievalFormIds: [ STANDARD_RECALL_ID ],
  tagIds: [],
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

const retrievalFormsError = computed( () => {
  if ( !submitted.value || form.retrievalFormIds.length ) {
    return '';
  }

  return 'Select at least one retrieval form.';
});

const removedRetrievalForms = computed( () => {
  if ( !props.concept ) {
    return [];
  }

  return props.concept.cards.filter( ( card ) => {
    const id = card.template?.id ?? STANDARD_RECALL_ID;

    return !form.retrievalFormIds.includes( id );
  });
});

const submitLabel = computed( () => props.mode === 'edit' ? 'Save changes' : 'Create concept' );

watch( () => props.concept, ( concept ) => {
  form.content = cloneConceptContent( concept?.content );
  form.title = concept?.title ?? '';
  form.deckIds = concept?.decks.map( ( deck ) => deck.id ) ?? [];
  form.retrievalFormIds = concept
    ? concept.cards.map( ( card ) => card.template?.id ?? STANDARD_RECALL_ID )
    : [ STANDARD_RECALL_ID ];
  form.tagIds = concept?.tags.map( ( tag ) => tag.id ) ?? [];
  submitted.value = false;
}, { immediate: true });

function submit() {
  submitted.value = true;

  if ( !form.title.trim() || !form.retrievalFormIds.length ) {
    return;
  }

  emit( 'submit', {
    content: cloneConceptContent( form.content ),
    deckIds: [ ...form.deckIds ],
    includeStandardRecall: form.retrievalFormIds.includes( STANDARD_RECALL_ID ),
    tagIds: [ ...form.tagIds ],
    templateIds: form.retrievalFormIds.filter( ( id ) => id !== STANDARD_RECALL_ID ),
    title: form.title
  });
}
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

    <section class="editor-section">
      <div class="editor-section__heading">
        <div>
          <h2>Concept</h2>
          <p>Use a short, recognizable title.</p>
        </div>
      </div>

      <UFormField
        label="Title"
        :error="titleError"
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
        />
      </UFormField>
    </section>

    <section class="editor-section">
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
        />

        <RichContentEditor
          v-model="form.content.answer"
          label="Answer"
          placeholder="Write an answer"
        />
      </div>
    </section>

    <section class="editor-section">
      <div class="editor-section__heading">
        <div>
          <h2>Retrieval forms</h2>
          <p>Choose how this concept is studied. Each form keeps its own schedule.</p>
        </div>
      </div>

      <UCheckboxGroup
        v-model="form.retrievalFormIds"
        :items="retrievalFormItems"
        legend="Retrieval forms"
        value-key="value"
        variant="card"
        class="retrieval-form-options"
        :ui="{ legend: 'sr-only' }"
        required
      />

      <p
        v-if="retrievalFormsError"
        class="editor-field-error"
      >
        {{ retrievalFormsError }}
      </p>

      <UAlert
        v-if="removedRetrievalForms.length"
        title="Scheduling progress will be removed"
        description="Saving will remove the deselected forms and their scheduling progress. Adding a form again starts it as new."
        icon="i-lucide-calendar-x-2"
        color="warning"
        variant="soft"
      />
    </section>

    <section class="editor-section">
      <div class="editor-section__heading">
        <div>
          <h2>Organization</h2>
          <p>Use decks for broader groups and tags for labels that can cross decks.</p>
        </div>

        <UButton
          type="button"
          leading-icon="i-lucide-settings-2"
          color="neutral"
          variant="soft"
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
        variant="ghost"
        :disabled="loading"
        @click="emit( 'cancel' )"
      >
        Cancel
      </UButton>

      <UButton
        type="submit"
        leading-icon="i-lucide-check"
        :loading="loading"
        size="lg"
      >
        {{ submitLabel }}
      </UButton>
    </footer>
  </form>
</template>
