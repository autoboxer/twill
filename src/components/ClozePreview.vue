<script setup>
import { computed, ref, watch } from 'vue';

import RichContentRenderer from './RichContentRenderer.vue';
import {
  collectClozeGroups,
  createClozePrompt
} from '../cloze/documents';

const props = defineProps({
  document: {
    type: Object,
    required: true
  }
});

const activeGroupId = ref( '' );
const groups = computed( () => collectClozeGroups( props.document ) );
const groupItems = computed( () => groups.value.map( ( group, index ) => ({
  label: `Card ${ index + 1 }`,
  value: group.id
}) ) );
const previewDocument = computed( () => createClozePrompt(
  props.document,
  activeGroupId.value
) );

watch( groups, ( currentGroups ) => {
  if ( !currentGroups.some( ( group ) => group.id === activeGroupId.value ) ) {
    activeGroupId.value = currentGroups[ 0 ]?.id ?? '';
  }
}, { immediate: true });
</script>

<template>
  <section class="cloze-preview">
    <div class="cloze-preview__heading">
      <div>
        <h3>Cloze preview</h3>
        <p>Each card hides every passage assigned to the same group.</p>
      </div>

      <USelect
        v-if="groups.length > 1"
        v-model="activeGroupId"
        :items="groupItems"
        value-key="value"
        aria-label="Preview card"
        class="cloze-preview__select"
      />
    </div>

    <div
      v-if="groups.length"
      class="cloze-preview__card"
    >
      <span>Card {{ groups.findIndex( ( group ) => group.id === activeGroupId ) + 1 }}</span>

      <RichContentRenderer
        :document="previewDocument"
        label="Cloze prompt preview"
      />
    </div>

    <p
      v-else
      class="cloze-preview__empty"
    >
      Select Prompt text, then use the cloze omission button.
    </p>
  </section>
</template>
