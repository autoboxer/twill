<script setup>
import { computed, ref, watch } from 'vue';

import { useBackups } from '../composables/useBackups';

const props = defineProps({
  recovery: { type: Boolean, default: false }
});
const open = defineModel( 'open', { type: Boolean, default: false });
const emit = defineEmits([ 'prepared' ]);
const { chooseFile, clearError, error, inspectBackup, prepareRestore, run, working } = useBackups();
const preview = ref( null );
const source = ref( '' );
const filename = computed( () => source.value.split( /[/\\]/ ).at( -1 ) );
const createdAt = computed( () => preview.value
  ? new Date( preview.value.createdAt ).toLocaleString()
  : '' );
const counts = computed( () => Object.entries( preview.value?.counts ?? {}) );

watch( open, ( visible ) => {
  if ( !visible ) {
    return;
  }

  preview.value = null;
  source.value = '';
  clearError();
  void selectBackup();
});

async function selectBackup() {
  await run( 'Checking backup', async () => {
    const selected = await chooseFile( 'restore' );

    if ( !selected ) {
      open.value = false;
      return;
    }

    const inspected = await inspectBackup( selected );

    source.value = selected;
    preview.value = inspected;
  });
}

async function confirmRestore() {
  if ( !preview.value ) {
    return;
  }

  await run( 'Preparing restore', async () => {
    await prepareRestore( source.value, preview.value.fingerprint );
    open.value = false;
  });

  if ( !open.value ) {
    emit( 'prepared' );
  }
}
</script>

<template>
  <UModal
    v-model:open="open"
    title="Restore backup"
    :description="props.recovery
      ? 'Replace this device’s library, settings and saved drafts.'
      : 'Replace this device’s library, settings and saved drafts. Twill will restart and end the current study session.'"
    :dismissible="!working"
    :close="false"
  >
    <template #body>
      <div class="backup-preview">
        <p
          v-if="working"
          role="status"
        >
          {{ working }}…
        </p>

        <template v-if="preview">
          <p class="backup-preview__filename">{{ filename }}</p>
          <p>Created {{ createdAt }} · Twill {{ preview.appVersion }}</p>

          <dl class="backup-preview__counts">
            <div
              v-for="[label, count] in counts"
              :key="label"
            >
              <dt>{{ label === 'media' ? 'Images' : label }}</dt>
              <dd>{{ count.toLocaleString() }}</dd>
            </div>
          </dl>

          <p>
            {{ props.recovery
              ? 'This replaces the library on this device. The backup file is not changed.'
              : 'Your current library will be replaced. Create a backup first if you want to keep it.' }}
          </p>
        </template>

        <UAlert
          v-if="error"
          title="Backup could not be restored"
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
          variant="subtle"
          :disabled="Boolean( working )"
          @click="open = false"
        >
          Cancel
        </UButton>

        <UButton
          v-if="!preview"
          variant="subtle"
          :disabled="Boolean( working )"
          @click="selectBackup"
        >
          Choose backup
        </UButton>

        <UButton
          v-else
          color="warning"
          variant="subtle"
          :loading="working === 'Preparing restore'"
          :disabled="Boolean( working )"
          @click="confirmRestore"
        >
          {{ props.recovery ? 'Restore' : 'Restore and restart' }}
        </UButton>
      </div>
    </template>
  </UModal>
</template>
