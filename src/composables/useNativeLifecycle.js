import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { computed, nextTick, onScopeDispose, readonly, ref, unref, watch } from 'vue';

import { suspendCssSnippets } from './useCssSnippets';

const request = ref( null );
const error = ref( '' );
const working = ref( false );
const completing = ref( false );
const guards = new Set();

let attempt = null;

const pending = computed( () => request.value !== null );

export async function initializeNativeLifecycle() {
  if ( !isTauri() ) {
    return;
  }

  await listen( 'twill-css-snippets-disabled', suspendCssSnippets );
  await listen( 'twill-native-action', ( event ) => {
    if ( request.value ) {
      return;
    }

    request.value = event.payload;
    void retry();
  });
  await invoke( 'native_lifecycle_ready' );
}

export function useNativeActionGuard({ busy, flush }) {
  const guard = async ( signal ) => {
    await waitUntilIdle( busy, signal );
    await nextTick();

    if ( !signal.aborted ) {
      await flush();
    }
  };

  guards.add( guard );
  onScopeDispose( () => guards.delete( guard ) );

  return { nativeActionPending: pending };
}

export function useNativeLifecycle() {
  return {
    completing: readonly( completing ),
    error: readonly( error ),
    pending,
    request: readonly( request ),
    retry,
    stay,
    working: readonly( working )
  };
}

async function retry() {
  if ( !request.value || working.value || completing.value ) {
    return;
  }

  const current = request.value;
  const controller = new AbortController();
  let approved = false;

  attempt = controller;
  working.value = true;
  error.value = '';

  try {
    await nextTick();

    for ( const guard of guards ) {
      await guard( controller.signal );
    }

    if ( controller.signal.aborted ) {
      return;
    }

    completing.value = true;
    await invoke( 'complete_native_action', { requestId: current.id, proceed: true });
    approved = true;
  } catch ( cause ) {
    if ( !controller.signal.aborted ) {
      error.value = typeof cause === 'string' ? cause : cause?.message
        || 'The latest changes could not be saved.';
    }
  } finally {
    if ( attempt === controller && !approved ) {
      working.value = false;
      completing.value = false;
    }
  }
}

async function stay() {
  if ( !request.value || completing.value ) {
    return;
  }

  attempt?.abort();
  attempt = null;
  completing.value = true;

  try {
    await invoke( 'complete_native_action', { requestId: request.value.id, proceed: false });
    request.value = null;
    error.value = '';
  } catch {
    error.value = 'The window action could not be cancelled. Please try again.';
  } finally {
    completing.value = false;
    working.value = false;
  }
}

function waitUntilIdle( busy, signal ) {
  if ( !unref( busy ) || signal.aborted ) {
    return Promise.resolve();
  }

  return new Promise( ( resolve ) => {
    const stop = watch( busy, ( value ) => {
      if ( !value ) {
        finish();
      }
    });

    function finish() {
      stop();
      signal.removeEventListener( 'abort', finish );
      resolve();
    }

    signal.addEventListener( 'abort', finish, { once: true });
  });
}
