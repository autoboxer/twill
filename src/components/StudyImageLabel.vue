<script setup>
import { computed, ref, watch } from 'vue';

import ImageLabelCanvas from './ImageLabelCanvas.vue';
import { useMediaImage } from '../composables/useMediaImage';
import { imageTargetDescription } from '../image-labeling/targets';

const props = defineProps({
  target: { type: Object, default: null },
  number: { type: Number, default: 1 },
  revealed: { type: Boolean, default: false }
});

const zoom = ref( 100 );
const imageRatio = ref( 1 );
const zoomItems = [
  { label: 'Fit', value: 100 },
  { label: '150%', value: 150 },
  { label: '200%', value: 200 }
];
const mediaId = computed( () => props.target?.image.mediaId ?? '' );
const { imageError, imageLoading, imageUrl } = useMediaImage( mediaId );

watch( () => props.target?.id, () => {
  zoom.value = 100;
});
</script>

<template>
  <section class="study-image-label" data-twill-study-image-label>
    <template v-if="target">
      <div class="study-image-label__heading">
        <p>Identify target {{ number }}.</p>
        <USelect
          v-model="zoom"
          :items="zoomItems"
          aria-label="Image zoom"
          size="sm"
        />
      </div>

      <p v-if="imageLoading" role="status">Loading image</p>
      <p v-else-if="imageError" role="alert">Image could not be loaded.</p>
      <div
        v-else
        class="study-image-label__viewport"
        :class="{ 'study-image-label__viewport--zoomed': zoom > 100 }"
        :style="{ '--image-ratio': imageRatio }"
        role="group"
        tabindex="0"
        aria-label="Target image"
      >
        <div class="study-image-label__surface" :style="{ width: `${ zoom }%` }">
          <ImageLabelCanvas
            :image-url="imageUrl"
            :target="target"
            :number="number"
            :alt="imageTargetDescription( target, number )"
            @load="imageRatio = $event.width / $event.height"
          />
        </div>
      </div>

      <p v-if="target.description" class="study-image-label__description">{{ target.description }}</p>

      <div v-if="revealed" class="study-image-label__answer" data-twill-image-label-answer>
        <strong>{{ target.label }}</strong>
        <p v-if="target.alternatives.length">Also accepted: {{ target.alternatives.join( ', ' ) }}</p>
      </div>
    </template>

    <p v-else role="alert">This image target is no longer available. Edit the concept before studying it.</p>
  </section>
</template>

<style scoped>
.study-image-label {
  display: grid;
  gap: 0.65rem;
  min-width: 0;
  overflow-wrap: anywhere;
}

.study-image-label__heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
}

.study-image-label__viewport {
  overflow: auto;
  width: 100%;
  max-width: min(100%, calc(60vh * var(--image-ratio)), calc(38rem * var(--image-ratio)));
  border: 1px solid var(--ui-border);
  border-radius: 4px;
}

.study-image-label__viewport--zoomed {
  max-height: min(60vh, 38rem);
}

.study-image-label__surface {
  overflow: hidden;
}

.study-image-label__viewport:focus-visible {
  outline: 2px solid var(--ui-primary);
  outline-offset: 2px;
}

.study-image-label__description,
.study-image-label__answer p {
  color: var(--ui-text-muted);
  font-size: 0.875rem;
}

.study-image-label__answer {
  display: grid;
  gap: 0.25rem;
  padding-top: 0.65rem;
  border-top: 1px solid var(--ui-border);
}
</style>
