<script setup>
import { computed, ref, watch } from 'vue';
import ImageLabelCanvas from './ImageLabelCanvas.vue';
import {
  cloneImageLabelTargets,
  createImageLabelTarget,
  imageLabelErrors,
  normalizeImageLabelTargets,
  MAXIMUM_IMAGE_LABEL_TARGETS
} from '../image-labeling/targets';

const props = defineProps({
  open: { type: Boolean, default: false },
  targets: { type: Array, required: true },
  imageUrl: { type: String, required: true },
  alt: { type: String, default: '' },
  otherTargetCount: { type: Number, default: 0 }
});
const emit = defineEmits([ 'update:open', 'save' ]);
const draft = ref([]);
const selectedId = ref( '' );
const submitted = ref( false );
const zoom = ref( 1 );
const selectedIndex = computed( () => draft.value.findIndex( target => target.id === selectedId.value ) );
const selected = computed( () => draft.value[ selectedIndex.value ]);
const total = computed( () => draft.value.length + props.otherTargetCount );
const errors = computed( () => selected.value && submitted.value ? imageLabelErrors( selected.value ) : {});
const alternatives = computed({
  get: () => selected.value?.alternatives.join( '\n' ) ?? '',
  set: value => {
    selected.value.alternatives = value.split( '\n' );
  }
});
const zoomItems = [
  { label: 'Fit', value: 1 }, { label: '150%', value: 1.5 }, { label: '200%', value: 2 }
];

watch( () => props.open, open => {
  if ( open ) {
    draft.value = cloneImageLabelTargets( props.targets );
    selectedId.value = draft.value[ 0 ]?.id ?? '';
    submitted.value = false;
    zoom.value = 1;
  }
}, { immediate: true });

function addTarget() {
  if ( total.value >= MAXIMUM_IMAGE_LABEL_TARGETS ) {
    return;
  }

  const target = createImageLabelTarget();

  draft.value.push( target );
  selectedId.value = target.id;
}

function updatePosition( position ) {
  if ( selected.value ) {
    Object.assign( selected.value, position );
  }
}

function setCoordinate( field, event ) {
  const value = event.target.valueAsNumber;

  if ( Number.isFinite( value ) ) {
    updatePosition({ [ field ]: Math.max( 0, Math.min( 100, value ) ) / 100 });
  }

  event.target.value = selected.value[ field ] * 100;
}

function moveTarget( offset ) {
  const destination = selectedIndex.value + offset;

  if ( destination < 0 || destination >= draft.value.length ) {
    return;
  }

  const [ target ] = draft.value.splice( selectedIndex.value, 1 );

  draft.value.splice( destination, 0, target );
}

function removeTarget() {
  const index = selectedIndex.value;

  draft.value.splice( index, 1 );
  selectedId.value = draft.value[ index ]?.id ?? draft.value[ index - 1 ]?.id ?? '';
}

function save() {
  submitted.value = true;
  const invalid = draft.value.find( target => Object.values( imageLabelErrors( target ) ).some( Boolean ) );

  if ( invalid ) {
    selectedId.value = invalid.id;
    return;
  }

  if ( total.value > MAXIMUM_IMAGE_LABEL_TARGETS ) {
    return;
  }

  emit( 'save', normalizeImageLabelTargets( draft.value ) );
  emit( 'update:open', false );
}
</script>

<template>
  <UModal
    :open="open"
    title="Image labels"
    :ui="{ content: 'z-50 sm:max-w-5xl', overlay: 'z-40' }"
    @update:open="emit( 'update:open', $event )"
  >
    <template #body>
      <div class="image-label-editor" data-twill-image-label-editor>
        <section class="image-label-editor__workspace">
          <div class="image-label-editor__toolbar">
            <UButton
              type="button"
              size="sm"
              variant="subtle"
              leading-icon="i-lucide-plus"
              :disabled="total >= MAXIMUM_IMAGE_LABEL_TARGETS"
              @click="addTarget"
            >
              Add point
            </UButton>
            <USelect
              v-model="zoom"
              :items="zoomItems"
              aria-label="Image zoom"
              class="w-24"
            />
          </div>

          <p class="image-label-editor__hint">
            Add a point, then click the image to place it. Drag the point or use the arrow keys to move it.
          </p>

          <div class="image-label-editor__viewport">
            <div :style="{ width: `${ zoom * 100 }%` }">
              <ImageLabelCanvas
                :image-url="imageUrl"
                :alt="alt"
                :target="selected"
                :number="selectedIndex + 1"
                editable
                @position="updatePosition"
              />
            </div>
          </div>

          <ol v-if="draft.length" class="image-label-editor__targets" aria-label="Image targets">
            <li v-for="( target, index ) in draft" :key="target.id">
              <button type="button" :aria-pressed="target.id === selectedId" @click="selectedId = target.id">
                <span>{{ index + 1 }}</span>
                <span>{{ target.label || 'Unlabeled point' }}</span>
              </button>
            </li>
          </ol>
          <p v-if="total >= MAXIMUM_IMAGE_LABEL_TARGETS" class="image-label-editor__hint">
            This Prompt has {{ MAXIMUM_IMAGE_LABEL_TARGETS }} targets.
          </p>
        </section>

        <section v-if="selected" :key="selectedId" class="image-label-editor__fields">
          <div class="image-label-editor__toolbar">
            <h3>Target {{ selectedIndex + 1 }}</h3>
            <div class="flex gap-1">
              <UButton
                type="button"
                icon="i-lucide-chevron-up"
                aria-label="Move target up"
                color="neutral"
                variant="ghost"
                size="sm"
                :disabled="selectedIndex === 0"
                @click="moveTarget( -1 )"
              />
              <UButton
                type="button"
                icon="i-lucide-chevron-down"
                aria-label="Move target down"
                color="neutral"
                variant="ghost"
                size="sm"
                :disabled="selectedIndex === draft.length - 1"
                @click="moveTarget( 1 )"
              />
              <UButton
                type="button"
                icon="i-lucide-trash-2"
                aria-label="Remove target"
                color="error"
                variant="ghost"
                size="sm"
                @click="removeTarget"
              />
            </div>
          </div>

          <UFormField label="Expected label" :error="errors.label">
            <UInput v-model="selected.label" class="w-full" data-twill-label-answer />
          </UFormField>
          <UFormField
            label="Accepted alternatives"
            hint="Optional"
            description="Enter one answer per line."
            :error="errors.alternatives"
          >
            <UTextarea
              v-model="alternatives"
              :rows="2"
              class="w-full"
              data-twill-label-alternatives
            />
          </UFormField>
          <UFormField
            label="Position description"
            hint="Optional"
            description="Describe where to look without naming the answer."
            :error="errors.description"
          >
            <UTextarea
              v-model="selected.description"
              :rows="2"
              class="w-full"
              data-twill-label-description
            />
          </UFormField>

          <div class="image-label-editor__coordinates">
            <UFormField label="From left (%)">
              <input
                type="number"
                :value="Number( ( selected.x * 100 ).toFixed( 4 ) )"
                min="0"
                max="100"
                step="0.1"
                aria-label="From left (%)"
                @change="setCoordinate( 'x', $event )"
              >
            </UFormField>
            <UFormField label="From top (%)">
              <input
                type="number"
                :value="Number( ( selected.y * 100 ).toFixed( 4 ) )"
                min="0"
                max="100"
                step="0.1"
                aria-label="From top (%)"
                @change="setCoordinate( 'y', $event )"
              >
            </UFormField>
          </div>
        </section>
      </div>
    </template>

    <template #footer>
      <div class="flex w-full justify-end gap-2">
        <UButton
          type="button"
          color="neutral"
          variant="ghost"
          @click="emit( 'update:open', false )"
        >
          Cancel
        </UButton>
        <UButton type="button" variant="subtle" @click="save">Apply labels</UButton>
      </div>
    </template>
  </UModal>
</template>

<style scoped>
.image-label-editor {
  display: grid;
  grid-template-columns: minmax(0, 1.5fr) minmax(0, 1fr);
  gap: 1.25rem;
  align-items: start;
}

.image-label-editor__workspace,
.image-label-editor__fields {
  display: grid;
  min-width: 0;
  gap: 0.85rem;
}

.image-label-editor__toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
}

.image-label-editor__hint {
  margin: 0;
  color: var(--ui-text-muted);
  font-size: 0.8rem;
}

.image-label-editor__viewport {
  overflow: auto;
  border: 1px solid var(--ui-border);
}

.image-label-editor__targets {
  display: grid;
  margin: 0;
  padding: 0;
  list-style: none;
  max-height: 14rem;
  overflow: auto;
}

.image-label-editor__targets button {
  display: grid;
  grid-template-columns: 1.5rem minmax(0, 1fr);
  width: 100%;
  padding: 0.4rem;
  text-align: left;
  font-size: 0.85rem;
  overflow-wrap: anywhere;
  border: 1px solid transparent;
  border-radius: 3px;
}

.image-label-editor__targets button[aria-pressed='true'] {
  background: var(--ui-bg-elevated);
  border-color: var(--ui-border-accented);
}

.image-label-editor__targets button:focus-visible {
  outline: 2px solid var(--ui-primary);
  outline-offset: -2px;
}

.image-label-editor__coordinates {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.75rem;
}

.image-label-editor__coordinates input {
  width: 100%;
  min-width: 0;
  padding: 0.4rem 0.5rem;
  border: 1px solid var(--ui-border-accented);
  border-radius: 4px;
  background: var(--ui-bg);
  font-size: 0.85rem;
}

@media (max-width: 640px) {
  .image-label-editor {
    grid-template-columns: minmax(0, 1fr);
  }

  .image-label-editor__targets {
    max-height: 8rem;
  }
}
</style>
