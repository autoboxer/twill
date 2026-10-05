<script setup>
import { onMounted, ref } from 'vue';
import { onBeforeRouteLeave } from 'vue-router';

import RestoreBackupDialog from '../RestoreBackupDialog.vue';
import { useActionNotifications } from '../../composables/useActionNotifications';
import { useBackups } from '../../composables/useBackups';
import { useStorageRecovery } from '../../composables/useStorageRecovery';

const { notifySuccess } = useActionNotifications();
const { cancelRestore, chooseFile, createBackup, error, exportLibrary, restart, run, working } = useBackups();
const { refreshStatus, status } = useStorageRecovery();
const restoreOpen = ref( false );

onMounted( () => run( 'Checking backups', refreshStatus ) );
onBeforeRouteLeave( () => !working.value );

async function saveArchive( kind ) {
  await run( kind === 'backup' ? 'Creating backup' : 'Exporting data', async () => {
    const destination = await chooseFile( kind );

    if ( !destination ) {
      return;
    }

    if ( kind === 'backup' ) {
      await createBackup( destination );
      notifySuccess( 'Backup created' );
    } else {
      await exportLibrary( destination );
      notifySuccess( 'Data exported' );
    }
  });
}

async function finishRestore() {
  await run( 'Restarting Twill', async () => {
    await refreshStatus();
    await restart();
  });
}

async function cancelPendingRestore() {
  await run( 'Cancelling restore', async () => {
    await cancelRestore();
    await refreshStatus();
    notifySuccess( 'Restore cancelled' );
  });
}
</script>

<template>
  <section
    id="settings-backups"
    class="settings-panel"
    data-twill-settings-section="backups"
    :aria-busy="Boolean( working )"
  >
    <header class="settings-panel__header">
      <span
        class="settings-panel__icon"
        aria-hidden="true"
      >
        <UIcon name="i-lucide-archive" />
      </span>

      <div>
        <h2>Backups and export</h2>
        <p>These files are not encrypted. Store them somewhere private.</p>
      </div>
    </header>

    <div class="settings-preference-row">
      <div>
        <strong>Backup</strong>
        <p>Save your library, images, settings, drafts and learning history.</p>
      </div>

      <UButton
        variant="subtle"
        leading-icon="i-lucide-archive"
        :disabled="Boolean( working )"
        @click="saveArchive( 'backup' )"
      >
        Create backup
      </UButton>
    </div>

    <div class="settings-preference-row">
      <div>
        <strong>Export</strong>
        <p>Readable JSON and images in a ZIP file. Use a backup to restore Twill.</p>
      </div>

      <UButton
        color="neutral"
        variant="subtle"
        leading-icon="i-lucide-download"
        :disabled="Boolean( working )"
        @click="saveArchive( 'export' )"
      >
        Export data
      </UButton>
    </div>

    <div class="settings-preference-row">
      <div>
        <strong>Restore</strong>
        <p>Replace this device’s library from a Twill backup.</p>
      </div>

      <UButton
        color="neutral"
        variant="subtle"
        leading-icon="i-lucide-history"
        :disabled="Boolean( working ) || status?.restorePending"
        @click="restoreOpen = true"
      >
        Restore backup
      </UButton>
    </div>

    <div
      v-if="status?.restorePending"
      class="backup-pending"
    >
      <p role="status">A backup is ready to restore when Twill restarts.</p>

      <div class="backup-actions">
        <UButton
          variant="subtle"
          :disabled="Boolean( working )"
          @click="finishRestore"
        >
          Restart Twill
        </UButton>

        <UButton
          color="neutral"
          variant="ghost"
          :disabled="Boolean( working )"
          @click="cancelPendingRestore"
        >
          Cancel restore
        </UButton>
      </div>
    </div>

    <p
      v-if="working && !restoreOpen"
      role="status"
      class="settings-save-status"
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

    <RestoreBackupDialog
      v-model:open="restoreOpen"
      @prepared="finishRestore"
    />
  </section>
</template>
