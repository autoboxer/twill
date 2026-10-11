<script setup>
import { computed, ref, watch } from 'vue';

import { choiceError, emptyChoice, emptyChoiceOption } from '../practice/choices';

defineProps({ pending: { type: Boolean, default: false } });
const choice = defineModel({ type: Object, default: null });
const draft = ref( emptyChoice() );
const enabled = computed({
  get: () => Boolean( choice.value ),
  set: ( value ) => {
    choice.value = value ? draft.value : null;
  }
});
const keepOrder = computed({
  get: () => !choice.value?.shuffle,
  set: ( value ) => {
    choice.value.shuffle = !value;
  }
});
const error = computed( () => choiceError( choice.value ) );

watch( choice, ( value ) => {
  if ( value ) {
    draft.value = value;
  }
}, { immediate: true });

function removeOption( id ) {
  choice.value.options = choice.value.options.filter( ( option ) => option.id !== id );

  if ( choice.value.correctOptionId === id ) {
    choice.value.correctOptionId = '';
  }
}
</script>

<template>
  <div class="practice-choice-editor">
    <UCheckbox v-model="enabled" label="Add answer choices" :disabled="pending" />

    <template v-if="choice">
      <fieldset v-for="(option, index) in choice.options" :key="option.id">
        <legend>Option {{ index + 1 }}</legend>
        <div class="practice-choice-editor__heading">
          <label>
            <input
              v-model="choice.correctOptionId"
              type="radio"
              name="choice-correct-answer"
              :value="option.id"
              :aria-label="`Mark option ${ index + 1 } as correct`"
              :disabled="pending"
            >
            Correct answer
          </label>
          <UButton
            icon="i-lucide-x"
            :aria-label="`Remove option ${ index + 1 }`"
            color="neutral"
            variant="ghost"
            size="xs"
            :disabled="pending || choice.options.length <= 2"
            @click="removeOption( option.id )"
          />
        </div>
        <label :for="`choice-option-${ option.id }`">Answer option</label>
        <UTextarea
          :id="`choice-option-${ option.id }`"
          v-model="option.text"
          :rows="2"
          :disabled="pending"
          class="w-full"
          :aria-label="`Option ${ index + 1 } text`"
        />
        <label :for="`choice-feedback-${ option.id }`">Feedback</label>
        <UTextarea
          :id="`choice-feedback-${ option.id }`"
          v-model="option.feedback"
          :rows="2"
          :disabled="pending"
          class="w-full"
          :aria-label="`Option ${ index + 1 } feedback`"
        />
      </fieldset>

      <UButton
        v-if="choice.options.length < 6"
        leading-icon="i-lucide-plus"
        color="neutral"
        variant="link"
        :disabled="pending"
        @click="choice.options.push( emptyChoiceOption() )"
      >
        Add option
      </UButton>
      <UCheckbox v-model="keepOrder" label="Keep option order" :disabled="pending" />
      <p v-if="error" role="status">{{ error }}</p>
    </template>
  </div>
</template>

<style scoped>
.practice-choice-editor,
.practice-choice-editor fieldset {
  display: grid;
  gap: 0.5rem;
  min-width: 0;
}

.practice-choice-editor fieldset {
  padding-block: 0.75rem;
  border-bottom: 1px solid var(--ui-border);
}

.practice-choice-editor legend {
  font-weight: 600;
}

.practice-choice-editor__heading,
.practice-choice-editor__heading label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.practice-choice-editor__heading {
  justify-content: space-between;
}

.practice-choice-editor input[type='radio'] {
  accent-color: var(--ui-primary);
}

.practice-choice-editor > button {
  justify-self: start;
}
</style>
