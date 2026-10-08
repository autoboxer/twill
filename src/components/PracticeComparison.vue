<script setup>
import { ref } from 'vue';

import RichContentRenderer from './RichContentRenderer.vue';

defineProps({
  activity: { type: Object, required: true }
});

const response = ref( '' );
const revealed = ref( false );
</script>

<template>
  <div class="practice-comparison" data-twill-comparison>
    <p class="practice-comparison__prompt">{{ activity.link.comparison.prompt }}</p>

    <div class="practice-comparison__cases">
      <section v-for="concept in [activity.source, activity.related]" :key="concept.id">
        <h3>{{ concept.title }}</h3>
        <RichContentRenderer :document="concept.content.prompt" :label="`${ concept.title } prompt`" />
        <div v-if="revealed" class="practice-comparison__answer">
          <h4>Answer</h4>
          <RichContentRenderer :document="concept.content.answer" :label="`${ concept.title } answer`" />
        </div>
      </section>
    </div>

    <div>
      <label for="comparison-response">Your explanation (optional)</label>
      <UTextarea
        id="comparison-response"
        v-model="response"
        :rows="3"
        class="w-full"
      />
    </div>

    <UButton
      color="neutral"
      variant="subtle"
      :aria-expanded="revealed"
      aria-controls="comparison-expected-explanation"
      @click="revealed = !revealed"
    >
      {{ revealed ? 'Hide explanation' : 'Show explanation' }}
    </UButton>

    <section v-if="revealed" id="comparison-expected-explanation" class="practice-comparison__explanation">
      <h3>Expected explanation</h3>
      <p>{{ activity.link.comparison.explanation }}</p>
    </section>
  </div>
</template>

<style scoped>
.practice-comparison {
  display: grid;
  gap: 1rem;
  min-width: 0;
}

.practice-comparison__prompt,
.practice-comparison__explanation p {
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.practice-comparison__prompt {
  font-weight: 600;
}

.practice-comparison__cases {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 1rem;
}

.practice-comparison__cases > section {
  min-width: 0;
  padding-inline-start: 0.75rem;
  border-inline-start: 1px solid var(--ui-border);
}

.practice-comparison h3,
.practice-comparison h4 {
  font-weight: 600;
  overflow-wrap: anywhere;
}

.practice-comparison__answer {
  margin-top: 0.75rem;
}

.practice-comparison__explanation {
  padding-top: 0.75rem;
  border-top: 1px solid var(--ui-border);
}

.practice-comparison > button {
  justify-self: start;
}

.practice-comparison label {
  display: block;
  margin-bottom: 0.35rem;
}

@media (max-width: 600px) {
  .practice-comparison__cases {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
