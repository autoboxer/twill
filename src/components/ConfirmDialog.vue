<script setup>
import { computed } from 'vue';

const props = defineProps({
  confirmLabel: {
    type: String,
    default: 'Confirm'
  },
  description: {
    type: String,
    required: true
  },
  error: {
    type: String,
    default: ''
  },
  loading: {
    type: Boolean,
    default: false
  },
  open: {
    type: Boolean,
    required: true
  },
  title: {
    type: String,
    required: true
  }
});

const emit = defineEmits([ 'confirm', 'update:open' ]);

const isOpen = computed({
  get: () => props.open,
  set: ( value ) => emit( 'update:open', value )
});
</script>

<template>
  <UModal
    v-model:open="isOpen"
    :title="title"
    :description="description"
    :dismissible="!loading"
    :ui="{ wrapper: 'min-w-0', description: 'break-words' }"
  >
    <template v-if="error" #body>
      <UAlert
        role="alert"
        :description="error"
        icon="i-lucide-circle-alert"
        color="error"
        variant="subtle"
      />
    </template>

    <template #footer>
      <div class="dialog-actions">
        <UButton
          color="neutral"
          variant="link"
          :disabled="loading"
          @click="isOpen = false"
        >
          Cancel
        </UButton>

        <UButton
          color="error"
          leading-icon="i-lucide-trash-2"
          :loading="loading"
          @click="emit( 'confirm' )"
        >
          {{ confirmLabel }}
        </UButton>
      </div>
    </template>
  </UModal>
</template>
