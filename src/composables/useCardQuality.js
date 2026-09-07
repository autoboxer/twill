import { invoke } from '@tauri-apps/api/core';
import { computed, ref } from 'vue';

import { conceptLibraryErrorMessage } from './useConceptLibrary';

export function useCardQuality() {
  const activeRequests = ref( 0 );
  const error = ref( '' );

  const isPending = computed( () => activeRequests.value > 0 );

  async function run( command, input ) {
    activeRequests.value += 1;
    error.value = '';

    try {
      return await invoke( command, input ? { input } : {});
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

  return {
    clearError,
    closeConcern: ( concernId, disposition ) => run(
      'close_card_quality_concern',
      { concernId, disposition }
    ),
    dismissSignal: ( cardId, observedThroughReviewId, signal = 'repeatedDifficulty' ) => run(
      'dismiss_card_quality_signal',
      { cardId, observedThroughReviewId, signal }
    ),
    error,
    flagCard: ( cardId, kind, note = '' ) => run(
      'create_card_quality_concern',
      { cardId, kind, note }
    ),
    getQueue: () => run( 'get_card_quality_queue' ),
    isPending
  };
}
