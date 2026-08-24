import { invoke } from '@tauri-apps/api/core';
import { computed, readonly, ref } from 'vue';

import { conceptLibraryErrorMessage } from './useConceptLibrary';

export const CSS_SNIPPET_SCHEMA_VERSION = 1;

const STYLE_ELEMENT_ID = 'twill-css-snippets';
const snippets = ref([]);
const loadError = ref( '' );
const loading = ref( false );
const pendingOperations = ref( 0 );
const ready = ref( false );
const safeMode = ref( false );

let initializationPromise = null;
let loadRequestSequence = 0;
let mutationQueue = Promise.resolve();

const enabledCount = computed( () => (
  snippets.value.filter( ( snippet ) => snippet.enabled ).length
) );
const isPending = computed( () => pendingOperations.value > 0 );

export function initializeCssSnippets({ force = false } = {}) {
  if ( initializationPromise && !force ) {
    return initializationPromise;
  }

  ensureStyleElement();

  const request = ++loadRequestSequence;

  loading.value = true;
  loadError.value = '';

  initializationPromise = Promise.all([
    invoke( 'get_css_snippets' ),
    invoke( 'get_css_snippet_runtime_state' )
  ])
    .then( ([ catalog, runtime ]) => {
      if ( request !== loadRequestSequence ) {
        return false;
      }

      snippets.value = sortSnippets( catalog.snippets );
      safeMode.value = runtime.safeMode;
      applyEnabledSnippets();

      return true;
    })
    .catch( ( cause ) => {
      if ( request === loadRequestSequence ) {
        loadError.value = conceptLibraryErrorMessage( cause );
      }

      return false;
    })
    .finally( () => {
      if ( request === loadRequestSequence ) {
        loading.value = false;
        ready.value = true;
      }
    });

  return initializationPromise;
}

export function useCssSnippets() {
  return {
    createSnippet,
    deleteSnippet,
    disableAllSnippets,
    enabledCount,
    initializeCssSnippets,
    isPending,
    loadError: readonly( loadError ),
    loading: readonly( loading ),
    ready: readonly( ready ),
    safeMode: readonly( safeMode ),
    setSnippetEnabled,
    snippets: readonly( snippets ),
    updateSnippet
  };
}

function createSnippet( name, source ) {
  return runMutation( async () => {
    const snippet = await invoke( 'create_css_snippet', {
      input: {
        name,
        content: {
          schemaVersion: CSS_SNIPPET_SCHEMA_VERSION,
          source
        }
      }
    });

    replaceSnippet( snippet );

    return snippet;
  });
}

function updateSnippet( id, name, source ) {
  return runMutation( async () => {
    const snippet = await invoke( 'update_css_snippet', {
      input: {
        id,
        name,
        content: {
          schemaVersion: CSS_SNIPPET_SCHEMA_VERSION,
          source
        }
      }
    });

    replaceSnippet( snippet );

    return snippet;
  });
}

function setSnippetEnabled( id, enabled ) {
  return runMutation( async () => {
    const snippet = await invoke( 'set_css_snippet_enabled', {
      input: { id, enabled }
    });

    replaceSnippet( snippet );

    return snippet;
  });
}

function disableAllSnippets() {
  return runMutation( async () => {
    await invoke( 'disable_all_css_snippets' );

    snippets.value = snippets.value.map( ( snippet ) => ({
      ...snippet,
      enabled: false
    }) );
    applyEnabledSnippets();
  });
}

function deleteSnippet( id ) {
  return runMutation( async () => {
    await invoke( 'delete_css_snippet', { input: { id } });

    snippets.value = snippets.value.filter( ( snippet ) => snippet.id !== id );
    applyEnabledSnippets();
  });
}

async function runMutation( operation ) {
  pendingOperations.value += 1;

  const request = mutationQueue
    .catch( () => undefined )
    .then( async () => {
      await initializationPromise;

      return operation();
    });

  mutationQueue = request;

  try {
    return await request;
  } finally {
    pendingOperations.value = Math.max( 0, pendingOperations.value - 1 );
  }
}

function replaceSnippet( updatedSnippet ) {
  const remainingSnippets = snippets.value.filter( ( snippet ) => (
    snippet.id !== updatedSnippet.id
  ) );

  snippets.value = sortSnippets([ ...remainingSnippets, updatedSnippet ]);
  applyEnabledSnippets();
}

function sortSnippets( values ) {
  return [ ...values ].sort( ( first, second ) => (
    first.name.localeCompare( second.name, 'en', { sensitivity: 'base' })
      || first.id.localeCompare( second.id )
  ) );
}

function ensureStyleElement() {
  let styleElement = document.getElementById( STYLE_ELEMENT_ID );

  if ( styleElement ) {
    return styleElement;
  }

  styleElement = document.createElement( 'style' );
  styleElement.id = STYLE_ELEMENT_ID;
  document.head.append( styleElement );

  return styleElement;
}

function applyEnabledSnippets() {
  const source = safeMode.value
    ? ''
    : snippets.value
      .filter( ( snippet ) => snippet.enabled )
      .map( ( snippet ) => snippet.content.source )
      .join( '\n\n' );

  ensureStyleElement().textContent = source;
  window.dispatchEvent( new CustomEvent( 'twill-css-snippets-changed' ) );
}
