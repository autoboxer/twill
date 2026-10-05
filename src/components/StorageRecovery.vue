<script setup>
import { ref } from 'vue';

import RestoreBackupDialog from './RestoreBackupDialog.vue';
import { useActionNotifications } from '../composables/useActionNotifications';
import { useBackups } from '../composables/useBackups';
import { useStorageRecovery } from '../composables/useStorageRecovery';

defineProps({
  opening: { type: Boolean, default: false },
  reason: { type: String, default: '' }
});
const emit = defineEmits([ 'retry' ]);
const { cancelRestore, error, run, working } = useBackups();
const { refreshStatus, status } = useStorageRecovery();
const { notifySuccess } = useActionNotifications();
const restoreOpen = ref( false );
const diagnosticsText = ref( '' );

async function cancelPendingRestore() {
  await run( 'Cancelling restore', async () => {
    await cancelRestore();
    await refreshStatus();
  });
}

async function copyDiagnostics() {
  await run( 'Copying diagnostics', async () => {
    const current = await refreshStatus();

    const text = JSON.stringify({
      app: 'Twill',
      ready: current.ready,
      restorePending: current.restorePending,
      diagnostics: current.diagnostics
    }, null, 2 );

    try {
      await navigator.clipboard.writeText( text );
      diagnosticsText.value = '';
      notifySuccess( 'Diagnostics copied' );
    } catch {
      diagnosticsText.value = text;
    }
  });
}
</script>

<template>
  <main
    id="main-content"
    class="storage-recovery"
    data-twill-page="storage-recovery"
    tabindex="-1"
    :aria-busy="opening || Boolean( working )"
  >
    <section class="storage-recovery__content">
      <UIcon
        :name="opening ? 'i-lucide-loader-circle' : 'i-lucide-database'"
        class="storage-recovery__icon"
        :class="{ 'animate-spin': opening }"
        aria-hidden="true"
      />

      <h1>{{ opening ? 'Opening your library' : 'Your library needs attention' }}</h1>

      <p
        v-if="opening"
        role="status"
      >
        Please wait while Twill opens your library.
      </p>

      <p v-else>{{ reason || 'Retry opening your library, or restore a backup.' }}</p>

      <p
        v-if="status?.restorePending"
        role="status"
      >
        A backup is ready to restore. Retry to open it, or cancel the restore.
      </p>

      <div class="backup-actions">
        <UButton
          variant="subtle"
          :disabled="opening || Boolean( working )"
          @click="emit( 'retry' )"
        >
          Retry
        </UButton>

        <UButton
          color="neutral"
          variant="subtle"
          :disabled="opening || Boolean( working ) || status?.restorePending"
          @click="restoreOpen = true"
        >
          Restore backup
        </UButton>

        <UButton
          v-if="status?.restorePending"
          color="neutral"
          variant="ghost"
          :disabled="opening || Boolean( working )"
          @click="cancelPendingRestore"
        >
          Cancel restore
        </UButton>
      </div>

      <p
        v-if="working && !restoreOpen"
        role="status"
      >
        {{ working }}…
      </p>

      <UAlert
        v-if="error && !restoreOpen"
        title="Operation could not finish"
        :description="error"
        color="error"
        variant="subtle"
        icon="i-lucide-circle-alert"
      />

      <UButton
        v-if="status?.diagnostics.length"
        class="storage-recovery__diagnostics"
        color="neutral"
        variant="link"
        :disabled="opening || Boolean( working )"
        @click="copyDiagnostics"
      >
        Copy diagnostics
      </UButton>

      <UFormField
        v-if="diagnosticsText"
        label="Diagnostics"
        description="Select and copy this text."
      >
        <UTextarea
          :model-value="diagnosticsText"
          aria-label="Storage diagnostics"
          readonly
          :rows="6"
          class="w-full"
        />
      </UFormField>
    </section>

    <RestoreBackupDialog
      v-model:open="restoreOpen"
      recovery
      @prepared="emit( 'retry' )"
    />
  </main>
</template>
