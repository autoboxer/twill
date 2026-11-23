import { invoke } from '@tauri-apps/api/core';
import { ref } from 'vue';

// Keep failed writes available across route changes so a later reveal can retry them
const queued = [];
const pending = ref( false );
let flushing = null;

export function recordPracticeExposure( conceptId, cardId = null, contentVersion = null ) {
  return invoke( 'record_practice_exposure', { input: { conceptId, cardId, contentVersion } });
}

export function usePracticeExposure() {
  function remember( conceptId, cardId = null ) {
    if ( conceptId ) {
      queued.push({ conceptId, cardId });
    }

    return flush();
  }

  function flush() {
    if ( flushing ) {
      return flushing;
    }

    pending.value = queued.length > 0;
    flushing = ( async () => {
      while ( queued.length ) {
        const { conceptId, cardId } = queued[ 0 ];

        try {
          await recordPracticeExposure( conceptId, cardId );
        } catch ( cause ) {
          if ( cause?.code !== 'notFound' ) {
            throw cause;
          }
        }

        queued.shift();
      }
    })().finally( () => {
      pending.value = false;
      flushing = null;
    });

    return flushing;
  }

  return { remember, flush, pending };
}
