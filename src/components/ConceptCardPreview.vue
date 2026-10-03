<script setup>
import { computed, ref, shallowRef, watch } from 'vue';

import { useTemplateLibrary } from '../composables/useTemplateLibrary';
import { conceptLibraryErrorMessage } from '../composables/useConceptLibrary';
import StudyCardContent from './StudyCardContent.vue';

const props = defineProps({
  content: { type: Object, required: true },
  title: { type: String, required: true },
  typeId: { type: String, required: true },
  media: { type: Array, required: true }
});

const builtInKinds = {
  'standard-recall': 'recall',
  'type-answer': 'typeAnswer',
  explain: 'explain',
  problem: 'problem'
};
const { getTemplate } = useTemplateLibrary();
const template = ref( null );
const error = ref( '' );
const loading = ref( false );
const revealed = ref( false );
const retry = ref( 0 );
const previewContent = shallowRef({ ...props.content });
const previewTitle = ref( props.title );

watch([ () => props.content.prompt, () => props.content.answer, () => props.title ], ( values, previous, onCleanup ) => {
  const timer = setTimeout( () => {
    previewContent.value = { ...props.content };
    previewTitle.value = props.title;
  }, 180 );

  onCleanup( () => clearTimeout( timer ) );
});

const card = computed( () => ({
  id: props.typeId,
  conceptTitle: previewTitle.value,
  content: previewContent.value,
  retrievalKind: builtInKinds[ props.typeId ] ?? 'recall',
  template: template.value
}) );

watch([ () => props.typeId, retry ], async ([ id ], previous, onCleanup ) => {
  let current = true;

  onCleanup( () => {
    current = false;
  });
  template.value = null;
  error.value = '';
  loading.value = !builtInKinds[ id ];
  revealed.value = false;

  if ( !loading.value ) {
    return;
  }

  try {
    const result = await getTemplate( id );

    if ( current ) {
      template.value = result;
    }
  } catch ( cause ) {
    if ( current ) {
      error.value = conceptLibraryErrorMessage( cause );
    }
  } finally {
    if ( current ) {
      loading.value = false;
    }
  }
}, { immediate: true });
</script>

<template>
  <section class="concept-card-preview" aria-label="Card preview">
    <div class="editor-section__heading">
      <h3>Preview</h3>
      <UButton
        type="button"
        color="neutral"
        variant="link"
        size="sm"
        :aria-pressed="revealed"
        @click="revealed = !revealed"
      >
        {{ revealed ? 'Show front' : 'Show answer' }}
      </UButton>
    </div>

    <p v-if="loading" role="status">Loading preview</p>
    <template v-else-if="error">
      <p role="alert">{{ error }}</p>
      <UButton type="button" variant="subtle" @click="retry += 1">Retry preview</UButton>
    </template>
    <template v-else>
      <StudyCardContent :card="card" :media="media" :answer-revealed="revealed" />
      <UInput
        v-if="typeId === 'type-answer' && !revealed"
        placeholder="Type your answer"
        aria-label="Study response preview"
        readonly
        tabindex="-1"
      />
    </template>
  </section>
</template>
