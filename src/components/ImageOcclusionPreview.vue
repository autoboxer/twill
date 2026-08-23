<script setup>
import { computed, ref, watch } from 'vue';

import ImageOcclusionCanvas from './ImageOcclusionCanvas.vue';
import { useMediaImage } from '../composables/useMediaImage';
import { collectImageOcclusionGroups } from '../image-occlusion/documents';

const props = defineProps({
  document: {
    type: Object,
    required: true
  }
});

const activeGroupId = ref( '' );
const groups = computed( () => collectImageOcclusionGroups( props.document ) );
const activeGroup = computed( () => groups.value.find( ( group ) => (
  group.id === activeGroupId.value
) ) );
const groupItems = computed( () => groups.value.map( ( group, index ) => ({
  label: `Card ${ index + 1 } · ${ group.regions.length } ${
    group.regions.length === 1 ? 'mask' : 'masks'
  }`,
  value: group.id
}) ) );
const mediaId = computed( () => activeGroup.value?.image.mediaId ?? '' );

const { imageError, imageLoading, imageUrl } = useMediaImage( mediaId );

watch( groups, ( currentGroups ) => {
  if ( !currentGroups.some( ( group ) => group.id === activeGroupId.value ) ) {
    activeGroupId.value = currentGroups[ 0 ]?.id ?? '';
  }
}, { immediate: true });
</script>

<template>
  <section class="image-occlusion-preview">
    <div class="image-occlusion-preview__heading">
      <div>
        <h3>Image occlusion preview</h3>
        <p>Every card hides the regions assigned to the same group.</p>
      </div>

      <USelect
        v-if="groups.length > 1"
        v-model="activeGroupId"
        :items="groupItems"
        value-key="value"
        aria-label="Preview image occlusion card"
        class="image-occlusion-preview__select"
      />
    </div>

    <div
      v-if="activeGroup"
      class="image-occlusion-preview__card"
    >
      <span>
        Card {{ groups.findIndex( ( group ) => group.id === activeGroupId ) + 1 }}
      </span>

      <div
        v-if="imageLoading"
        class="image-occlusion-preview__state"
        aria-label="Loading image occlusion preview"
      >
        <UIcon name="i-lucide-loader-circle" />
      </div>

      <div
        v-else-if="imageError"
        class="image-occlusion-preview__state"
        role="status"
      >
        <UIcon name="i-lucide-image-off" />
        <span>Image could not be loaded.</span>
      </div>

      <ImageOcclusionCanvas
        v-else
        :alt="activeGroup.image.alt"
        :image-url="imageUrl"
        :regions="activeGroup.regions"
        :visible-group-id="activeGroupId"
      />
    </div>

    <p
      v-else
      class="image-occlusion-preview__empty"
    >
      Add a Prompt image, then edit its masks.
    </p>
  </section>
</template>
