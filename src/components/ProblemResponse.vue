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
        <p>Workpad text is optional and is not saved.</p>
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
      role="status"
      tabindex="-1"
    >
      <div class="problem-comparison__heading">
        <strong>Check your solution</strong>
        <p>Compare your work with the solution checkpoints, then grade your recall.</p>
      </div>

      <div class="problem-comparison__content">
        <div>
          <span>Your work</span>

          <p
            v-if="hasResponse"
            class="problem-comparison__attempt"
          >
            {{ modelValue }}
          </p>

          <p
            v-else
            class="problem-comparison__empty"
          >
            No workpad response.
          </p>
        </div>

        <div>
          <span>Solution checkpoints</span>

          <ol>
            <li
              v-for="checkpoint in settings.checkpoints"
              :key="checkpoint"
            >
              <span>{{ checkpoint }}</span>
            </li>
          </ol>
        </div>
      </div>
    </div>
  </section>
</template>
