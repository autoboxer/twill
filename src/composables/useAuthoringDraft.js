import { invoke } from '@tauri-apps/api/core';
import { computed, ref } from 'vue';

const AUTHORING_DRAFT_SCHEMA_VERSION = 1;
const DEFAULT_AUTOSAVE_DELAY = 600;

export function useAuthoringDraft( kind, autosaveDelay = DEFAULT_AUTOSAVE_DELAY ) {
  const draft = ref( null );
  const error = ref( '' );
  const status = ref( 'untouched' );

  let context = null;
  let pendingOperation = null;
  let persistenceWorker = null;
  let timer = null;

  const hasPendingPersistence = computed( () => (
    status.value === 'dirty'
    || status.value === 'saving'
    || status.value === 'error'
  ) );

  async function load( targetId = null ) {
    return invoke( 'get_authoring_draft', {
      input: {
        kind,
        targetId
      }
    });
  }

  function start({ targetId = null, baseChangeId = null }, existingDraft = null ) {
    clearTimer();
    pendingOperation = null;
    context = {
      baseChangeId,
      targetId
    };
    draft.value = existingDraft;
    error.value = '';
    status.value = existingDraft ? 'restorable' : 'untouched';
  }

  function scheduleSave( payload, mediaIds = []) {
    ensureStarted();

    pendingOperation = {
      kind: 'save',
      input: {
        kind,
        targetId: context.targetId,
        schemaVersion: AUTHORING_DRAFT_SCHEMA_VERSION,
        baseChangeId: context.baseChangeId,
        payload: structuredClone( payload ),
        mediaIds: [ ...new Set( mediaIds ) ]
      }
    };
    error.value = '';
    status.value = 'dirty';
    schedulePersistence();
  }

  function scheduleDelete() {
    ensureStarted();

    pendingOperation = {
      kind: 'delete',
      input: locator()
    };
    error.value = '';
    status.value = 'dirty';
    schedulePersistence();
  }

  async function flush() {
    clearTimer();

    if ( persistenceWorker ) {
      await persistenceWorker;
    }

    if ( pendingOperation ) {
      await runPersistenceWorker();
    }

    if ( status.value === 'error' ) {
      throw new Error( error.value );
    }
  }

  async function discard() {
    ensureStarted();
    clearTimer();

    pendingOperation = {
      kind: 'delete',
      input: locator()
    };
    error.value = '';
    status.value = 'dirty';

    if ( persistenceWorker ) {
      await persistenceWorker;
    }

    if ( pendingOperation ) {
      await runPersistenceWorker();
    }

    if ( status.value === 'error' ) {
      throw new Error( error.value );
    }
  }

  async function refresh() {
    ensureStarted();

    try {
      const currentDraft = await load( context.targetId );

      draft.value = currentDraft;
      error.value = '';
      status.value = currentDraft ? 'saved' : 'untouched';

      return currentDraft;
    } catch ( cause ) {
      error.value = draftErrorMessage( cause );
      status.value = 'error';

      throw cause;
    }
  }

  function retry() {
    if ( pendingOperation ) {
      error.value = '';
      status.value = 'dirty';
      void runPersistenceWorker();

      return;
    }

    if ( context ) {
      void refresh().catch( () => undefined );
    }
  }

  function schedulePersistence() {
    clearTimer();
    timer = window.setTimeout( () => {
      timer = null;
      void runPersistenceWorker();
    }, autosaveDelay );
  }

  async function runPersistenceWorker() {
    if ( persistenceWorker ) {
      return persistenceWorker;
    }

    persistenceWorker = persistPendingOperations();

    try {
      await persistenceWorker;
    } finally {
      persistenceWorker = null;
    }
  }

  async function persistPendingOperations() {
    while ( pendingOperation ) {
      const operation = pendingOperation;

      pendingOperation = null;
      error.value = '';
      status.value = 'saving';

      try {
        if ( operation.kind === 'save' ) {
          draft.value = await invoke( 'upsert_authoring_draft', {
            input: operation.input
          });
        } else {
          await invoke( 'delete_authoring_draft', {
            input: operation.input
          });
          draft.value = null;
        }
      } catch ( cause ) {
        if ( pendingOperation ) {
          continue;
        }

        pendingOperation = operation;
        error.value = draftErrorMessage( cause );
        status.value = 'error';

        return;
      }
    }

    status.value = draft.value ? 'saved' : 'untouched';
  }

  function locator() {
    return {
      kind,
      targetId: context.targetId
    };
  }

  function ensureStarted() {
    if ( !context ) {
      throw new Error( 'Authoring draft session has not started.' );
    }
  }

  function clearTimer() {
    if ( timer !== null ) {
      window.clearTimeout( timer );
      timer = null;
    }
  }

  return {
    discard,
    draft,
    error,
    flush,
    hasPendingPersistence,
    load,
    refresh,
    retry,
    scheduleDelete,
    scheduleSave,
    start,
    status
  };
}

function draftErrorMessage( error ) {
  if ( typeof error === 'string' ) {
    return error;
  }

  if ( error && typeof error.message === 'string' ) {
    return error.message;
  }

  return 'The draft could not be saved locally.';
}
