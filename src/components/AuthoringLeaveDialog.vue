<script setup>
defineProps({
  open: Boolean,
  loading: Boolean,
  pendingImports: Boolean,
  action: { type: String, default: '' },
  error: { type: String, default: '' },
  title: { type: String, required: true }
});

const emit = defineEmits([ 'stay', 'keep', 'discard' ]);
</script>

<template>
  <UModal
    :open="open"
    :title="title"
    :description="pendingImports
      ? 'Wait for image imports to finish before leaving.'
      : 'Keep your draft to continue later, or discard your unfinished changes.'"
    :dismissible="!loading"
    :close="!loading"
    @update:open="( value ) => { if ( !value && !loading ) emit( 'stay' ) }"
  >
    <template v-if="error" #body>
      <UAlert
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
          @click="emit( 'stay' )"
        >
          Stay
        </UButton>

        <UButton
          color="error"
          variant="subtle"
          :disabled="loading || pendingImports"
          :loading="action === 'discard'"
          @click="emit( 'discard' )"
        >
          Discard and leave
        </UButton>

        <UButton
          variant="subtle"
          :disabled="loading || pendingImports"
          :loading="action === 'keep'"
          @click="emit( 'keep' )"
        >
          Leave and keep draft
        </UButton>
      </div>
    </template>
  </UModal>
</template>
