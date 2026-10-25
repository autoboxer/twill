<script setup>
import { computed, ref, watch } from 'vue';

import StudyImageLabel from './StudyImageLabel.vue';
import { collectImageLabelTargets } from '../image-labeling/targets';

const props = defineProps({ document: { type: Object, required: true } });
const activeId = ref( '' );
const revealed = ref( false );
const targets = computed( () => collectImageLabelTargets( props.document ) );
const index = computed( () => targets.value.findIndex( target => target.id === activeId.value ) );
const target = computed( () => targets.value[ index.value ]);
const items = computed( () => targets.value.map( ( target, index ) => ({
  label: `Target ${ index + 1 }`, value: target.id
}) ) );

watch( targets, ( values ) => {
  if ( !values.some( target => target.id === activeId.value ) ) {
    activeId.value = values[ 0 ]?.id ?? '';
  }

  revealed.value = false;
}, { immediate: true });

watch( activeId, () => {
  revealed.value = false;
});
</script>

<template>
  <section class="image-label-card-preview" aria-label="Image labeling preview">
    <template v-if="target">
      <div class="editor-section__heading">
        <h3>Preview</h3>
        <USelect
          v-if="items.length > 1"
          v-model="activeId"
          :items="items"
          aria-label="Preview image target"
        />
        <UButton
          type="button"
          color="neutral"
          variant="link"
          :aria-pressed="revealed"
          @click="revealed = !revealed"
        >
          {{ revealed ? 'Show front' : 'Show answer' }}
        </UButton>
      </div>
      <StudyImageLabel :target="target" :number="index + 1" :revealed="revealed" />
    </template>
    <p v-else>Add a Prompt image, then choose Add labels.</p>
  </section>
</template>

<style scoped>
.image-label-card-preview {
  display: grid;
  gap: 0.75rem;
  min-width: 0;
}
</style>
