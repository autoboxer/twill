import { invoke } from '@tauri-apps/api/core';
import { readonly, ref } from 'vue';

const working = ref( '' );
const error = ref( '' );

export function useBackups() {
  async function run( label, operation ) {
    if ( working.value ) {
      return;
    }

    working.value = label;
    error.value = '';

    try {
      await operation();
    } catch ( cause ) {
      error.value = typeof cause === 'string' ? cause : cause?.message
        || 'The operation could not finish. Try again.';
    } finally {
      working.value = '';
    }
  }

  return {
    cancelRestore: () => invoke( 'cancel_restore' ),
    chooseFile: ( action ) => invoke( 'choose_archive_file', { action }),
    clearError: () => {
      error.value = '';
    },
    createBackup: ( destination ) => invoke( 'create_backup', { destination }),
    error: readonly( error ),
    exportLibrary: ( destination ) => invoke( 'export_library', { destination }),
    inspectBackup: ( source ) => invoke( 'inspect_backup', { source }),
    prepareRestore: ( source, fingerprint ) => invoke( 'prepare_restore', { source, fingerprint }),
    restart: () => invoke( 'restart_application' ),
    run,
    working: readonly( working )
  };
}
