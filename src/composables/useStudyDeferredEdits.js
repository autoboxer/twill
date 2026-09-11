import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { useRouter } from 'vue-router';

import { conceptLibraryErrorMessage } from './useConceptLibrary';
import { useDeferredEdits } from './useDeferredEdits';

export function useStudyDeferredEdits( session ) {
  const {
    actionsBlocked,
    currentCard,
    sessionBusy,
    pretestPending
  } = session;
  const router = useRouter();
  const {
    getDeferredEdits,
    queueDeferredEdit,
    removeDeferredEdit
  } = useDeferredEdits();

  const deferredEdits = ref([]);

  const deferredError = ref( '' );

  const deferredLoading = ref( true );

  const deferredPendingConceptId = ref( '' );

  const deferredStartPending = ref( false );

  let deferredRequestSequence = 0;

  let viewActive = true;

  const queuedConceptIds = computed( () => new Set(
    deferredEdits.value.map( ( item ) => item.conceptId )
  ) );

  const currentConceptQueued = computed( () => (
    queuedConceptIds.value.has( currentCard.value?.conceptId )
  ) );

  const canQueueCurrentConcept = computed( () => (
    Boolean( currentCard.value )
    && !actionsBlocked.value
    && !currentConceptQueued.value
    && !deferredLoading.value
    && !deferredPendingConceptId.value
    && !deferredStartPending.value
    && !pretestPending.value
  ) );

  onMounted( loadDeferredEditQueue );

  onBeforeUnmount( () => {
    viewActive = false;
    deferredRequestSequence += 1;
  });

  async function loadDeferredEditQueue() {
    const request = ++deferredRequestSequence;

    deferredError.value = '';
    deferredLoading.value = true;

    try {
      const queue = await getDeferredEdits();

      if ( request === deferredRequestSequence && viewActive ) {
        deferredEdits.value = queue.items;
      }
    } catch ( cause ) {
      if ( request === deferredRequestSequence && viewActive ) {
        deferredError.value = conceptLibraryErrorMessage( cause );
      }
    } finally {
      if ( request === deferredRequestSequence && viewActive ) {
        deferredLoading.value = false;
      }
    }
  }

  async function queueCurrentConcept() {
    const card = currentCard.value;

    if ( !card || !canQueueCurrentConcept.value ) {
      return;
    }

    deferredError.value = '';
    deferredPendingConceptId.value = card.conceptId;

    try {
      await queueDeferredEdit( card.conceptId, card.conceptLastChangeId );
      await loadDeferredEditQueue();
    } catch ( cause ) {
      if ( viewActive ) {
        deferredError.value = conceptLibraryErrorMessage( cause );
      }
    } finally {
      if ( viewActive ) {
        deferredPendingConceptId.value = '';
      }
    }
  }

  async function removeQueuedConcept( conceptId ) {
    if ( deferredPendingConceptId.value || deferredStartPending.value ) {
      return;
    }

    deferredError.value = '';
    deferredPendingConceptId.value = conceptId;

    try {
      await removeDeferredEdit( conceptId );
      await loadDeferredEditQueue();
    } catch ( cause ) {
      if ( viewActive ) {
        deferredError.value = conceptLibraryErrorMessage( cause );
      }
    } finally {
      if ( viewActive ) {
        deferredPendingConceptId.value = '';
      }
    }
  }

  async function startDeferredEditing() {
    const firstItem = deferredEdits.value[ 0 ];

    if (
      deferredStartPending.value
      || sessionBusy.value
      || deferredPendingConceptId.value
      || firstItem?.targetStatus !== 'current'
    ) {
      return;
    }

    deferredStartPending.value = true;
    deferredError.value = '';

    try {
      await router.push({
        name: 'concept-edit',
        params: { conceptId: firstItem.conceptId },
        query: { deferred: '1' }
      });
    } catch ( cause ) {
      if ( viewActive ) {
        deferredError.value = cause.message || 'Queued editing could not be started.';
      }
    } finally {
      if ( viewActive ) {
        deferredStartPending.value = false;
      }
    }
  }

  return {
    canQueueCurrentConcept,
    currentConceptQueued,
    deferredEdits,
    deferredError,
    deferredLoading,
    deferredPendingConceptId,
    deferredStartPending,
    loadDeferredEditQueue,
    queueCurrentConcept,
    removeQueuedConcept,
    startDeferredEditing
  };
}
