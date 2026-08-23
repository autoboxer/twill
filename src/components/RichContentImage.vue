<script setup>
import { NodeViewWrapper } from '@tiptap/vue-3';
import { computed, ref } from 'vue';

import ImageOcclusionCanvas from './ImageOcclusionCanvas.vue';
import { useMediaImage } from '../composables/useMediaImage';
import {
  collectImageOcclusionGroups,
  createImageOcclusionId,
  imageOcclusionGroupIds,
  imageOcclusionRegions,
  MAXIMUM_IMAGE_OCCLUSION_GROUPS,
  MAXIMUM_IMAGE_OCCLUSION_REGIONS
} from '../image-occlusion/documents';

const props = defineProps({
  deleteNode: {
    type: Function,
    required: true
  },
  editor: {
    type: Object,
    required: true
  },
  extension: {
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

const alt = computed( () => props.node.attrs.alt ?? '' );
const draftRegions = ref([]);
const isEditable = computed( () => props.editor.isEditable );
const mediaId = computed( () => props.node.attrs.mediaId ?? '' );
const occlusionDialogOpen = ref( false );
const regions = computed( () => imageOcclusionRegions( props.node ) );
const studyDisplay = computed( () => {
  const configuredDisplay = props.extension.options.imageOcclusionDisplay;
  const display = typeof configuredDisplay === 'function'
    ? configuredDisplay()
    : configuredDisplay;
  const groupId = display?.groupId ?? '';

  if ( !regions.value.some( ( region ) => region.groupId === groupId ) ) {
    return { groupId: '', revealed: false };
  }

  return {
    groupId,
    revealed: Boolean( display.revealed )
  };
});
const studyGroupId = computed( () => studyDisplay.value.groupId );
const studyRevealed = computed( () => studyDisplay.value.revealed );
const selectedRegionId = ref( '' );
const imageOcclusionEnabled = computed( () => {
  const enabled = props.extension.options.imageOcclusionEnabled;

  return isEditable.value && Boolean(
    typeof enabled === 'function' ? enabled() : enabled
  );
});
const draftGroupIds = computed( () => imageOcclusionGroupIds( draftRegions.value ) );
const draftGroupNumbers = computed( () => new Map(
  draftGroupIds.value.map( ( groupId, index ) => [ groupId, index + 1 ])
) );
const promptDocument = computed( () => {
  const configuredDocument = props.extension.options.imageOcclusionDocument;

  return typeof configuredDocument === 'function'
    ? configuredDocument()
    : configuredDocument;
});
const persistedDocumentGroups = computed( () => (
  promptDocument.value ? collectImageOcclusionGroups( promptDocument.value ) : []
) );
const persistedGroupIds = computed( () => new Set(
  imageOcclusionGroupIds( regions.value )
) );
const otherGroupCount = computed( () => persistedDocumentGroups.value.filter( ( group ) => (
  !persistedGroupIds.value.has( group.id )
) ).length );
const otherRegionCount = computed( () => persistedDocumentGroups.value.reduce(
  ( count, group ) => persistedGroupIds.value.has( group.id )
    ? count
    : count + group.regions.length,
  0
) );
const totalGroupCount = computed( () => otherGroupCount.value + draftGroupIds.value.length );
const totalRegionCount = computed( () => otherRegionCount.value + draftRegions.value.length );
const selectedRegion = computed( () => draftRegions.value.find( ( region ) => (
  region.id === selectedRegionId.value
) ) );
const selectedGroupRegionCount = computed( () => draftRegions.value.filter( ( region ) => (
  region.groupId === selectedRegion.value?.groupId
) ).length );
const canCreateGroup = computed( () => (
  totalGroupCount.value < MAXIMUM_IMAGE_OCCLUSION_GROUPS
) );
const canAssignNewGroup = computed( () => (
  canCreateGroup.value || selectedGroupRegionCount.value === 1
) );
const groupItems = computed( () => [
  {
    disabled: !canAssignNewGroup.value,
    label: 'New card',
    value: 'new'
  },
  ...draftGroupIds.value.map( ( groupId, index ) => ({
    label: `Card ${ index + 1 }`,
    value: groupId
  }) )
]);
const newRegionGroupId = computed( () => canCreateGroup.value
  ? ''
  : selectedRegion.value?.groupId ?? draftGroupIds.value[ 0 ] ?? ''
);
const canAddRegion = computed( () => (
  totalRegionCount.value < MAXIMUM_IMAGE_OCCLUSION_REGIONS
  && ( canCreateGroup.value || Boolean( newRegionGroupId.value ) )
) );
const capacityMessage = computed( () => {
  if ( totalRegionCount.value >= MAXIMUM_IMAGE_OCCLUSION_REGIONS ) {
    return `This Prompt already has ${ MAXIMUM_IMAGE_OCCLUSION_REGIONS } masks.`;
  }

  if ( !canCreateGroup.value && draftGroupIds.value.length ) {
    return [
      `This Prompt has ${ MAXIMUM_IMAGE_OCCLUSION_GROUPS } image occlusion cards.`,
      'New masks join the selected card.'
    ].join( ' ' );
  }

  if ( !canCreateGroup.value ) {
    return `This Prompt already has ${ MAXIMUM_IMAGE_OCCLUSION_GROUPS } image occlusion cards.`;
  }

  return '';
});

const { imageError, imageLoading, imageUrl } = useMediaImage( mediaId );

function updateAlt( event ) {
  props.updateAttributes({
    alt: event.target.value || null
  });
}

function openOcclusionEditor() {
  draftRegions.value = regions.value.map( ( region ) => ({ ...region }) );
  selectedRegionId.value = draftRegions.value[ 0 ]?.id ?? '';
  occlusionDialogOpen.value = true;
}

function addCenteredRegion() {
  if ( !canAddRegion.value ) {
    return;
  }

  const offset = ( draftRegions.value.length % 6 ) * 0.025;
  const region = {
    groupId: newRegionGroupId.value || createImageOcclusionId(),
    height: 0.18,
    id: createImageOcclusionId(),
    width: 0.24,
    x: 0.38 + offset,
    y: 0.38 + offset
  };

  draftRegions.value = [ ...draftRegions.value, region ];
  selectedRegionId.value = region.id;
}

function updateDraftRegions( updatedRegions ) {
  draftRegions.value = updatedRegions;

  if (
    selectedRegionId.value
    && !updatedRegions.some( ( region ) => region.id === selectedRegionId.value )
  ) {
    selectedRegionId.value = updatedRegions[ 0 ]?.id ?? '';
  }
}

function updateSelectedGroup( groupId ) {
  if ( !selectedRegion.value ) {
    return;
  }

  if ( groupId === 'new' && !canAssignNewGroup.value ) {
    return;
  }

  const updatedGroupId = groupId === 'new'
    ? createImageOcclusionId()
    : groupId;

  draftRegions.value = draftRegions.value.map( ( region ) => (
    region.id === selectedRegionId.value
      ? { ...region, groupId: updatedGroupId }
      : region
  ) );
}

function removeSelectedRegion() {
  if ( !selectedRegion.value ) {
    return;
  }

  const index = draftRegions.value.findIndex( ( region ) => (
    region.id === selectedRegionId.value
  ) );
  const updatedRegions = draftRegions.value.filter( ( region ) => (
    region.id !== selectedRegionId.value
  ) );

  draftRegions.value = updatedRegions;
  selectedRegionId.value = updatedRegions[ index ]?.id
    ?? updatedRegions[ index - 1 ]?.id
    ?? '';
}

function saveOcclusionRegions() {
  props.updateAttributes({
    occlusionRegions: draftRegions.value.map( ( region ) => ({ ...region }) )
  });
  occlusionDialogOpen.value = false;
}

function regionCardLabel( region ) {
  return `Card ${ draftGroupNumbers.value.get( region.groupId ) ?? 1 }`;
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

      <ImageOcclusionCanvas
        v-else-if="studyGroupId"
        :alt="alt"
        :image-url="imageUrl"
        :regions="regions"
        :revealed="studyRevealed"
        :visible-group-id="studyGroupId"
      />

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
        v-if="imageOcclusionEnabled"
        type="button"
        leading-icon="i-lucide-square-pen"
        color="neutral"
        variant="soft"
        size="sm"
        :disabled="imageLoading || imageError"
        @click="openOcclusionEditor"
      >
        {{ regions.length ? 'Edit masks' : 'Add masks' }}
      </UButton>

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

    <UModal
      v-model:open="occlusionDialogOpen"
      title="Image occlusion"
      description="Draw rectangles over the areas to recall. Regions assigned to the same card are hidden together."
      :ui="{
        content: 'image-occlusion-modal z-50 sm:max-w-6xl',
        overlay: 'z-40'
      }"
    >
      <template #body>
        <div class="image-occlusion-editor">
          <section class="image-occlusion-editor__workspace">
            <p>
              Drag on the image to add a mask. Drag a mask to move it or use its corner handles to resize it.
            </p>

            <ImageOcclusionCanvas
              :alt="alt"
              :can-create="canAddRegion"
              editable
              :image-url="imageUrl"
              :new-region-group-id="newRegionGroupId"
              :regions="draftRegions"
              :selected-region-id="selectedRegionId"
              @update:regions="updateDraftRegions"
              @update:selected-region-id="selectedRegionId = $event"
            />
          </section>

          <aside class="image-occlusion-editor__sidebar">
            <div class="image-occlusion-editor__sidebar-heading">
              <div>
                <h3>Masks</h3>
                <p>
                  <span>{{ totalRegionCount }} masks</span>
                  <span>{{ totalGroupCount }} cards in Prompt</span>
                </p>
              </div>

              <UButton
                type="button"
                leading-icon="i-lucide-plus"
                color="neutral"
                variant="soft"
                size="sm"
                :disabled="!canAddRegion"
                class="image-occlusion-editor__add"
                @click="addCenteredRegion"
              >
                Add mask
              </UButton>
            </div>

            <p
              v-if="capacityMessage"
              class="image-occlusion-editor__capacity"
            >
              {{ capacityMessage }}
            </p>

            <div
              v-if="draftRegions.length"
              class="image-occlusion-mask-list"
            >
              <button
                v-for="( region, index ) in draftRegions"
                :key="region.id"
                type="button"
                :class="{
                  'image-occlusion-mask-list__item--active': region.id === selectedRegionId
                }"
                class="image-occlusion-mask-list__item"
                @click="selectedRegionId = region.id"
              >
                <span>Mask {{ index + 1 }}</span>
                <small>{{ regionCardLabel( region ) }}</small>
              </button>
            </div>

            <p
              v-else
              class="image-occlusion-editor__empty"
            >
              Draw on the image or add a centered mask.
            </p>

            <div
              v-if="selectedRegion"
              class="image-occlusion-region-settings"
            >
              <UFormField label="Card">
                <USelect
                  :model-value="selectedRegion.groupId"
                  :items="groupItems"
                  value-key="value"
                  class="w-full"
                  @update:model-value="updateSelectedGroup"
                />
              </UFormField>

              <p>
                Arrow keys move the selected mask. Hold Shift with an arrow key to resize it.
              </p>

              <UButton
                type="button"
                leading-icon="i-lucide-trash-2"
                color="error"
                variant="ghost"
                @click="removeSelectedRegion"
              >
                Remove mask
              </UButton>
            </div>
          </aside>
        </div>
      </template>

      <template #footer>
        <div class="dialog-actions">
          <UButton
            type="button"
            color="neutral"
            variant="ghost"
            @click="occlusionDialogOpen = false"
          >
            Cancel
          </UButton>

          <UButton
            type="button"
            @click="saveOcclusionRegions"
          >
            Save masks
          </UButton>
        </div>
      </template>
    </UModal>
  </NodeViewWrapper>
</template>
