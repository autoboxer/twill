import { invoke } from '@tauri-apps/api/core';
import { computed, onBeforeUnmount, ref, watch } from 'vue';

import { conceptLibraryErrorMessage } from './useConceptLibrary';

export function useLinkedPractice({ sourceCard, answerRevealed, blocked }) {
  const active = ref( null );
  const open = ref( false );
  const links = ref([]);
  const selectedCase = ref( null );
  const loading = ref( false );
  const error = ref( '' );
  const results = ref([]);
  const available = computed( () => !active.value && Boolean( sourceCard.value )
    && answerRevealed.value && !blocked.value );
  let sequence = 0;
  let disposed = false;

  function close() {
    sequence += 1;
    open.value = false;
    selectedCase.value = null;
    loading.value = false;
    error.value = '';
  }

  async function choose() {
    if ( !available.value ) {
      return;
    }

    close();
    open.value = true;
    links.value = [];
    await loadLinks();
  }

  async function request( load, apply ) {
    const token = ++sequence;
    const sourceId = sourceCard.value?.id;

    loading.value = true;
    error.value = '';

    try {
      const result = await load();

      if ( disposed || token !== sequence || sourceCard.value?.id !== sourceId ) {
        return false;
      }

      return apply( result );
    } catch ( cause ) {
      if ( !disposed && token === sequence ) {
        error.value = cause?.code === 'notFound'
          ? 'This case or connection is no longer available. Choose another case.'
          : conceptLibraryErrorMessage( cause );
      }

      return false;
    } finally {
      if ( token === sequence ) {
        loading.value = false;
      }
    }
  }

  function loadLinks() {
    selectedCase.value = null;

    return request(
      () => invoke( 'get_practice_links', { conceptId: sourceCard.value.conceptId }),
      ( result ) => {
        links.value = result;
        return true;
      }
    );
  }

  function fetchCase( linkId ) {
    return invoke( 'get_linked_practice', {
      input: { conceptId: sourceCard.value.conceptId, linkId }
    }).then( ( result ) => {
      const { concept, templates } = result;
      const byId = new Map( templates.map( ( template ) => [ template.id, template ]) );
      const cards = concept.cards.map( ( card ) => ({
        ...card,
        conceptId: concept.id,
        conceptLastChangeId: concept.lastChangeId,
        conceptTitle: concept.title,
        content: concept.content,
        templateId: card.template?.id ?? null,
        template: card.template ? byId.get( card.template.id ) : null
      }) );

      return { ...result, cards };
    });
  }

  function select( link ) {
    if ( !open.value || loading.value || link.archived ) {
      return;
    }

    return request( () => fetchCase( link.id ), ( result ) => {
      selectedCase.value = result;
      return true;
    });
  }

  function start( cardId, origin ) {
    const selected = selectedCase.value;

    if ( !open.value || loading.value || !selected || active.value || blocked.value ) {
      return false;
    }

    return request( () => fetchCase( selected.link.id ), ( result ) => {
      if ( result.sourceLastChangeId !== sourceCard.value.conceptLastChangeId ) {
        error.value = 'The current concept changed. Start a new session before linked practice.';
        return false;
      }

      if ( result.concept.lastChangeId !== selected.concept.lastChangeId
        || result.link.lastChangeId !== selected.link.lastChangeId
        || JSON.stringify( result.templates ) !== JSON.stringify( selected.templates ) ) {
        selectedCase.value = result;
        error.value = 'This case changed. Choose a card again.';
        return false;
      }

      const card = result.cards.find( ( candidate ) => candidate.id === cardId );

      if ( !card ) {
        selectedCase.value = result;
        error.value = 'This card is no longer available. Choose another card.';
        return false;
      }

      active.value = {
        card,
        link: result.link,
        media: result.concept.media,
        sourceConceptId: sourceCard.value.conceptId,
        sourceLastChangeId: sourceCard.value.conceptLastChangeId,
        answerRevealed: false,
        origin
      };
      close();
      return true;
    });
  }

  function finish( outcome = null ) {
    const practice = active.value;

    if ( !practice ) {
      return null;
    }

    if ( outcome ) {
      results.value = [ ...results.value, {
        cardId: practice.card.id,
        conceptId: practice.card.conceptId,
        ...outcome
      }];
    }

    active.value = null;
    return practice.origin;
  }

  function reset() {
    close();
    active.value = null;
    results.value = [];
  }

  function snapshot() {
    return { active: active.value, results: [ ...results.value ] };
  }

  function restore( saved ) {
    close();
    active.value = saved?.active ?? null;
    results.value = [ ...( saved?.results ?? []) ];
  }

  async function validate() {
    const practice = active.value;

    if ( !practice ) {
      return { valid: true, sourceChanged: false };
    }

    if ( practice.sourceConceptId !== sourceCard.value?.conceptId
      || practice.sourceLastChangeId !== sourceCard.value?.conceptLastChangeId ) {
      return { valid: false, sourceChanged: true };
    }

    let status = { valid: false, sourceChanged: false };

    await request( async () => {
      try {
        const result = await fetchCase( practice.link.id );
        const card = result.cards.find( ( card ) => card.id === practice.card.id );

        return {
          sourceChanged: result.sourceLastChangeId !== practice.sourceLastChangeId,
          valid: result.sourceLastChangeId === practice.sourceLastChangeId
            && result.link.lastChangeId === practice.link.lastChangeId
            && card?.conceptLastChangeId === practice.card.conceptLastChangeId
            && JSON.stringify( card?.template ) === JSON.stringify( practice.card.template )
        };
      } catch {
        try {
          const source = await invoke( 'get_concept', { conceptId: practice.sourceConceptId });

          return { valid: false, sourceChanged: source.archived || source.lastChangeId !== practice.sourceLastChangeId };
        } catch ( cause ) {
          return { valid: false, sourceChanged: cause?.code === 'notFound' };
        }
      }
    }, ( result ) => {
      status = result;
      return result.valid;
    });

    return status;
  }

  watch( () => sourceCard.value?.id, close, { flush: 'sync' });

  onBeforeUnmount( () => {
    disposed = true;
    sequence += 1;
  });

  return {
    active,
    available,
    choose,
    close,
    error,
    finish,
    links,
    loadLinks,
    loading,
    open,
    reset,
    restore,
    results,
    select,
    selectedCase,
    snapshot,
    start,
    validate
  };
}
