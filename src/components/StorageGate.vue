<script setup>
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue';

import AppShell from './AppShell.vue';
import StorageRecovery from './StorageRecovery.vue';
import { initializeAppearance } from '../composables/useAppearance';
import { useBackups } from '../composables/useBackups';
import { initializeCssSnippets } from '../composables/useCssSnippets';
import { useNativeActionGuard } from '../composables/useNativeLifecycle';
import { useStorageRecovery } from '../composables/useStorageRecovery';
import { completeStartup } from '../startup';

const { openStorage, refreshStatus, status } = useStorageRecovery();
const { working } = useBackups();
const opening = ref( false );
const ready = ref( false );
const error = ref( '' );
let progressTimer;
let active = true;

useNativeActionGuard({
  busy: computed( () => opening.value || Boolean( working.value ) ),
  flush: () => {}
});

onMounted( () => openLibrary( false ) );
onBeforeUnmount( () => {
  active = false;
  clearTimeout( progressTimer );
});

async function openLibrary( retry = true ) {
  if ( opening.value ) {
    return;
  }

  opening.value = true;
  error.value = '';
  progressTimer = setTimeout( completeStartup, 1500 );

  try {
    let current = await ( retry ? openStorage() : refreshStatus() );

    while ( active && !current.ready && !current.reason ) {
      await new Promise( ( resolve ) => setTimeout( resolve, 100 ) );
      current = await refreshStatus();
    }

    if ( !active ) {
      return;
    }

    if ( current.ready ) {
      await Promise.all([ initializeAppearance(), initializeCssSnippets() ]);
      ready.value = true;
    }
  } catch {
    error.value = 'Your library could not be opened. Retry, or restore a backup.';
  } finally {
    opening.value = false;
    clearTimeout( progressTimer );

    if ( !ready.value ) {
      await nextTick();
      completeStartup();
    }
  }
}
</script>

<template>
  <AppShell v-if="ready" />

  <StorageRecovery
    v-else
    :opening="opening"
    :reason="error || status?.reason || ''"
    @retry="openLibrary"
  />
</template>
