<script setup>
import { ref } from 'vue';

defineProps({
  busy: { type: Boolean, default: false },
  complete: { type: Boolean, default: false },
  paused: { type: Boolean, default: false }
});

const emit = defineEmits([ 'pause', 'resume', 'end' ]);
const endOpen = ref( false );
</script>

<template>
  <div class="study-session-controls">
    <UButton
      v-if="!complete"
      :leading-icon="paused ? 'i-lucide-play' : 'i-lucide-pause'"
      color="neutral"
      variant="subtle"
      :disabled="busy"
      @click="emit(paused ? 'resume' : 'pause')"
    >
      {{ paused ? 'Resume session' : 'Pause session' }}
    </UButton>

    <UButton
      color="neutral"
      variant="link"
      :disabled="busy"
      @click="endOpen = true"
    >
      End session
    </UButton>

    <UModal
      v-model:open="endOpen"
      title="End this session?"
      description="Unfinished responses and session progress will be discarded. Completed reviews and pretests remain saved."
      :ui="{ overlay: 'z-70', content: 'z-71 rounded-md' }"
    >
      <template #footer>
        <div class="dialog-actions">
          <UButton color="neutral" variant="subtle" @click="endOpen = false">
            Cancel
          </UButton>
          <UButton
            color="error"
            variant="subtle"
            :disabled="busy"
            @click="emit('end'); endOpen = false"
          >
            End session
          </UButton>
        </div>
      </template>
    </UModal>
  </div>
</template>

<style scoped>
.study-session-controls {
  display: flex;
  width: min(100%, 60rem);
  flex-wrap: wrap;
  justify-content: flex-end;
  margin: 0 auto 1rem;
  gap: 0.75rem;
}
</style>
