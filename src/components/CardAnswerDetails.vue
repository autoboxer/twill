<script setup>
import { computed } from 'vue';
import StudyIdeaList from './StudyIdeaList.vue';

const props = defineProps({
  card: {
    type: Object,
    required: true
  },

  comparisonEnabled: {
    type: Boolean,
    default: false
  },

  checks: {
    type: Object,
    default: () => ({})
  }
});

const emit = defineEmits([ 'compare' ]);

const details = computed( () => {
  if ( props.card.typeAnswer ) {
    return { label: 'Accepted answers', items: props.card.typeAnswer.acceptedAnswers };
  }

  if ( props.card.explain ) {
    return { label: 'Key points', items: props.card.explain.keyPoints, kind: 'idea' };
  }

  if ( props.card.problem ) {
    return { label: 'Solution checkpoints', items: props.card.problem.checkpoints, kind: 'checkpoint' };
  }

  return null;
});
</script>

<template>
  <section v-if="details?.items.length" class="card-answer-details">
    <h4>{{ details.label }}</h4>

    <StudyIdeaList
      v-if="comparisonEnabled && details.kind"
      :items="details.items"
      :kind="details.kind"
      :checks="checks"
      @compare="( id, value ) => emit( 'compare', id, value )"
    />

    <ol v-else>
      <li v-for="( item, index ) in details.items" :key="index">
        {{ item }}
      </li>
    </ol>
  </section>
</template>

<style scoped>
.card-answer-details {
  display: grid;
  min-width: 0;
  gap: 0.5rem;
  overflow-wrap: anywhere;
}

.card-answer-details h4 {
  font-size: 0.9rem;
  font-weight: 600;
}

.card-answer-details > ol:not(.study-idea-list) {
  display: grid;
  gap: 0.35rem;
  padding-left: 1.5rem;
  list-style: decimal;
  white-space: pre-wrap;
}
</style>
