<script setup>
import { computed, ref, shallowRef, watch } from 'vue';

import { useTemplateLibrary } from '../composables/useTemplateLibrary';
import { useStudyComparison } from '../composables/useStudyComparison';
import { conceptLibraryErrorMessage } from '../composables/useConceptLibrary';
import CardAnswerDetails from './CardAnswerDetails.vue';
import StudyCardContent from './StudyCardContent.vue';
import StudyAssistance from './StudyAssistance.vue';
import StudyAnswerParts from './StudyAnswerParts.vue';
import { supportsAnswerParts } from '../answer-parts/documents';
import { usesAnswerPartComparison } from '../study/comparison';

const props = defineProps({
  content: { type: Object, required: true },
  title: { type: String, required: true },
  typeId: { type: String, required: true },
  typeSettings: { type: Object, default: null },
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
const revealedAssistance = ref([]);
const revealedAnswerParts = ref([]);
const retry = ref( 0 );
const previewContent = shallowRef({ ...props.content });
const previewTitle = ref( props.title );

watch([
  () => props.content.prompt,
  () => props.content.answer,
  () => props.content.assistance?.hint,
  () => props.content.assistance?.reference,
  () => props.title
], ( values, previous, onCleanup ) => {
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
  typeAnswer: props.typeId === 'type-answer' ? props.typeSettings : null,
  explain: props.typeId === 'explain' ? props.typeSettings : null,
  problem: props.typeId === 'problem' ? props.typeSettings : null,
  template: template.value
}) );
const selectiveAnswer = computed( () => supportsAnswerParts( card.value ) );
const { comparisonChecks, restoreComparison, setComparison } = useStudyComparison({
  currentCard: card,
  answerRevealed: revealed,
  actionsBlocked: loading
});

watch([ previewContent, () => props.typeSettings ], () => restoreComparison(), { deep: true });

watch([ () => props.typeId, retry ], async ([ id ], previous, onCleanup ) => {
  let current = true;

  onCleanup( () => {
    current = false;
  });
  template.value = null;
  error.value = '';
  loading.value = !builtInKinds[ id ];
  revealed.value = false;
  revealedAssistance.value = [];
  revealedAnswerParts.value = [];
  restoreComparison();

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

function toggleAssistance( id ) {
  revealedAssistance.value = revealedAssistance.value.includes( id )
    ? revealedAssistance.value.filter( ( value ) => value !== id )
    : [ ...revealedAssistance.value, id ];
}

function toggleAnswerParts( ids ) {
  revealedAnswerParts.value = ids.every( ( id ) => revealedAnswerParts.value.includes( id ) )
    ? revealedAnswerParts.value.filter( ( id ) => !ids.includes( id ) )
    : [ ...new Set([ ...revealedAnswerParts.value, ...ids ]) ];
}
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
      <StudyCardContent
        :card="card"
        :media="media"
        :answer-revealed="revealed"
        :hide-answer="selectiveAnswer"
      />
      <StudyAnswerParts
        v-if="selectiveAnswer"
        :document="previewContent.answer"
        :full-answer-revealed="revealed"
        :revealed="revealedAnswerParts"
        :comparison-enabled="usesAnswerPartComparison( card )"
        :written-response="card.retrievalKind === 'typeAnswer'"
        :checks="comparisonChecks"
        @toggle="toggleAnswerParts"
        @compare="setComparison"
      />
      <CardAnswerDetails
        v-if="revealed"
        :card="card"
        comparison-enabled
        :checks="comparisonChecks"
        @compare="setComparison"
      />
      <StudyAssistance
        :content="previewContent"
        :revealed="revealedAssistance"
        @toggle="toggleAssistance"
      />
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
