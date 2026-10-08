import { invoke } from '@tauri-apps/api/core';
import { onBeforeUnmount, ref, toValue, watch } from 'vue';

import { conceptLibraryErrorMessage } from './useConceptLibrary';
import { useNativeActionGuard } from './useNativeLifecycle';

export function usePracticeLinks( conceptId ) {
  const links = ref([]);
  const loading = ref( true );
  const loadError = ref( '' );
  const actionError = ref( '' );
  const pending = ref( false );
  let requestSequence = 0;
  let disposed = false;

  useNativeActionGuard({ busy: pending, flush: () => {} });

  async function load() {
    const request = ++requestSequence;
    const id = toValue( conceptId );

    loading.value = true;
    loadError.value = '';

    try {
      const result = await invoke( 'get_practice_links', { conceptId: id });

      if ( request === requestSequence ) {
        links.value = result;
      }
    } catch ( cause ) {
      if ( request === requestSequence ) {
        loadError.value = conceptLibraryErrorMessage( cause );
      }
    } finally {
      if ( request === requestSequence ) {
        loading.value = false;
      }
    }
  }

  async function change( command, input ) {
    if ( pending.value || disposed ) {
      return false;
    }

    const id = toValue( conceptId );

    pending.value = true;
    actionError.value = '';

    try {
      await invoke( command, { input });

      if ( disposed || id !== toValue( conceptId ) ) {
        return false;
      }

      await load();

      return !disposed && id === toValue( conceptId );
    } catch ( cause ) {
      if ( !disposed && id === toValue( conceptId ) ) {
        actionError.value = conceptLibraryErrorMessage( cause );

        if ( cause.code === 'conflict' || cause.code === 'notFound' ) {
          await load();
        }
      }

      return false;
    } finally {
      pending.value = false;
    }
  }

  watch( () => toValue( conceptId ), () => {
    links.value = [];
    actionError.value = '';
    void load();
  }, { immediate: true, flush: 'sync' });

  onBeforeUnmount( () => {
    disposed = true;
    requestSequence += 1;
  });

  return {
    actionError,
    createLink: ( relatedConceptId, objective ) => change( 'create_practice_link', {
      conceptId: toValue( conceptId ), relatedConceptId, objective
    }),
    links,
    load,
    loadError,
    loading,
    pending,
    removeLink: ( link ) => change( 'remove_practice_link', {
      id: link.id, expectedChangeId: link.lastChangeId
    }),
    updateLink: ( link, objective ) => change( 'update_practice_link', {
      id: link.id, expectedChangeId: link.lastChangeId, objective
    })
  };
}
