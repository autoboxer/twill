import { invoke } from '@tauri-apps/api/core';
import { computed, ref } from 'vue';

import { resolveStudyQueue } from '../study/queue';
import { markStudyConceptChanged } from '../study/resume';

export function conceptLibraryErrorMessage( error ) {
  if ( typeof error === 'string' ) {
    return error;
  }

  if ( error && typeof error.message === 'string' ) {
    return error.message;
  }

  return 'Local data could not be accessed.';
}

export function useConceptLibrary() {
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

  function clearError() {
    error.value = '';
  }

  async function changeConcept( command, input ) {
    const result = await run( command, { input });

    markStudyConceptChanged( result?.id ?? input.id );

    return result;
  }

  return {
    clearError,
    createConcept: ( input ) => run( 'create_concept', { input }),
    createDeck: ( name ) => run( 'create_deck', { input: { name } }),
    createTag: ( name ) => run( 'create_tag', { input: { name } }),
    deleteConcept: ( id ) => changeConcept( 'delete_concept', { id }),
    deleteDeck: ( id ) => run( 'delete_deck', { input: { id } }),
    deleteTag: ( id ) => run( 'delete_tag', { input: { id } }),
    error,
    finalizeConcept: ( input ) => changeConcept( 'finalize_concept', input ),
    getConcept: ( conceptId ) => run( 'get_concept', { conceptId }),
    getLibrary: ( input = {}) => run( 'get_library', { input }),
    getLibraryOrganizations: () => run( 'get_library_organizations' ),
    getStudyQueue: async ( input = {}) => resolveStudyQueue( await run( 'get_study_queue', { input }) ),
    isPending,
    readMedia: ( mediaId ) => run( 'read_media', { mediaId }),
    recordPretest: ( cardId, outcome ) => run( 'record_pretest', {
      input: { cardId, outcome }
    }),
    recordReview: ( cardId, rating ) => run( 'record_review', {
      input: { cardId, rating }
    }),
    reverseReview: ( reviewId ) => run( 'reverse_review', {
      input: { reviewId }
    }),
    renameDeck: ( id, name ) => run( 'rename_deck', { input: { id, name } }),
    renameTag: ( id, name ) => run( 'rename_tag', { input: { id, name } }),
    setConceptArchived: ( id, archived ) => changeConcept( 'set_concept_archived', { id, archived }),
    updateConcept: ( input ) => changeConcept( 'update_concept', input )
  };
}
