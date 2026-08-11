<script setup>
import { computed } from 'vue';

import {
  MAXIMUM_TEMPLATE_BLOCKS_PER_SIDE,
  templateFields
} from '../templates/defaults';

const props = defineProps({
  error: {
    type: String,
    default: ''
  },
  label: {
    type: String,
    required: true
  },
  modelValue: {
    type: Array,
    required: true
  }
});

const emit = defineEmits([ 'update:modelValue' ]);

const atBlockLimit = computed( () => {
  return props.modelValue.length >= MAXIMUM_TEMPLATE_BLOCKS_PER_SIDE;
});
const availableFields = computed( () => templateFields.map( ( field ) => ({
  ...field,
  used: props.modelValue.some( ( block ) => {
    return block.type === 'field' && block.field === field.value;
  })
}) ) );

function fieldDetails( value ) {
  return templateFields.find( ( field ) => field.value === value );
}

function addField( field ) {
  if (
    atBlockLimit.value
    || availableFields.value.find( ( item ) => item.value === field )?.used
  ) {
    return;
  }

  updateBlocks([
    ...props.modelValue,
    { type: 'field', field }
  ]);
}

function addText() {
  if ( atBlockLimit.value ) {
    return;
  }

  updateBlocks([
    ...props.modelValue,
    { type: 'text', text: '' }
  ]);
}

function updateText( index, text ) {
  const blocks = props.modelValue.map( ( block, blockIndex ) => {
    return blockIndex === index ? { ...block, text } : { ...block };
  });

  updateBlocks( blocks );
}

function moveBlock( index, direction ) {
  const target = index + direction;

  if ( target < 0 || target >= props.modelValue.length ) {
    return;
  }

  const blocks = props.modelValue.map( ( block ) => ({
    ...block
  }) );

  [ blocks[ index ], blocks[ target ] ] = [ blocks[ target ], blocks[ index ] ];
  updateBlocks( blocks );
}

function removeBlock( index ) {
  updateBlocks( props.modelValue.filter( ( _, blockIndex ) => blockIndex !== index ) );
}

function updateBlocks( blocks ) {
  emit( 'update:modelValue', blocks );
}
</script>

<template>
  <section class="template-side-editor">
    <header class="template-side-editor__header">
      <div>
        <h3>{{ label }}</h3>
        <p>Fields and text appear in this order.</p>
      </div>

      <UBadge
        :label="`${ modelValue.length } ${ modelValue.length === 1 ? 'block' : 'blocks' }`"
        color="neutral"
        variant="soft"
      />
    </header>

    <div
      v-if="modelValue.length"
      class="template-block-list"
    >
      <div
        v-for="( block, index ) in modelValue"
        :key="`${ block.type }-${ block.field ?? index }-${ index }`"
        class="template-block"
      >
        <span
          class="template-block__icon"
          aria-hidden="true"
        >
          <UIcon
            :name="block.type === 'field'
              ? fieldDetails( block.field )?.icon
              : 'i-lucide-type'"
          />
        </span>

        <div
          v-if="block.type === 'field'"
          class="template-block__copy"
        >
          <strong>{{ fieldDetails( block.field )?.label }}</strong>
          <span>{{ fieldDetails( block.field )?.description }}</span>
        </div>

        <UInput
          v-else
          :model-value="block.text"
          :aria-label="`${ label } text block ${ index + 1 }`"
          placeholder="Text"
          :maxlength="1000"
          class="template-block__input"
          @update:model-value="updateText( index, $event )"
        />

        <div class="template-block__actions">
          <UButton
            type="button"
            icon="i-lucide-chevron-up"
            :aria-label="`Move ${ label } block ${ index + 1 } up`"
            color="neutral"
            variant="ghost"
            size="sm"
            square
            :disabled="index === 0"
            @click="moveBlock( index, -1 )"
          />

          <UButton
            type="button"
            icon="i-lucide-chevron-down"
            :aria-label="`Move ${ label } block ${ index + 1 } down`"
            color="neutral"
            variant="ghost"
            size="sm"
            square
            :disabled="index === modelValue.length - 1"
            @click="moveBlock( index, 1 )"
          />

          <UButton
            type="button"
            icon="i-lucide-x"
            :aria-label="`Remove ${ label } block ${ index + 1 }`"
            color="error"
            variant="ghost"
            size="sm"
            square
            @click="removeBlock( index )"
          />
        </div>
      </div>
    </div>

    <div
      v-else
      class="template-block-list__empty"
    >
      Add at least one field.
    </div>

    <p
      v-if="error"
      class="template-field-error"
    >
      {{ error }}
    </p>

    <div class="template-insert-controls">
      <span>Add field</span>

      <UButton
        v-for="field in availableFields"
        :key="field.value"
        type="button"
        :label="field.label"
        :leading-icon="field.icon"
        :disabled="field.used || atBlockLimit"
        color="neutral"
        variant="soft"
        size="sm"
        @click="addField( field.value )"
      />

      <UButton
        type="button"
        label="Text"
        leading-icon="i-lucide-type"
        color="neutral"
        variant="soft"
        size="sm"
        :disabled="atBlockLimit"
        @click="addText"
      />
    </div>
  </section>
</template>
