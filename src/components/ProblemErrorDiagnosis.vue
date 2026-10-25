<script setup>
defineProps({
  diagnosis: { type: Object, required: true },
  responses: { type: Object, default: null },
  revealed: { type: Boolean, default: false },
  disabled: { type: Boolean, default: false },
  checked: { type: Boolean, default: false }
});

const emit = defineEmits([ 'response', 'toggle' ]);
const questions = [
  { id: 'location', label: 'Where is the error?' },
  { id: 'explanation', label: 'Why is it wrong?' },
  { id: 'correction', label: 'Correct the work' }
];
const answers = [
  { id: 'location', label: 'Error location' },
  { id: 'explanation', label: 'Why it is wrong' },
  { id: 'correction', label: 'Corrected work' }
];
</script>

<template>
  <section class="problem-error-diagnosis" data-twill-error-diagnosis aria-label="Find the error">
    <div class="problem-error-diagnosis__example">
      <h4>Example with an error</h4>
      <p>{{ diagnosis.attempt }}</p>
    </div>

    <template v-if="responses">
      <UFormField v-for="question in questions" :key="question.id" :label="question.label">
        <UTextarea
          :model-value="responses[ question.id ] ?? ''"
          :aria-label="question.label"
          :rows="2"
          autoresize
          :maxrows="8"
          :readonly="checked"
          :disabled="disabled"
          class="problem-error-diagnosis__input"
          @update:model-value="emit( 'response', question.id, $event )"
        />
      </UFormField>

      <UButton
        v-if="!checked"
        type="button"
        color="neutral"
        variant="link"
        size="sm"
        :aria-expanded="revealed"
        :disabled="disabled"
        @click="emit( 'toggle' )"
      >
        {{ revealed ? 'Hide correction' : 'Show correction' }}
      </UButton>
    </template>

    <div v-if="revealed || checked" class="problem-error-diagnosis__correction" aria-label="Authored correction">
      <div v-for="answer in answers" :key="answer.id">
        <h4>{{ answer.label }}</h4>
        <p>{{ diagnosis[ answer.id ] }}</p>
      </div>
    </div>
  </section>
</template>

<style scoped>
.problem-error-diagnosis,
.problem-error-diagnosis__correction {
  display: grid;
  min-width: 0;
  gap: 0.75rem;
}

.problem-error-diagnosis__example,
.problem-error-diagnosis__correction {
  padding-left: 0.75rem;
  border-left: 2px solid var(--ui-border);
}

.problem-error-diagnosis__example {
  border-color: var(--ui-border-accented);
}

.problem-error-diagnosis h4,
.problem-error-diagnosis p {
  margin: 0;
  font-size: 0.9rem;
  overflow-wrap: anywhere;
  white-space: pre-wrap;
}

.problem-error-diagnosis h4 {
  margin-bottom: 0.3rem;
  font-weight: 600;
}

.problem-error-diagnosis__input {
  width: 100%;
}

.problem-error-diagnosis > button {
  justify-self: start;
}
</style>
