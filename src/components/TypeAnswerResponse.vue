<script setup>
import { computed, ref } from 'vue';

import {
  compareTypeAnswer,
  normalizeTypeAnswer
} from '../type-answer/comparison';

const props = defineProps({
  acceptedAnswers: {
    type: Array,
    required: true
  },

  modelValue: {
    type: String,
    required: true
  },

  revealed: {
    type: Boolean,
    required: true
  }
});

const emit = defineEmits([ 'submit', 'update:modelValue' ]);

const result = ref( null );
const root = ref( null );

const comparison = computed( () => props.revealed
  ? compareTypeAnswer( props.modelValue, props.acceptedAnswers )
  : null
);

defineExpose({ focus });

function focus() {
  if ( props.revealed ) {
    result.value?.focus();
    return;
  }

  root.value?.querySelector( 'input' )?.focus();
}

function submit( event ) {
  if ( event?.isComposing || !normalizeTypeAnswer( props.modelValue ) ) {
    return;
  }

  event?.preventDefault();
  emit( 'submit' );
}
</script>

<template>
  <section
    ref="root"
    class="type-answer-response"
  >
    <UFormField
      v-if="!revealed"
      label="Your answer"
      hint="Required before reveal"
      required
    >
      <UInput
        :model-value="modelValue"
        :maxlength="1000"
        placeholder="Type your answer"
        autocomplete="off"
        autocapitalize="off"
        spellcheck="false"
        class="type-answer-response__input"
        size="xl"
        @update:model-value="emit( 'update:modelValue', $event )"
        @keydown.enter="submit"
      />
    </UFormField>

    <div
      v-else
      ref="result"
      class="type-answer-comparison"
      :class="{
        'type-answer-comparison--exact': comparison.exact
      }"
      role="status"
      tabindex="-1"
    >
      <header class="type-answer-comparison__heading">
        <span aria-hidden="true">
          <UIcon :name="comparison.exact ? 'i-lucide-check' : 'i-lucide-circle-alert'" />
        </span>

        <div>
          <strong>{{ comparison.exact ? 'Exact match' : 'Not an exact match' }}</strong>
          <p>
            {{ comparison.exact
              ? 'Your response matches an accepted answer.'
              : 'Compare your response with the closest accepted answer.' }}
          </p>
        </div>
      </header>

      <div class="type-answer-comparison__rows">
        <div>
          <span>Your response</span>
          <p>
            <template
              v-for="( segment, index ) in comparison.responseSegments"
              :key="index"
            >
              <mark v-if="segment.different">{{ segment.text }}</mark>
              <span v-else>{{ segment.text }}</span>
            </template>
          </p>
        </div>

        <div v-if="!comparison.exact">
          <span>Closest accepted answer</span>
          <p>
            <template
              v-for="( segment, index ) in comparison.acceptedSegments"
              :key="index"
            >
              <mark v-if="segment.different">{{ segment.text }}</mark>
              <span v-else>{{ segment.text }}</span>
            </template>
          </p>
        </div>
      </div>
    </div>
  </section>
</template>
