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
    <UTooltip v-if="!complete" :text="paused ? 'Resume session' : 'Pause session'">
      <UButton
        :leading-icon="paused ? 'i-lucide-play' : 'i-lucide-pause'"
        :aria-label="paused ? 'Resume session' : 'Pause session'"
        color="neutral"
        variant="ghost"
        size="sm"
        :disabled="busy"
        @click="emit( paused ? 'resume' : 'pause' )"
      />
    </UTooltip>

    <UButton
      color="neutral"
      variant="link"
      size="sm"
      :disabled="busy"
      @click="endOpen = true"
    >
      End session
    </UButton>

    <UModal
      v-model:open="endOpen"
      title="End this session?"
      description="Unfinished responses and session progress will be discarded."
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
  align-items: center;
  gap: 0.25rem;
}
</style>
