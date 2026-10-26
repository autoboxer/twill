<script setup>
import { computed, ref } from 'vue';
import StudyIdeaList from './StudyIdeaList.vue';
import { explainFocusDetails } from '../explain/focus';

const props = defineProps({
  modelValue: {
    type: String,
    required: true
  },

  revealed: {
    type: Boolean,
    required: true
  },

  checks: {
    type: Object,
    default: () => ({})
  },

  disabled: {
    type: Boolean,
    default: false
  },

  settings: {
    type: Object,
    required: true
  }
});

const emit = defineEmits([ 'update:modelValue', 'compare' ]);

const result = ref( null );
const root = ref( null );

const hasResponse = computed( () => Boolean( props.modelValue.trim() ) );
const focusDetails = computed( () => explainFocusDetails( props.settings.focus ) );

defineExpose({ focus });

function focus() {
  if ( props.revealed ) {
    result.value?.focus();
    return;
  }

  root.value?.querySelector( 'textarea' )?.focus();
}
</script>

<template>
  <section
    ref="root"
    class="explain-response"
    :data-twill-reconstruction="focusDetails.reconstruction || undefined"
  >
    <header class="explain-response__heading">
      <span aria-hidden="true">
        <UIcon name="i-lucide-message-square-text" />
      </span>

      <div>
        <strong>{{ focusDetails.instruction }}</strong>
      </div>
    </header>

    <UFormField
      v-if="!revealed"
      :label="focusDetails.inputLabel"
    >
      <UTextarea
        :model-value="modelValue"
        :placeholder="focusDetails.placeholder"
        :rows="5"
        autoresize
        :maxrows="10"
        class="explain-response__input"
        :disabled="disabled"
        @update:model-value="emit( 'update:modelValue', $event )"
      />
    </UFormField>

    <div
      v-else
      ref="result"
      class="explain-comparison"
      role="group"
      :aria-label="focusDetails.comparison"
      tabindex="-1"
    >
      <div class="explain-comparison__heading">
        <strong>{{ focusDetails.comparison }}</strong>
      </div>

      <div
        class="explain-comparison__content"
        :class="{ 'explain-comparison__content--written': hasResponse }"
      >
        <div v-if="hasResponse">
          <span>{{ focusDetails.response }}</span>

          <p class="explain-comparison__attempt">
            {{ modelValue }}
          </p>
        </div>

        <div>
          <span>{{ focusDetails.criteria }}</span>

          <StudyIdeaList
            :items="settings.keyPoints"
            kind="idea"
            :ordered="settings.focus === 'sequence'"
            :item-label="focusDetails.reconstruction ? focusDetails.criterion.toLowerCase() : ''"
            :checks="checks"
            :written-response="hasResponse"
            :disabled="disabled"
            @compare="( id, value ) => emit( 'compare', id, value )"
          />
        </div>
      </div>
    </div>
  </section>
</template>
