<script setup>
import { computed } from 'vue';
import {
  exampleFollowUpError,
  FOLLOW_UP_PROMPT_LIMIT,
  FOLLOW_UP_REFERENCE_LIMIT
} from '../explain/followUp';

const props = defineProps({
  modelValue: { type: Object, default: null },
  disabled: { type: Boolean, default: false },
  submitted: { type: Boolean, default: false }
});

const emit = defineEmits([ 'update:modelValue' ]);
const error = computed( () => (
  props.submitted ? exampleFollowUpError( props.modelValue ) : ''
) );

function update( field, value ) {
  emit( 'update:modelValue', { ...props.modelValue, [ field ]: value });
}
</script>

<template>
  <div class="example-follow-up-editor" data-twill-follow-up-editor>
    <UButton
      v-if="!modelValue"
      type="button"
      color="neutral"
      variant="link"
      size="sm"
      :disabled="disabled"
      @click="emit( 'update:modelValue', { prompt: '', reference: '' } )"
    >
      Add follow-up
    </UButton>
    <template v-else>
      <UFormField label="Follow-up question" description="Offer another case or change to explore after comparison." required>
        <UTextarea
          :model-value="modelValue.prompt"
          aria-label="Follow-up question"
          :rows="2"
          :maxrows="6"
          autoresize
          class="w-full"
          :disabled="disabled"
          @update:model-value="update( 'prompt', $event )"
        />
        <template #hint>{{ Array.from( modelValue.prompt ).length }} / {{ FOLLOW_UP_PROMPT_LIMIT }}</template>
      </UFormField>
      <UFormField label="Follow-up reference" description="Provide a checked example and explain the reasoning." required>
        <UTextarea
          :model-value="modelValue.reference"
          aria-label="Follow-up reference"
          :rows="3"
          :maxrows="8"
          autoresize
          class="w-full"
          :disabled="disabled"
          @update:model-value="update( 'reference', $event )"
        />
        <template #hint>{{ Array.from( modelValue.reference ).length }} / {{ FOLLOW_UP_REFERENCE_LIMIT }}</template>
      </UFormField>
      <p v-if="error" class="editor-field-error" role="alert">{{ error }}</p>
      <UButton
        type="button"
        color="neutral"
        variant="link"
        size="sm"
        :disabled="disabled"
        @click="emit( 'update:modelValue', null )"
      >
        Remove follow-up
      </UButton>
    </template>
  </div>
</template>

<style scoped>
.example-follow-up-editor {
  display: grid;
  gap: 0.75rem;
  min-width: 0;
}

.example-follow-up-editor > button {
  justify-self: start;
}
</style>
