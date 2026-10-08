<script setup>
import { comparisonChoicesForResponse } from '../study/comparison';

defineProps({
  label: {
    type: String,
    required: true
  },
  modelValue: {
    type: String,
    default: 'unchecked'
  },
  writtenResponse: {
    type: Boolean,
    default: false
  },
  disabled: {
    type: Boolean,
    default: false
  }
});

const emit = defineEmits([ 'update:modelValue' ]);
</script>

<template>
  <USelect
    :model-value="modelValue"
    :items="comparisonChoicesForResponse( writtenResponse )"
    :aria-label="`Compare ${ label }`"
    :disabled="disabled"
    variant="subtle"
    size="sm"
    class="study-idea-check"
    :class="{ 'study-idea-check--written': writtenResponse }"
    data-twill-idea-check
    @update:model-value="emit( 'update:modelValue', $event )"
  />
</template>
