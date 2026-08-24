<script setup>
import { computed, ref } from 'vue';

const props = defineProps({
  modelValue: {
    type: String,
    required: true
  },

  revealed: {
    type: Boolean,
    required: true
  },

  settings: {
    type: Object,
    required: true
  }
});

const emit = defineEmits([ 'update:modelValue' ]);

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
        <p>Use the Prompt as the topic. Scratchpad text is optional and is not saved.</p>
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
      role="status"
      tabindex="-1"
    >
      <div class="explain-comparison__heading">
        <strong>Compare your explanation</strong>
        <p>Check your attempt against the key points, then grade your recall.</p>
      </div>

      <div class="explain-comparison__content">
        <div>
          <span>Your explanation</span>

          <p
            v-if="hasResponse"
            class="explain-comparison__attempt"
          >
            {{ modelValue }}
          </p>

          <p
            v-else
            class="explain-comparison__empty"
          >
            No scratchpad response.
          </p>
        </div>

        <div>
          <span>Key points</span>

          <ul>
            <li
              v-for="keyPoint in settings.keyPoints"
              :key="keyPoint"
            >
              <UIcon
                name="i-lucide-check"
                aria-hidden="true"
              />

              <span>{{ keyPoint }}</span>
            </li>
          </ul>
        </div>
      </div>
    </div>
  </section>
</template>
