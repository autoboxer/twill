<script setup>
import { nextTick, ref } from 'vue';

import { templateFields } from '../templates/defaults';

const props = defineProps({
  description: {
    type: String,
    default: ''
  },
  error: {
    type: String,
    default: ''
  },
  label: {
    type: String,
    required: true
  },
  modelValue: {
    type: String,
    required: true
  },
  rows: {
    type: Number,
    default: 10
  },
  showFields: {
    type: Boolean,
    default: true
  }
});

const emit = defineEmits([ 'update:modelValue' ]);

const input = ref( null );

async function insertField( field ) {
  const token = `{{ ${ field } }}`;
  const textarea = textareaElement();

  if ( !textarea ) {
    const separator = props.modelValue && !props.modelValue.endsWith( '\n' ) ? '\n' : '';

    emit( 'update:modelValue', `${ props.modelValue }${ separator }${ token }` );
    return;
  }

  const start = textarea.selectionStart;
  const end = textarea.selectionEnd;
  const value = `${ props.modelValue.slice( 0, start ) }${ token }${ props.modelValue.slice( end ) }`;
  const cursor = start + token.length;

  emit( 'update:modelValue', value );
  await nextTick();

  const updatedTextarea = textareaElement();

  updatedTextarea?.focus();
  updatedTextarea?.setSelectionRange( cursor, cursor );
}

function textareaElement() {
  const root = input.value?.$el;

  if ( root?.matches?.( 'textarea' ) ) {
    return root;
  }

  return root?.querySelector?.( 'textarea' ) ?? null;
}
</script>

<template>
  <div class="template-markup-editor">
    <div class="template-markup-editor__heading">
      <div>
        <label>{{ label }}</label>
        <p v-if="description">{{ description }}</p>
      </div>

      <div
        v-if="showFields"
        class="template-token-buttons"
        :aria-label="`Insert a field into ${ label }`"
      >
        <UButton
          v-for="field in templateFields"
          :key="field.value"
          type="button"
          :label="field.label"
          :leading-icon="field.icon"
          color="neutral"
          variant="ghost"
          size="xs"
          @click="insertField( field.value )"
        />
      </div>
    </div>

    <UTextarea
      ref="input"
      :model-value="modelValue"
      :aria-label="label"
      :rows="rows"
      :class="{ 'template-code-input--error': error }"
      class="template-code-input"
      autoresize
      @update:model-value="emit( 'update:modelValue', $event )"
    />

    <p
      v-if="error"
      class="template-field-error"
    >
      {{ error }}
    </p>
  </div>
</template>
