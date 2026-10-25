<script setup>
import { ref, watch } from 'vue';
import ProblemErrorDiagnosis from './ProblemErrorDiagnosis.vue';

const props = defineProps({
  example: { type: Object, required: true }
});

const mode = ref( 'full' );

watch( () => props.example, () => {
  mode.value = 'full';
});
</script>

<template>
  <section class="worked-example" data-twill-worked-example aria-label="Worked example">
    <div class="worked-example__modes" role="group" aria-label="Example preview">
      <UButton
        type="button"
        color="neutral"
        :variant="mode === 'full' ? 'subtle' : 'ghost'"
        size="sm"
        :aria-pressed="mode === 'full'"
        @click="mode = 'full'"
      >
        Full example
      </UButton>
      <UButton
        type="button"
        color="neutral"
        :variant="mode === 'completion' ? 'subtle' : 'ghost'"
        size="sm"
        :aria-pressed="mode === 'completion'"
        @click="mode = 'completion'"
      >
        Completion preview
      </UButton>
      <UButton
        v-if="example.errorDiagnosis"
        type="button"
        color="neutral"
        :variant="mode === 'diagnosis' ? 'subtle' : 'ghost'"
        size="sm"
        :aria-pressed="mode === 'diagnosis'"
        @click="mode = 'diagnosis'"
      >
        Find the error
      </UButton>
    </div>

    <ProblemErrorDiagnosis
      v-if="mode === 'diagnosis' && example.errorDiagnosis"
      :diagnosis="example.errorDiagnosis"
      revealed
    />
    <ol v-else class="worked-example__steps">
      <li v-for="( step, index ) in example.steps" :key="step.id" :data-step-id="step.id">
        <h4>{{ index + 1 }}. {{ step.prompt }}</h4>
        <p v-if="mode === 'full' || step.showInCompletion" class="worked-example__solution">{{ step.solution }}</p>
        <p v-else class="worked-example__hidden">This solution is hidden in completion practice.</p>
        <p v-if="step.principlePrompt" class="worked-example__principle">{{ step.principlePrompt }}</p>
      </li>
    </ol>
  </section>
</template>

<style scoped>
.worked-example,
.worked-example__steps {
  display: grid;
  min-width: 0;
  gap: 0.75rem;
}

.worked-example__modes {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
}

.worked-example__steps {
  padding: 0;
  margin: 0;
  list-style: none;
}

.worked-example__steps > li {
  min-width: 0;
  padding-left: 0.75rem;
  border-left: 2px solid var(--ui-border);
}

.worked-example h4,
.worked-example p {
  margin: 0;
  overflow-wrap: anywhere;
  white-space: pre-wrap;
}

.worked-example h4 {
  font-size: 0.88rem;
  font-weight: 600;
}

.worked-example p {
  margin-top: 0.35rem;
  font-size: 0.86rem;
}

.worked-example__hidden,
.worked-example__principle {
  color: var(--ui-text-muted);
}
</style>
