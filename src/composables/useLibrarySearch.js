import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';

import { parseLibraryQuery, serializeLibraryQuery } from '../library/search';
import { conceptLibraryErrorMessage, useConceptLibrary } from './useConceptLibrary';

export function useLibrarySearch() {
  const route = useRoute();
  const router = useRouter();
  const { getLibrary, getLibraryOrganizations } = useConceptLibrary();
  const input = ref( parseLibraryQuery( route.query ) );
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
  const navigationQuery = computed( () => serializeLibraryQuery( input.value ) );
  const hasFilters = computed( () => Boolean(
    input.value.query.trim() || input.value.deckId || input.value.tagId
    || input.value.cardType !== 'all' || input.value.state !== 'all'
    || input.value.includeArchived
  ) );
  const pendingLocations = new Map();
  let locationSequence = 0;
  let catalogLoaded = false;
  let requestSequence = 0;
  let searchTimer;
  let disposed = false;

  function field( name ) {
    return computed({
      get: () => input.value[ name ],
      set: ( value ) => update({ [ name ]: value })
    });
  }

  function update( changes ) {
    const previousQuery = input.value.query;

    input.value = { ...input.value, page: 1, ...changes };
    scheduleSearch( input.value.query !== previousQuery ? 180 : 0 );
    void writeLocation();
  }

  async function writeLocation() {
    const query = navigationQuery.value;
    const write = ++locationSequence;

    pendingLocations.set( write, JSON.stringify( query ) );

    try {
      await router.replace({ name: 'library', query });
    } catch ( cause ) {
      if ( !disposed && write === locationSequence ) {
        loadError.value = cause.message || 'Search location could not be updated.';
      }
    } finally {
      pendingLocations.delete( write );

      if ( !pendingLocations.size ) {
        readLocation();
      }
    }
  }

  function readLocation() {
    if ( disposed || route.name !== 'library' ) {
      requestSequence += 1;
      clearTimeout( searchTimer );
      return;
    }

    const next = parseLibraryQuery( route.query );
    const key = JSON.stringify( serializeLibraryQuery( next ) );

    // Older URL writes must not replace newer input while navigation is settling
    if ( key === JSON.stringify( navigationQuery.value )
      || [ ...pendingLocations.values() ].includes( key ) ) {
      return;
    }

    input.value = next;
    scheduleSearch();
  }

  function scheduleSearch( delay = 0 ) {
    const request = ++requestSequence;

    clearTimeout( searchTimer );
    loading.value = true;
    loadError.value = '';
    searchTimer = setTimeout( () => load( request ), delay );
  }

  async function load( request ) {
    const searchInput = {
      ...input.value,
      cardType: input.value.cardType === 'all' ? null : input.value.cardType,
      state: input.value.state === 'all' ? null : input.value.state
    };

    try {
      const [ result, catalog ] = await Promise.all([
        getLibrary( searchInput ),
        !catalogLoaded ? getLibraryOrganizations() : null
      ]);

      if ( request !== requestSequence ) {
        return;
      }

      if ( catalog ) {
        organizations.value = catalog;
        catalogLoaded = true;
      }

      const missing = {};

      for ( const [ field, choices ] of [[ 'deckId', 'decks' ], [ 'tagId', 'tags' ]]) {
        if ( input.value[ field ]
          && !organizations.value[ choices ].some( ( item ) => item.id === input.value[ field ]) ) {
          missing[ field ] = null;
        }
      }

      if ( Object.keys( missing ).length ) {
        update( missing );
        return;
      }

      library.value = result;

      if ( input.value.page !== result.page ) {
        input.value = { ...input.value, page: result.page };
        void writeLocation();
      }
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
    update({ ...parseLibraryQuery({}), sort: input.value.sort });
  }

  watch( () => route.fullPath, readLocation, { flush: 'sync' });

  onBeforeUnmount( () => {
    disposed = true;
    requestSequence += 1;
    clearTimeout( searchTimer );
  });

  scheduleSearch();

  return {
    cardType: field( 'cardType' ),
    clearFilters,
    deckId: field( 'deckId' ),
    hasFilters,
    includeArchived: field( 'includeArchived' ),
    library,
    loadError,
    loading,
    navigationQuery,
    organizations,
    page: field( 'page' ),
    query: field( 'query' ),
    refresh: () => scheduleSearch(),
    refreshOrganizations,
    sort: field( 'sort' ),
    state: field( 'state' ),
    tagId: field( 'tagId' ),
    update
  };
}
