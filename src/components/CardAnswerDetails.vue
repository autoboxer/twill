<script setup>
import { computed } from 'vue';

const props = defineProps({
  card: {
    type: Object,
    required: true
  }
});

const details = computed( () => {
  if ( props.card.typeAnswer ) {
    return { label: 'Accepted answers', items: props.card.typeAnswer.acceptedAnswers };
  }

  if ( props.card.explain ) {
    return { label: 'Key points', items: props.card.explain.keyPoints };
  }

  if ( props.card.problem ) {
    return { label: 'Solution checkpoints', items: props.card.problem.checkpoints };
  }

  return null;
});
</script>

<template>
  <section v-if="details?.items.length" class="card-answer-details">
    <h4>{{ details.label }}</h4>

    <ol>
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

.card-answer-details ol {
  display: grid;
  gap: 0.35rem;
  padding-left: 1.5rem;
  list-style: decimal;
  white-space: pre-wrap;
}
</style>
