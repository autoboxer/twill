import { invoke } from '@tauri-apps/api/core';
import { readonly, ref } from 'vue';

const status = ref( null );

export function useStorageRecovery() {
  async function openStorage() {
    status.value = await invoke( 'retry_storage' );

    return status.value;
  }

  async function refreshStatus() {
    status.value = await invoke( 'get_storage_status' );

    return status.value;
  }

  return { openStorage, refreshStatus, status: readonly( status ) };
}
