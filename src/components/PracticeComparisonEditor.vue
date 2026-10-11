<script setup>
import { computed, ref, toRaw, watch } from 'vue';

import ConfirmDialog from './ConfirmDialog.vue';
import PracticeChoiceEditor from './PracticeChoiceEditor.vue';
import { choiceError } from '../practice/choices';

const props = defineProps({
  link: { type: Object, default: null },
  pending: { type: Boolean, default: false },
  error: { type: String, default: '' }
});
const emit = defineEmits([ 'close', 'save' ]);

const prompt = ref( '' );
const explanation = ref( '' );
const choice = ref( null );
const removeOpen = ref( false );
const open = computed({
  get: () => Boolean( props.link ),
  set: ( value ) => {
    if ( !value && !props.pending ) {
      emit( 'close' );
    }
  }
});
const promptLength = computed( () => Array.from( prompt.value.trim() ).length );
const explanationLength = computed( () => Array.from( explanation.value.trim() ).length );
const valid = computed( () => promptLength.value > 0 && promptLength.value <= 500
  && explanationLength.value > 0 && explanationLength.value <= 2_000 && !choiceError( choice.value ) );

watch( () => props.link, ( link ) => {
  prompt.value = link?.comparison?.prompt ?? '';
  explanation.value = link?.comparison?.explanation ?? '';
  choice.value = link?.comparison?.choice ? structuredClone( toRaw( link.comparison.choice ) ) : null;
  removeOpen.value = false;
});

function save() {
  if ( valid.value && !props.pending ) {
    emit( 'save', { prompt: prompt.value.trim(), explanation: explanation.value.trim(), choice: choice.value });
  }
}
</script>

<template>
  <UModal
    v-model:open="open"
    :title="link?.comparison ? 'Edit comparison' : 'Add comparison'"
    :description="link?.title"
    :dismissible="!pending && !removeOpen"
    :close="!pending && !removeOpen"
    :ui="{ content: 'rounded-md', description: 'break-words' }"
  >
    <template #body>
      <form id="practice-comparison-form" class="practice-comparison-form" @submit.prevent="save">
        <UAlert
          v-if="error"
          :description="error"
          color="error"
          variant="subtle"
          role="alert"
        />

        <div>
          <label for="comparison-prompt">Comparison prompt</label>
          <UTextarea
            id="comparison-prompt"
            v-model="prompt"
            :rows="3"
            :disabled="pending"
            class="w-full"
            placeholder="Which requirement changes the transport you would choose?"
          />
          <p v-if="promptLength > 500" role="alert">Use no more than 500 characters.</p>
        </div>

        <div>
          <label for="comparison-explanation">Expected explanation</label>
          <UTextarea
            id="comparison-explanation"
            v-model="explanation"
            :rows="5"
            :disabled="pending"
            class="w-full"
          />
          <p v-if="explanationLength > 2000" role="alert">Use no more than 2,000 characters.</p>
        </div>

        <PracticeChoiceEditor v-model="choice" :pending="pending" />
      </form>
    </template>

    <template #footer>
      <div class="dialog-actions">
        <UButton
          v-if="link?.comparison"
          color="error"
          variant="link"
          :disabled="pending"
          @click="removeOpen = true"
        >
          Remove comparison
        </UButton>
        <UButton
          color="neutral"
          variant="link"
          :disabled="pending"
          @click="open = false"
        >
          Cancel
        </UButton>
        <UButton
          type="submit"
          form="practice-comparison-form"
          variant="subtle"
          :loading="pending"
          :disabled="pending || !valid"
        >
          Save comparison
        </UButton>
      </div>
    </template>
  </UModal>

  <ConfirmDialog
    v-model:open="removeOpen"
    title="Remove comparison?"
    description="The link and both concepts will stay in your library."
    confirm-label="Remove comparison"
    :loading="pending"
    :error="error"
    @confirm="emit('save', null)"
  />
</template>

<style scoped>
.practice-comparison-form {
  display: grid;
  gap: 1rem;
}

.practice-comparison-form label {
  display: block;
  margin-bottom: 0.35rem;
}
</style>
