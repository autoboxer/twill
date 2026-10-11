<script setup>
import { nextTick } from 'vue';

import PracticeComparison from './PracticeComparison.vue';

defineProps({
  open: { type: Boolean, default: false },
  activity: { type: Object, required: true }
});
const emit = defineEmits([ 'close', 'closed', 'pause' ]);
const response = defineModel( 'response', { type: String, default: '' });
const revealed = defineModel( 'revealed', { type: Boolean, default: false });
const choiceState = defineModel( 'choiceState', { type: Object, default: null });

async function restoreStudyFocus( event ) {
  event.preventDefault();
  await nextTick();

  emit( 'closed' );
}
</script>

<template>
  <UModal
    :open="open"
    title="Compare cases"
    description="Compare the feature that changes which method fits."
    :content="{ onCloseAutoFocus: restoreStudyFocus }"
    :ui="{ overlay: 'z-70', content: 'z-71 rounded-md sm:max-w-3xl', wrapper: 'pr-10' }"
    @update:open="!$event && emit('close')"
  >
    <template #body>
      <PracticeComparison
        v-model:response="response"
        v-model:revealed="revealed"
        v-model:choice-state="choiceState"
        :activity="activity"
        data-twill-study-comparison
      />
    </template>

    <template #footer>
      <div class="dialog-actions">
        <UButton
          leading-icon="i-lucide-pause"
          color="neutral"
          variant="link"
          @click="emit('pause')"
        >
          Pause session
        </UButton>
        <UButton color="neutral" variant="subtle" @click="emit('close')">
          Return to session
        </UButton>
      </div>
    </template>
  </UModal>
</template>
