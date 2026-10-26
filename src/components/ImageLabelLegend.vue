<script setup>
import { ref } from 'vue';
import { imageTargetDescription } from '../image-labeling/targets';

defineProps({
  targets: { type: Array, required: true },
  selectedId: { type: String, default: '' }
});
const emit = defineEmits([ 'select' ]);
const showLabels = ref( false );
</script>

<template>
  <div class="image-label-preview" contenteditable="false" data-twill-image-label-preview>
    <div class="image-label-preview__heading">
      <span>Image labels</span>
      <UButton
        type="button"
        color="neutral"
        variant="link"
        size="sm"
        :aria-expanded="showLabels"
        @click="showLabels = !showLabels"
      >
        {{ showLabels ? 'Hide labels' : 'Show labels' }}
      </UButton>
    </div>

    <ol class="image-label-preview__targets" aria-label="Image targets">
      <li v-for="( target, index ) in targets" :key="target.id">
        <button
          type="button"
          :aria-pressed="selectedId === target.id"
          :aria-label="showLabels ? undefined : imageTargetDescription( target, index + 1 )"
          @click="emit( 'select', target.id )"
        >
          <span>{{ index + 1 }}</span>
          <span v-if="showLabels">
            {{ target.label }}
            <small v-if="target.alternatives.length">Also accepted: {{ target.alternatives.join( '; ' ) }}</small>
          </span>
          <span v-else>{{ target.description || `Target ${ index + 1 }` }}</span>
        </button>
      </li>
    </ol>
  </div>
</template>

<style scoped>
.image-label-preview__heading {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 0.5rem;
  margin-top: 0.5rem;
  color: var(--ui-text-muted);
  font-size: 0.8rem;
}

.image-label-preview .image-label-preview__targets {
  margin: 0;
  padding: 0;
  list-style: none;
}

.image-label-preview .image-label-preview__targets > li {
  padding: 0;
  margin: 0;
}

.image-label-preview__targets button {
  display: grid;
  grid-template-columns: 1.5rem minmax(0, 1fr);
  width: 100%;
  padding: 0.4rem;
  text-align: left;
  border: 1px solid transparent;
  border-radius: 3px;
  overflow-wrap: anywhere;
  font-size: 0.85rem;
}

.image-label-preview__targets button[aria-pressed='true'] {
  background: var(--ui-bg-elevated);
  border-color: var(--ui-border-accented);
}

.image-label-preview__targets button:focus-visible {
  outline: 2px solid var(--ui-primary);
  outline-offset: -2px;
}

.image-label-preview__targets small {
  display: block;
  color: var(--ui-text-muted);
  font-size: 0.75rem;
}
</style>
