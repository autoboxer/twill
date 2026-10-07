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
    class="problem-response"
  >
    <header class="problem-response__heading">
      <span aria-hidden="true">
        <UIcon name="i-lucide-list-ordered" />
      </span>

      <div>
        <strong>Work through the problem</strong>
      </div>
    </header>

    <UFormField
      v-if="!revealed"
      label="Workpad"
    >
      <UTextarea
        :model-value="modelValue"
        placeholder="Write your work"
        :rows="6"
        autoresize
        :maxrows="12"
        class="problem-response__input"
        @update:model-value="emit( 'update:modelValue', $event )"
      />
    </UFormField>

    <div
      v-else
      ref="result"
      class="problem-comparison"
      role="group"
      aria-label="Check your solution"
      tabindex="-1"
    >
      <div class="problem-comparison__heading">
        <strong>Check your solution</strong>
      </div>

      <div
        class="problem-comparison__content"
        :class="{ 'problem-comparison__content--written': hasResponse }"
      >
        <div v-if="hasResponse">
          <span>Your work</span>

          <p class="problem-comparison__attempt">
            {{ modelValue }}
          </p>
        </div>

        <div>
          <span>Solution checkpoints</span>

          <StudyIdeaList
            :items="settings.checkpoints"
            kind="checkpoint"
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
