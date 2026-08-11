import { invoke } from '@tauri-apps/api/core';
import { computed, ref } from 'vue';

import { conceptLibraryErrorMessage } from './useConceptLibrary';

export function useTemplateLibrary() {
  const activeRequests = ref( 0 );
  const error = ref( '' );
  let requestSequence = 0;

  const isPending = computed( () => activeRequests.value > 0 );

  async function run( command, arguments_ = {}) {
    const request = ++requestSequence;

    activeRequests.value += 1;
    error.value = '';

    try {
      return await invoke( command, arguments_ );
    } catch ( cause ) {
      if ( request === requestSequence ) {
        error.value = conceptLibraryErrorMessage( cause );
      }

      throw cause;
    } finally {
      activeRequests.value -= 1;
    }
  }

  function clearError() {
    requestSequence += 1;
    error.value = '';
  }

  return {
    clearError,
    createTemplate: ( input ) => run( 'create_template', { input }),
    deleteTemplate: ( id ) => run( 'delete_template', { input: { id } }),
    error,
    getTemplate: ( templateId ) => run( 'get_template', { templateId }),
    getTemplates: () => run( 'get_templates' ),
    isPending,
    preparePreview: ( content ) => run( 'prepare_template_preview', { content }),
    updateTemplate: ( input ) => run( 'update_template', { input })
  };
}
