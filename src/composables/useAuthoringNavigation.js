import { computed, nextTick, onBeforeUnmount, onMounted, ref, toValue } from 'vue';
import { onBeforeRouteLeave, onBeforeRouteUpdate } from 'vue-router';

import { useNativeActionGuard } from './useNativeLifecycle';
import { useActionNotifications } from './useActionNotifications';

export function useAuthoringNavigation({
  discardDraft,
  editorResolved,
  flushDraft,
  hasPendingImports = false,
  hasPendingPersistence,
  isModified,
  recoveryBusy,
  recoveryOpen,
  saveInProgress
}) {
  const { notifySuccess } = useActionNotifications();
  const allowNavigation = ref( false );
  const leaveDialogOpen = ref( false );
  const leaveError = ref( '' );
  const leaveLoading = ref( false );
  const leaveAction = ref( '' );
  let leaveResolution = null;
  let leaveDecision = null;

  const hasUnsavedChanges = computed( () => (
    isModified.value
    || hasPendingPersistence.value
    || toValue( hasPendingImports )
  ) );
  const { nativeActionPending } = useNativeActionGuard({
    busy: computed( () => (
      saveInProgress.value
      || toValue( hasPendingImports )
      || recoveryBusy.value
      || leaveLoading.value
    ) ),
    flush: flushDraft,
    confirm: confirmNativeAction
  });

  onBeforeRouteLeave( protectNavigation );
  onBeforeRouteUpdate( protectNavigation );

  onMounted( () => {
    window.addEventListener( 'beforeunload', warnBeforeWindowClose );
    document.addEventListener( 'visibilitychange', flushHiddenDraft );
  });

  onBeforeUnmount( () => {
    window.removeEventListener( 'beforeunload', warnBeforeWindowClose );
    document.removeEventListener( 'visibilitychange', flushHiddenDraft );

    if ( leaveResolution ) {
      leaveResolution( false );
    }
  });

  function protectNavigation() {
    if ( nativeActionPending.value ) {
      return false;
    }

    if ( allowNavigation.value || !editorResolved.value ) {
      return recoveryOpen.value ? false : true;
    }

    if ( saveInProgress.value ) {
      return false;
    }

    if ( leaveLoading.value ) {
      return leaveDecision;
    }

    if ( !hasUnsavedChanges.value ) {
      return true;
    }

    return requestDecision();
  }

  function requestDecision() {
    if ( leaveResolution ) {
      leaveResolution( false );
    }

    leaveError.value = '';
    leaveDialogOpen.value = true;

    leaveDecision = new Promise( ( resolve ) => {
      leaveResolution = resolve;
    });

    return leaveDecision;
  }

  function stayInEditor() {
    if ( leaveLoading.value ) {
      return;
    }

    leaveDialogOpen.value = false;

    if ( leaveResolution ) {
      leaveResolution( false );
      leaveResolution = null;
    }
  }

  async function leaveEditor( action = 'keep' ) {
    if ( leaveLoading.value || toValue( hasPendingImports ) ) {
      return;
    }

    leaveLoading.value = true;
    leaveAction.value = action;
    leaveError.value = '';

    try {
      await nextTick();

      if ( action === 'discard' ) {
        await discardDraft();
      } else {
        await flushDraft();
      }
    } catch {
      leaveError.value = action === 'discard'
        ? 'The draft could not be discarded. Try again or stay in the editor.'
        : 'The draft could not be saved. Try again or stay in the editor.';
      leaveLoading.value = false;
      leaveAction.value = '';
      return;
    }

    leaveLoading.value = false;
    leaveAction.value = '';
    leaveDialogOpen.value = false;

    if ( leaveResolution ) {
      if ( action === 'keep' && isModified.value ) {
        notifySuccess( 'Draft saved' );
      }

      leaveResolution( true );
      leaveResolution = null;
    }
  }

  async function confirmNativeAction( signal ) {
    if ( !editorResolved.value || !hasUnsavedChanges.value ) {
      await flushDraft();
      return true;
    }

    const decision = requestDecision();
    const cancel = () => stayInEditor();

    signal.addEventListener( 'abort', cancel, { once: true });

    try {
      return await decision;
    } finally {
      signal.removeEventListener( 'abort', cancel );
    }
  }

  function warnBeforeWindowClose( event ) {
    if ( nativeActionPending.value ) {
      return;
    }

    if ( !hasUnsavedChanges.value ) {
      return;
    }

    event.preventDefault();
    event.returnValue = '';
  }

  function flushHiddenDraft() {
    if ( document.visibilityState === 'hidden' && isModified.value && !leaveLoading.value ) {
      void flushDraft().catch( () => undefined );
    }
  }

  return {
    allowNavigation,
    leaveAction,
    leaveDialogOpen,
    leaveEditor,
    leaveError,
    leaveLoading,
    stayInEditor
  };
}
