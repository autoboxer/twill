<script setup>
import { computed } from 'vue';

const props = defineProps({
  choice: { type: Object, required: true },
  revealed: { type: Boolean, default: false }
});
const state = defineModel( 'state', { type: Object, required: true });
const selectedId = computed({
  get: () => state.value.selectedId,
  set: ( value ) => {
    state.value = { ...state.value, selectedId: value };
  }
});
const options = computed( () => state.value.optionIds.map( ( id ) => (
  props.choice.options.find( ( option ) => option.id === id )
) ) );
const selected = computed( () => props.choice.options.find( ( option ) => option.id === state.value.submittedId ) );
const correct = computed( () => props.choice.options.find( ( option ) => option.id === props.choice.correctOptionId ) );
</script>

<template>
  <div class="practice-choice" data-twill-choice>
    <fieldset>
      <legend>Choose an answer</legend>
      <label v-for="option in options" :key="option.id" :data-twill-choice-option="option.id">
        <input
          v-model="selectedId"
          type="radio"
          name="practice-choice-answer"
          :value="option.id"
          :disabled="Boolean( state.submittedId )"
        >
        <span>{{ option.text }}</span>
      </label>
    </fieldset>

    <section
      v-if="selected && revealed"
      class="practice-choice__feedback"
      aria-live="polite"
      data-twill-choice-feedback
    >
      <h3>{{ selected.id === correct.id ? 'Correct' : 'Incorrect' }}</h3>
      <p>{{ selected.feedback }}</p>
      <template v-if="selected.id !== correct.id">
        <h4>Correct answer</h4>
        <p>{{ correct.text }}</p>
        <p>{{ correct.feedback }}</p>
      </template>
    </section>
  </div>
</template>

<style scoped>
.practice-choice,
.practice-choice fieldset {
  display: grid;
  gap: 0.5rem;
  min-width: 0;
}

.practice-choice legend {
  margin-bottom: 0.5rem;
  font-weight: 600;
}

.practice-choice label {
  display: flex;
  align-items: flex-start;
  gap: 0.65rem;
  padding: 0.6rem;
  border: 1px solid var(--ui-border);
  border-radius: 0.25rem;
  cursor: pointer;
}

.practice-choice label:has(input:checked) {
  background: var(--ui-bg-elevated);
}

.practice-choice label:has(input:focus-visible) {
  outline: 2px solid var(--ui-primary);
  outline-offset: 2px;
}

.practice-choice input {
  flex: none;
  margin-top: 0.25rem;
  accent-color: var(--ui-primary);
}

.practice-choice label span,
.practice-choice__feedback p {
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.practice-choice__feedback {
  display: grid;
  gap: 0.5rem;
  margin-top: 0.25rem;
}
</style>
