<script setup>
import { computed } from 'vue';

import { useNativeLifecycle } from '../composables/useNativeLifecycle';

const { completing, error, pending, request, retry, stay, working } = useNativeLifecycle();
const actionLabel = computed( () => request.value?.action === 'reload' ? 'reloading' : 'closing' );
</script>

<template>
  <UModal
    :open="pending"
    :dismissible="false"
    :close="false"
    :title="error ? 'Changes need attention' : `Saving before ${ actionLabel }`"
    :description="error
      ? 'Retry saving, or stay in Twill to keep editing.'
      : 'Saving unfinished work as a local draft.'"
  >
    <template #body>
      <UAlert
        v-if="error"
        :description="error"
        title="Twill has stayed open"
        icon="i-lucide-cloud-alert"
        color="error"
        variant="subtle"
      />

      <p v-else>
        Waiting for current edits and image imports to finish saving.
      </p>
    </template>

    <template #footer>
      <div class="dialog-actions">
        <UButton
          :disabled="completing"
          color="neutral"
          variant="subtle"
          @click="stay"
        >
          Stay in Twill
        </UButton>

        <UButton
          v-if="error"
          :loading="working"
          :disabled="completing"
          variant="subtle"
          @click="retry"
        >
          Retry
        </UButton>
      </div>
    </template>
  </UModal>
</template>
