import { invoke } from '@tauri-apps/api/core';
import { onBeforeUnmount, ref, watch } from 'vue';

import { conceptLibraryErrorMessage } from './useConceptLibrary';

export function useCaseComparisonPractice({ sourceCard, onExposed }) {
  const active = ref( null );
  const loading = ref( false );
  const error = ref( '' );
  let sequence = 0;
  let disposed = false;

  function reset() {
    sequence += 1;
    active.value = null;
    loading.value = false;
    error.value = '';
  }

  function fetchComparison( conceptId, linkId ) {
    return invoke( 'get_practice_comparison', { input: { conceptId, linkId } });
  }

  async function start( selected ) {
    const source = sourceCard.value;
    const token = ++sequence;

    loading.value = true;
    error.value = '';

    try {
      const activity = await fetchComparison( source.conceptId, selected.link.id );

      if ( disposed || token !== sequence || sourceCard.value?.id !== source.id ) {
        return false;
      }

      if ( activity.source.lastChangeId !== source.conceptLastChangeId ) {
        error.value = 'The current concept changed. Start a new session before comparing cases.';
        return false;
      }

      if ( activity.link.lastChangeId !== selected.link.lastChangeId
        || activity.related.lastChangeId !== selected.concept.lastChangeId ) {
        error.value = 'This comparison changed. Choose the case again.';
        return false;
      }

      active.value = { activity, response: '', revealed: false };

      for ( const concept of [ activity.source, activity.related ]) {
        if ( concept.cards.some( ( card ) => [ 'cloze', 'imageOcclusion' ].includes( card.retrievalKind ) ) ) {
          onExposed( concept.id );
        }
      }

      return true;
    } catch ( cause ) {
      if ( !disposed && token === sequence ) {
        error.value = conceptLibraryErrorMessage( cause );
      }

      return false;
    } finally {
      if ( token === sequence ) {
        loading.value = false;
      }
    }
  }

  async function validate() {
    const saved = active.value?.activity;

    if ( !saved ) {
      return { valid: true, sourceChanged: false };
    }

    const token = ++sequence;
    loading.value = true;

    try {
      const current = await fetchComparison( saved.source.id, saved.link.id );

      return {
        valid: !disposed && token === sequence
          && current.source.lastChangeId === saved.source.lastChangeId
          && current.related.lastChangeId === saved.related.lastChangeId
          && current.link.lastChangeId === saved.link.lastChangeId,
        sourceChanged: current.source.lastChangeId !== saved.source.lastChangeId
      };
    } catch {
      try {
        const source = await invoke( 'get_concept', { conceptId: saved.source.id });

        return { valid: false, sourceChanged: source.archived || source.lastChangeId !== saved.source.lastChangeId };
      } catch ( cause ) {
        return { valid: false, sourceChanged: cause?.code === 'notFound' };
      }
    } finally {
      if ( token === sequence ) {
        loading.value = false;
      }
    }
  }

  function restore( saved ) {
    reset();
    active.value = saved ? { ...saved } : null;
  }

  watch( () => sourceCard.value?.id, reset, { flush: 'sync' });
  watch( () => active.value?.revealed, ( revealed ) => {
    if ( revealed ) {
      onExposed( active.value.activity.source.id );
      onExposed( active.value.activity.related.id );
    }
  }, { flush: 'sync' });

  onBeforeUnmount( () => {
    disposed = true;
    sequence += 1;
  });

  return { active, error, loading, reset, restore, start, validate };
}
