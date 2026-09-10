import { onBeforeUnmount, ref, watch } from 'vue';

import { conceptLibraryErrorMessage, useConceptLibrary } from './useConceptLibrary';

export function useLibrarySearch() {
  const { getLibrary, getLibraryOrganizations } = useConceptLibrary();
  const query = ref( '' );
  const activeFilter = ref({ id: '', kind: 'all' });
  const includeArchived = ref( false );
  const page = ref( 1 );
  const loading = ref( true );
  const loadError = ref( '' );
  const organizations = ref({ decks: [], tags: [] });
  const library = ref({
    concepts: [],
    conceptCount: 0,
    archivedCount: 0,
    totalCount: 0,
    page: 1,
    pageSize: 50
  });
  let catalogLoaded = false;
  let requestSequence = 0;
  let searchTimer;

  function scheduleSearch( delay = 0 ) {
    const request = ++requestSequence;

    clearTimeout( searchTimer );
    loading.value = true;
    loadError.value = '';
    searchTimer = setTimeout( () => load( request ), delay );
  }

  async function load( request ) {
    const input = {
      query: query.value,
      includeArchived: includeArchived.value,
      deckId: activeFilter.value.kind === 'deck' ? activeFilter.value.id : null,
      tagId: activeFilter.value.kind === 'tag' ? activeFilter.value.id : null,
      page: page.value
    };
    const needsCatalog = !catalogLoaded;

    try {
      const [ result, catalog ] = await Promise.all([
        getLibrary( input ),
        needsCatalog ? getLibraryOrganizations() : null
      ]);

      if ( request !== requestSequence ) {
        return;
      }

      if ( catalog ) {
        organizations.value = catalog;
        catalogLoaded = true;
      }

      const choices = activeFilter.value.kind === 'deck'
        ? organizations.value.decks
        : organizations.value.tags;

      if ( activeFilter.value.kind !== 'all'
        && !choices.some( ( item ) => item.id === activeFilter.value.id ) ) {
        activeFilter.value = { id: '', kind: 'all' };
        return;
      }

      library.value = result;
    } catch ( cause ) {
      if ( request === requestSequence ) {
        loadError.value = conceptLibraryErrorMessage( cause );
      }
    } finally {
      if ( request === requestSequence ) {
        loading.value = false;
      }
    }
  }

  function refreshOrganizations() {
    catalogLoaded = false;
    scheduleSearch();
  }

  function clearFilters() {
    query.value = '';
    activeFilter.value = { id: '', kind: 'all' };
  }

  watch([ query, includeArchived, activeFilter ], ( values, previous ) => {
    page.value = 1;
    scheduleSearch( values[ 0 ] !== previous[ 0 ] ? 180 : 0 );
  }, { flush: 'sync' });

  watch( page, () => scheduleSearch(), { flush: 'sync' });

  onBeforeUnmount( () => {
    requestSequence += 1;
    clearTimeout( searchTimer );
  });

  scheduleSearch();

  return {
    activeFilter,
    clearFilters,
    includeArchived,
    library,
    loadError,
    loading,
    organizations,
    page,
    query,
    refresh: () => scheduleSearch(),
    refreshOrganizations
  };
}
