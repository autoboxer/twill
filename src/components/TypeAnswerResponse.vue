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

  optional: {
    type: Boolean,
    default: false
  },

  revealed: {
    type: Boolean,
    required: true
  }
});

const emit = defineEmits([ 'submit', 'update:modelValue' ]);

const result = ref( null );
const root = ref( null );

const comparison = computed( () => props.revealed && (
  !props.optional || normalizeTypeAnswer( props.modelValue )
)
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
  if (
    event?.isComposing
    || ( !props.optional && !normalizeTypeAnswer( props.modelValue ) )
  ) {
    return;
  }

  event?.preventDefault();
  emit( 'submit' );
}
</script>

<template>
  <section
    v-if="!revealed || comparison"
    ref="root"
    class="type-answer-response"
    :class="{ 'type-answer-response--optional': optional }"
  >
    <UFormField
      v-if="!revealed"
      :label="optional ? 'Your label' : 'Your answer'"
      :hint="optional ? 'Optional' : 'Required before reveal'"
      :required="!optional"
    >
      <UInput
        :model-value="modelValue"
        :maxlength="1000"
        :placeholder="optional ? 'Type a label' : 'Type your answer'"
        autocomplete="off"
        autocapitalize="off"
        spellcheck="false"
        class="type-answer-response__input"
        :size="optional ? 'md' : 'xl'"
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
