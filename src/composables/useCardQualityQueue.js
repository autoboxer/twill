import { computed, onBeforeUnmount, ref } from 'vue';

import { cardQualityItemKey } from '../card-quality/presentation';
import { useCardQuality } from './useCardQuality';
import { conceptLibraryErrorMessage } from './useConceptLibrary';

export function useCardQualityQueue() {
  const { closeConcern, dismissSignal, getQueue } = useCardQuality();
  const items = ref([]);
  const loading = ref( true );
  const loadError = ref( '' );
  const actionError = ref( '' );
  const pendingKey = ref( '' );
  const notice = ref( '' );
  const needsRefresh = ref( false );
  let requestSequence = 0;

  const groups = computed( () => {
    const byCard = new Map();

    for ( const item of items.value ) {
      if ( !byCard.has( item.cardId ) ) {
        byCard.set( item.cardId, { card: item, items: [] });
      }

      byCard.get( item.cardId ).items.push( item );
    }

    return [ ...byCard.values() ];
  });
  const actionsDisabled = computed( () => (
    loading.value || Boolean( pendingKey.value ) || needsRefresh.value
  ) );

  onBeforeUnmount( () => {
    requestSequence += 1;
  });

  async function refresh() {
    if ( pendingKey.value ) {
      return;
    }

    const request = ++requestSequence;

    loading.value = true;
    loadError.value = '';

    try {
      const queue = await getQueue();

      if ( request === requestSequence ) {
        items.value = queue.items;
        actionError.value = '';
        needsRefresh.value = false;
      }
    } catch ( cause ) {
      if ( request === requestSequence ) {
        loadError.value = conceptLibraryErrorMessage( cause );
        needsRefresh.value = true;
      }
    } finally {
      if ( request === requestSequence ) {
        loading.value = false;
      }
    }
  }

  async function finishItem( item, disposition ) {
    if ( actionsDisabled.value ) {
      return;
    }

    const request = ++requestSequence;
    const key = cardQualityItemKey( item );

    pendingKey.value = key;
    actionError.value = '';
    notice.value = '';

    try {
      if ( item.source === 'manual' ) {
        await closeConcern( item.concernId, disposition );
      } else {
        await dismissSignal( item.cardId, item.evidence.observedThroughReviewId );
      }

      if ( request !== requestSequence ) {
        return;
      }

      items.value = items.value.filter( ( entry ) => cardQualityItemKey( entry ) !== key );
      notice.value = disposition === 'resolved' ? 'Issue resolved.' : 'Suggestion dismissed.';
      pendingKey.value = '';
      await refresh();
    } catch ( cause ) {
      if ( request === requestSequence ) {
        actionError.value = cause?.code === 'conflict' || cause?.code === 'notFound'
          ? 'This item has changed. Refresh the queue to review its current state before acting.'
          : conceptLibraryErrorMessage( cause );
        needsRefresh.value = cause?.code === 'conflict' || cause?.code === 'notFound';
      }
    } finally {
      if ( request === requestSequence ) {
        pendingKey.value = '';
      }
    }
  }

  return {
    actionError,
    actionsDisabled,
    finishItem,
    groups,
    loadError,
    loading,
    notice,
    pendingKey,
    refresh
  };
}
