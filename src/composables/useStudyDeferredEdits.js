import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { useRouter } from 'vue-router';

import { conceptLibraryErrorMessage } from './useConceptLibrary';
import { useDeferredEdits } from './useDeferredEdits';
import { useActionNotifications } from './useActionNotifications';
import { useNativeActionGuard } from './useNativeLifecycle';

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
    removeDeferredEdit,
    updateDeferredEditNote
  } = useDeferredEdits();
  const { notifySuccess } = useActionNotifications();

  const deferredEdits = ref([]);

  const deferredError = ref( '' );

  const deferredLoading = ref( true );

  const deferredPendingConceptId = ref( '' );

  const deferredStartPending = ref( false );
  const immediateEditError = ref( '' );
  const noteTarget = ref( null );
  const noteError = ref( '' );

  const notePending = computed( () => Boolean(
    noteTarget.value && deferredPendingConceptId.value === noteTarget.value.conceptId
  ) );

  useNativeActionGuard({
    busy: computed( () => Boolean( deferredPendingConceptId.value ) ),
    flush: () => {}
  });

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

  const canEditCurrentConcept = computed( () => (
    Boolean( currentCard.value )
    && !actionsBlocked.value
    && !deferredStartPending.value
    && !deferredPendingConceptId.value
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

      if ( viewActive ) {
        notifySuccess( 'Queued for editing' );
      }
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

  function openQueuedNote( item ) {
    if ( !item || sessionBusy.value || deferredPendingConceptId.value || deferredStartPending.value ) {
      return;
    }

    noteError.value = '';
    noteTarget.value = { ...item };
  }

  function editCurrentNote() {
    const item = deferredEdits.value.find( ( item ) => item.conceptId === currentCard.value?.conceptId );

    if ( item ) {
      openQueuedNote( item );
    }
  }

  function closeQueuedNote() {
    if ( !notePending.value ) {
      noteTarget.value = null;
      noteError.value = '';
    }
  }

  async function saveQueuedNote( note ) {
    const target = noteTarget.value;

    if ( !target || deferredPendingConceptId.value || deferredStartPending.value ) {
      return;
    }

    noteError.value = '';
    deferredPendingConceptId.value = target.conceptId;

    try {
      const updated = await updateDeferredEditNote( target, note );

      if ( viewActive ) {
        deferredEdits.value = deferredEdits.value.map( ( item ) => (
          item.conceptId === updated.conceptId ? updated : item
        ) );
        noteTarget.value = null;
        notifySuccess( 'Note saved' );
      }
    } catch ( cause ) {
      if ( viewActive ) {
        noteError.value = conceptLibraryErrorMessage( cause );

        if ( cause?.code === 'deferredEditChanged' ) {
          noteError.value = 'This queued edit changed. Close and reopen the note before saving.';
          await loadDeferredEditQueue();
        }
      }
    } finally {
      if ( viewActive ) {
        deferredPendingConceptId.value = '';
      }
    }
  }

  async function editCurrentConcept() {
    const card = currentCard.value;

    if ( !card || !canEditCurrentConcept.value ) {
      return;
    }

    immediateEditError.value = '';
    deferredStartPending.value = true;
    session.pauseSession();

    try {
      const failure = await router.push({
        name: 'concept-edit',
        params: { conceptId: card.conceptId },
        query: { study: '1' }
      });

      if ( failure && viewActive ) {
        throw new Error( 'Editing could not be started. Try again.' );
      }
    } catch ( cause ) {
      if ( viewActive ) {
        immediateEditError.value = conceptLibraryErrorMessage( cause );
        await session.resumeSession();
      }
    } finally {
      if ( viewActive ) {
        deferredStartPending.value = false;
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
    canEditCurrentConcept,
    canQueueCurrentConcept,
    currentConceptQueued,
    deferredEdits,
    deferredError,
    deferredLoading,
    deferredPendingConceptId,
    deferredStartPending,
    immediateEditError,
    noteError,
    notePending,
    noteTarget,
    closeQueuedNote,
    editCurrentConcept,
    editCurrentNote,
    loadDeferredEditQueue,
    queueCurrentConcept,
    openQueuedNote,
    removeQueuedConcept,
    saveQueuedNote,
    startDeferredEditing
  };
}
