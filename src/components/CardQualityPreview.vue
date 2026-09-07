<script setup>
import { computed, onBeforeUnmount, ref, watch } from 'vue';

import { cardQualityFormName } from '../card-quality/presentation';
import {
  conceptLibraryErrorMessage,
  useConceptLibrary
} from '../composables/useConceptLibrary';
import { useTemplateLibrary } from '../composables/useTemplateLibrary';
import { richDocumentHasContent } from '../rich-content/schema';
import ContentState from './ContentState.vue';
import RichContentRenderer from './RichContentRenderer.vue';
import StudyCardContent from './StudyCardContent.vue';

const props = defineProps({
  item: {
    type: Object,
    required: true
  }
});

const { getConcept } = useConceptLibrary();
const { getTemplate } = useTemplateLibrary();
const card = ref( null );
const media = ref([]);
const loading = ref( true );
const error = ref( '' );
const revealed = ref( false );
let requestSequence = 0;

const formName = computed( () => card.value
  ? cardQualityFormName( card.value, card.value.content.prompt )
  : ''
);
const answerDetails = computed( () => {
  if ( card.value?.typeAnswer ) {
    return { label: 'Accepted answers', items: card.value.typeAnswer.acceptedAnswers };
  }

  if ( card.value?.explain ) {
    return { label: 'Key points', items: card.value.explain.keyPoints };
  }

  if ( card.value?.problem ) {
    return { label: 'Solution checkpoints', items: card.value.problem.checkpoints };
  }

  return null;
});
const feedbackDocuments = computed( () => [
  { label: 'Explanation and context', document: card.value?.content.feedback?.explanation },
  { label: 'Common mistakes', document: card.value?.content.feedback?.commonMistakes }
].filter( ( item ) => richDocumentHasContent( item.document ) ) );

watch( () => props.item, loadPreview, { immediate: true });

onBeforeUnmount( () => {
  requestSequence += 1;
});

async function loadPreview() {
  const request = ++requestSequence;
  const { conceptId, cardId } = props.item;

  loading.value = true;
  error.value = '';
  card.value = null;
  revealed.value = false;

  try {
    const concept = await getConcept( conceptId );

    if ( request !== requestSequence ) {
      return;
    }

    const form = concept.cards.find( ( entry ) => entry.id === cardId );

    if ( concept.archived || !form ) {
      error.value = 'This card is no longer active. Refresh the queue to update its items.';
      return;
    }

    const template = form.template ? await getTemplate( form.template.id ) : null;

    if ( request === requestSequence ) {
      card.value = {
        ...form,
        conceptTitle: concept.title,
        content: concept.content,
        template
      };
      media.value = concept.media;
    }
  } catch ( cause ) {
    if ( request === requestSequence ) {
      error.value = conceptLibraryErrorMessage( cause );
    }
  } finally {
    if ( request === requestSequence ) {
      loading.value = false;
    }
  }
}
</script>

<template>
  <div class="quality-preview">
    <ContentState
      v-if="loading"
      kind="loading"
      title="Loading card"
    />

    <ContentState
      v-else-if="error"
      kind="error"
      title="Card could not be loaded"
      :description="error"
    >
      <template #actions>
        <UButton variant="subtle" @click="loadPreview">Retry preview</UButton>
      </template>
    </ContentState>

    <template v-else-if="card">
      <div class="quality-preview__heading">
        <div>
          <h3>{{ formName }}</h3>
          <p>Current card · {{ revealed ? 'Answer' : 'Front' }}</p>
        </div>

        <UButton
          :aria-pressed="revealed"
          color="neutral"
          variant="subtle"
          @click="revealed = !revealed"
        >
          {{ revealed ? 'Show front' : 'Show answer' }}
        </UButton>
      </div>

      <StudyCardContent :card="card" :media="media" :answer-revealed="revealed" />

      <template v-if="revealed">
        <section v-if="answerDetails" class="quality-preview__details">
          <h4>{{ answerDetails.label }}</h4>

          <ol>
            <li v-for="( detail, index ) in answerDetails.items" :key="index">
              {{ detail }}
            </li>
          </ol>
        </section>

        <section
          v-for="document in feedbackDocuments"
          :key="document.label"
          class="quality-preview__details"
        >
          <h4>{{ document.label }}</h4>
          <RichContentRenderer :document="document.document" :label="document.label" />
        </section>
      </template>
    </template>
  </div>
</template>

<style scoped>
.quality-preview {
  display: grid;
  gap: 1.25rem;
  min-width: 0;
  padding: 1.25rem;
  border-top: 1px solid var(--ui-border);
  background: var(--ui-bg);
  overflow-wrap: anywhere;
}

.quality-preview__heading {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
}

.quality-preview h3,
.quality-preview h4 {
  font-size: 0.9rem;
  font-weight: 600;
}

.quality-preview__heading p {
  margin-top: 0.25rem;
  color: var(--ui-text-muted);
  font-size: 0.8rem;
}

.quality-preview__details {
  display: grid;
  gap: 0.5rem;
}

.quality-preview__details ol {
  display: grid;
  gap: 0.35rem;
  padding-left: 1.5rem;
  list-style: decimal;
  white-space: pre-wrap;
}
</style>
