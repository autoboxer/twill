<script setup>
import { computed } from 'vue';

import {
  createRichContentExtensions,
  richContentStarterKit
} from '../rich-content/schema';

const props = defineProps({
  document: {
    type: Object,
    required: true
  },
  imageOcclusionGroupId: {
    type: String,
    default: ''
  },
  imageOcclusionRevealed: {
    type: Boolean,
    default: false
  },
  label: {
    type: String,
    required: true
  }
});

const extensions = createRichContentExtensions({
  imageOcclusionDisplay: () => ({
    groupId: props.imageOcclusionGroupId,
    revealed: props.imageOcclusionRevealed
  })
});
const hasContent = computed( () => props.document.content?.some( ( node ) => {
  return node.type !== 'paragraph' || Boolean( node.content?.length );
}) );
const starterKit = richContentStarterKit( false );
</script>

<template>
  <UEditor
    v-if="hasContent"
    :model-value="document"
    :aria-label="label"
    :editable="false"
    :extensions="extensions"
    :image="false"
    :mention="false"
    :starter-kit="starterKit"
    content-type="json"
    class="rich-content-renderer"
  />

  <p
    v-else
    class="rich-content-renderer__empty"
  >
    No content.
  </p>
</template>
