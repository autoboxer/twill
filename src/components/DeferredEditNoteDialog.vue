<script setup>
import { computed, ref, watch } from 'vue';

const props = defineProps({
  target: { type: Object, default: null },
  pending: Boolean,
  error: { type: String, default: '' }
});
const emit = defineEmits([ 'close', 'save' ]);
const note = ref( '' );
const noteLength = computed( () => Array.from( note.value.trim() ).length );
const canSave = computed( () => (
  !props.pending
  && noteLength.value <= 500
  && note.value.trim() !== props.target?.note
) );

watch( () => props.target, ( target ) => {
  note.value = target?.note ?? '';
}, { immediate: true });
</script>

<template>
  <UModal
    :open="Boolean( target )"
    title="Queued edit note"
    :description="target?.conceptTitle"
    :dismissible="!pending"
    :close="!pending"
    @update:open="( open ) => { if ( !open && !pending ) emit( 'close' ) }"
  >
    <template #body>
      <div class="deferred-edit-note">
        <UFormField
          label="Note (optional)"
          name="note"
          :error="noteLength > 500 ? 'Use 500 characters or fewer.' : undefined"
        >
          <UTextarea
            v-model="note"
            :rows="3"
            :disabled="pending"
            class="w-full"
          />
        </UFormField>

        <span class="deferred-edit-note__length">{{ noteLength }} / 500</span>

        <UAlert
          v-if="error"
          :description="error"
          color="error"
          variant="subtle"
          icon="i-lucide-circle-alert"
        />
      </div>
    </template>

    <template #footer>
      <div class="dialog-actions">
        <UButton
          color="neutral"
          variant="link"
          :disabled="pending"
          @click="emit( 'close' )"
        >
          Cancel
        </UButton>
        <UButton
          variant="subtle"
          :loading="pending"
          :disabled="!canSave"
          @click="emit( 'save', note )"
        >
          Save note
        </UButton>
      </div>
    </template>
  </UModal>
</template>
