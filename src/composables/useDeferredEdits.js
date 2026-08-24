import { invoke } from '@tauri-apps/api/core';
import { computed, ref } from 'vue';

import { conceptLibraryErrorMessage } from './useConceptLibrary';

export function useDeferredEdits() {
  const activeRequests = ref( 0 );
  const error = ref( '' );

  const isPending = computed( () => activeRequests.value > 0 );

  async function run( command, arguments_ = {}) {
    activeRequests.value += 1;
    error.value = '';

    try {
      return await invoke( command, arguments_ );
    } catch ( cause ) {
      error.value = conceptLibraryErrorMessage( cause );
      throw cause;
    } finally {
      activeRequests.value -= 1;
    }
  }

  return {
    clearError: () => {
      error.value = '';
    },
    error,
    getDeferredEdits: () => run( 'get_deferred_edits' ),
    isPending,
    queueDeferredEdit: ( conceptId, baseChangeId ) => run(
      'queue_deferred_edit',
      { input: { conceptId, baseChangeId } }
    ),
    removeDeferredEdit: ( conceptId ) => run( 'remove_deferred_edit', {
      input: { id: conceptId }
    })
  };
}
