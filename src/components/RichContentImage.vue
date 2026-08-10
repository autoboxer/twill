<script setup>
import { NodeViewWrapper } from '@tiptap/vue-3';
import { computed, onBeforeUnmount, ref, watch } from 'vue';

import { useConceptLibrary } from '../composables/useConceptLibrary';

const props = defineProps({
  deleteNode: {
    type: Function,
    required: true
  },
  editor: {
    type: Object,
    required: true
  },
  node: {
    type: Object,
    required: true
  },
  updateAttributes: {
    type: Function,
    required: true
  }
});

const { readMedia } = useConceptLibrary();

const imageError = ref( false );
const imageLoading = ref( true );
const imageUrl = ref( '' );
let loadRequestSequence = 0;

const alt = computed( () => props.node.attrs.alt ?? '' );
const isEditable = computed( () => props.editor.isEditable );
const mediaId = computed( () => props.node.attrs.mediaId ?? '' );

watch( mediaId, loadImage, { immediate: true });

onBeforeUnmount( () => {
  loadRequestSequence += 1;
  releaseImageUrl();
});

async function loadImage() {
  const request = ++loadRequestSequence;
  const requestedMediaId = mediaId.value;

  releaseImageUrl();
  imageError.value = false;
  imageLoading.value = true;

  if ( !requestedMediaId ) {
    imageError.value = true;
    imageLoading.value = false;
    return;
  }

  try {
    const response = await readMedia( requestedMediaId );

    if ( request !== loadRequestSequence ) {
      return;
    }

    const bytes = normalizeBytes( response );

    imageUrl.value = URL.createObjectURL( new Blob([ bytes ]) );
  } catch {
    if ( request === loadRequestSequence ) {
      imageError.value = true;
    }
  } finally {
    if ( request === loadRequestSequence ) {
      imageLoading.value = false;
    }
  }
}

function normalizeBytes( value ) {
  if ( value instanceof ArrayBuffer || ArrayBuffer.isView( value ) ) {
    return value;
  }

  if ( Array.isArray( value ) ) {
    return Uint8Array.from( value );
  }

  throw new TypeError( 'Media response was not binary.' );
}

function releaseImageUrl() {
  if ( imageUrl.value ) {
    URL.revokeObjectURL( imageUrl.value );
    imageUrl.value = '';
  }
}

function updateAlt( event ) {
  props.updateAttributes({
    alt: event.target.value || null
  });
}
</script>

<template>
  <NodeViewWrapper
    as="figure"
    class="rich-image"
    :class="{ 'rich-image--editable': isEditable }"
  >
    <div
      class="rich-image__frame"
      data-drag-handle
    >
      <div
        v-if="imageLoading"
        class="rich-image__state"
        aria-label="Loading image"
      >
        <UIcon
          name="i-lucide-loader-circle"
          class="rich-image__spinner"
        />
      </div>

      <div
        v-else-if="imageError"
        class="rich-image__state rich-image__state--error"
        role="status"
      >
        <UIcon name="i-lucide-image-off" />
        <span>Image could not be loaded.</span>
      </div>

      <img
        v-else
        :src="imageUrl"
        :alt="alt"
        :title="node.attrs.title || undefined"
        draggable="false"
      >
    </div>

    <figcaption
      v-if="isEditable"
      class="rich-image__controls"
      contenteditable="false"
    >
      <label class="rich-image__alt">
        <span class="sr-only">Alternative text</span>
        <input
          :value="alt"
          type="text"
          maxlength="500"
          placeholder="Alternative text"
          @input="updateAlt"
        >
      </label>

      <UButton
        type="button"
        icon="i-lucide-trash-2"
        aria-label="Remove image"
        color="error"
        variant="ghost"
        size="sm"
        @click="deleteNode"
      />
    </figcaption>
  </NodeViewWrapper>
</template>
