<script setup>
import { computed } from 'vue';

import { assistanceDocuments } from '../study/assistance';
import RichContentRenderer from './RichContentRenderer.vue';

const props = defineProps({
  content: { type: Object, required: true },
  revealed: { type: Array, required: true },
  disabled: { type: Boolean, default: false }
});

const emit = defineEmits([ 'toggle' ]);
const documents = computed( () => assistanceDocuments( props.content ) );
</script>

<template>
  <section
    v-if="documents.length"
    class="study-assistance"
    aria-label="Optional help"
    data-twill-hints
  >
    <div class="study-assistance__controls">
      <UButton
        v-for="item in documents"
        :key="item.id"
        color="neutral"
        variant="link"
        size="sm"
        :leading-icon="item.id === 'hint' ? 'i-lucide-lightbulb' : 'i-lucide-book-open'"
        :disabled="disabled"
        :aria-expanded="revealed.includes( item.id )"
        :aria-controls="`study-help-${ item.id }`"
        @click="emit( 'toggle', item.id )"
      >
        {{ revealed.includes( item.id ) ? `Hide ${ item.label.toLowerCase() }` : item.label }}
      </UButton>
    </div>

    <template v-for="item in documents" :key="item.id">
      <div
        v-if="revealed.includes( item.id )"
        :id="`study-help-${ item.id }`"
        class="study-assistance__document"
        role="region"
        :aria-label="item.label"
      >
        <RichContentRenderer :document="item.document" :label="item.label" />
      </div>
    </template>
  </section>
</template>

<style scoped>
.study-assistance {
  display: grid;
  gap: 0.5rem;
  min-width: 0;
}

.study-assistance__controls {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.study-assistance__document {
  min-width: 0;
  padding: 0.5rem 0.75rem;
  border-left: 2px solid var(--ui-border-accented);
}
</style>
