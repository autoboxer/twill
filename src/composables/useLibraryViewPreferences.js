import { readonly, ref } from 'vue';

import { useDevicePreferences } from './useDevicePreferences';

const { getDevicePreferences, setLibraryViewPreferences } = useDevicePreferences();
const preferences = ref({ showTags: false, showDecks: false, showPromptPreview: false });
const ready = ref( false );
const loading = ref( false );
const error = ref( '' );

let persisted = { ...preferences.value };
let persistenceQueue = Promise.resolve();
let revision = 0;

async function load() {
  if ( ready.value || loading.value ) {
    return;
  }

  loading.value = true;
  error.value = '';

  try {
    const device = await getDevicePreferences();

    persisted = device.libraryView;
    preferences.value = { ...persisted };
    ready.value = true;
  } catch {
    error.value = 'View options could not be loaded.';
  } finally {
    loading.value = false;
  }
}

async function setOption( name, enabled ) {
  if ( !ready.value ) {
    return;
  }

  const requested = { ...preferences.value, [ name ]: enabled };
  const requestRevision = ++revision;

  preferences.value = requested;
  error.value = '';

  // Keep rapid changes in order, including when the Library is left and reopened
  const request = persistenceQueue
    .catch( () => undefined )
    .then( () => setLibraryViewPreferences( requested ) );

  persistenceQueue = request;

  try {
    const device = await request;

    persisted = device.libraryView;

    if ( requestRevision === revision ) {
      preferences.value = { ...persisted };
    }
  } catch {
    if ( requestRevision === revision ) {
      preferences.value = { ...persisted };
      error.value = 'View options could not be saved. Previous options restored; try again.';
    }
  }
}

export function useLibraryViewPreferences() {
  void load();

  return {
    preferences: readonly( preferences ),
    ready: readonly( ready ),
    loading: readonly( loading ),
    error: readonly( error ),
    load,
    setOption
  };
}
