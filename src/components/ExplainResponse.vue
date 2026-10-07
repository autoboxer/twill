<script setup>
import { computed, ref } from 'vue';
import StudyIdeaList from './StudyIdeaList.vue';

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
const focusInstruction = computed( () => ({
  causeAndEffect: 'Explain the causes and effects',
  compareAndContrast: 'Compare and contrast',
  how: 'Explain how',
  why: 'Explain why'
})[ props.settings.focus ] ?? 'Build an explanation' );

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
  >
    <header class="explain-response__heading">
      <span aria-hidden="true">
        <UIcon name="i-lucide-message-square-text" />
      </span>

      <div>
        <strong>{{ focusInstruction }}</strong>
      </div>
    </header>

    <UFormField
      v-if="!revealed"
      label="Scratchpad"
    >
      <UTextarea
        :model-value="modelValue"
        placeholder="Write an explanation"
        :rows="5"
        autoresize
        :maxrows="10"
        class="explain-response__input"
        @update:model-value="emit( 'update:modelValue', $event )"
      />
    </UFormField>

    <div
      v-else
      ref="result"
      class="explain-comparison"
      role="group"
      aria-label="Compare your explanation"
      tabindex="-1"
    >
      <div class="explain-comparison__heading">
        <strong>Compare your explanation</strong>
      </div>

      <div
        class="explain-comparison__content"
        :class="{ 'explain-comparison__content--written': hasResponse }"
      >
        <div v-if="hasResponse">
          <span>Your explanation</span>

          <p class="explain-comparison__attempt">
            {{ modelValue }}
          </p>
        </div>

        <div>
          <span>Key points</span>

          <StudyIdeaList
            :items="settings.keyPoints"
            kind="idea"
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
