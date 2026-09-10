import { computed, onBeforeUnmount, onMounted, ref, toValue } from 'vue';
import { onBeforeRouteLeave, onBeforeRouteUpdate } from 'vue-router';

import { useNativeActionGuard } from './useNativeLifecycle';

export function useAuthoringNavigation({
  editorResolved,
  flushDraft,
  hasPendingImports = false,
  hasPendingPersistence,
  isModified,
  recoveryBusy,
  recoveryOpen,
  saveInProgress
}) {
  const allowNavigation = ref( false );
  const leaveDialogOpen = ref( false );
  const leaveError = ref( '' );
  const leaveLoading = ref( false );
  let leaveResolution = null;

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
    flush: flushDraft
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

    if ( !hasUnsavedChanges.value ) {
      return true;
    }

    if ( leaveResolution ) {
      leaveResolution( false );
    }

    leaveError.value = '';
    leaveDialogOpen.value = true;

    return new Promise( ( resolve ) => {
      leaveResolution = resolve;
    });
  }

  function stayInEditor() {
    leaveDialogOpen.value = false;

    if ( leaveResolution ) {
      leaveResolution( false );
      leaveResolution = null;
    }
  }

  async function leaveEditor() {
    if ( leaveLoading.value || toValue( hasPendingImports ) ) {
      return;
    }

    leaveLoading.value = true;
    leaveError.value = '';

    try {
      await flushDraft();
    } catch {
      leaveError.value = 'The latest changes could not be saved. Retry or stay in the editor.';
      leaveLoading.value = false;
      return;
    }

    leaveLoading.value = false;
    leaveDialogOpen.value = false;

    if ( leaveResolution ) {
      leaveResolution( true );
      leaveResolution = null;
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
    if ( document.visibilityState === 'hidden' && isModified.value ) {
      void flushDraft().catch( () => undefined );
    }
  }

  return {
    allowNavigation,
    leaveDialogOpen,
    leaveEditor,
    leaveError,
    leaveLoading,
    stayInEditor
  };
}
