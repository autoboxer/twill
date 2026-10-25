import { invoke } from '@tauri-apps/api/core';
import { computed, onBeforeUnmount, ref, watch } from 'vue';

function emptyState() {
  return { mode: 'workpad', started: false, responses: {}, principles: {}, revealed: [] };
}

export function useGuidedProblem({ currentCard, answerRevealed, blocked, markAssistance, onExposed }) {
  const state = ref( emptyState() );
  const loading = ref( false );
  const error = ref( '' );
  const example = computed( () => currentCard.value?.problem?.workedExample ?? null );
  let sequence = 0;
  let disposed = false;

  function chooseMode( mode ) {
    if ( blocked.value || !example.value || ![ 'workpad', 'example', 'completion' ].includes( mode ) ) {
      return;
    }

    if ( mode !== 'workpad' ) {
      state.value.started = true;
      markAssistance();
      onExposed();
    }

    state.value.mode = mode;
  }

  function toggleSolution( id ) {
    if ( blocked.value || answerRevealed.value || state.value.mode !== 'completion'
      || !example.value?.steps.some( ( step ) => step.id === id && !step.showInCompletion ) ) {
      return;
    }

    state.value.revealed = state.value.revealed.includes( id )
      ? state.value.revealed.filter( ( item ) => item !== id )
      : [ ...state.value.revealed, id ];
  }

  function setResponse( id, value, principle = false ) {
    if ( blocked.value || answerRevealed.value || state.value.mode !== 'completion'
      || typeof value !== 'string'
      || !example.value?.steps.some( ( step ) => step.id === id && ( !principle || step.principlePrompt ) ) ) {
      return;
    }

    const field = principle ? 'principles' : 'responses';

    state.value[ field ] = { ...state.value[ field ], [ id ]: value };
  }

  function snapshot() {
    return {
      ...state.value,
      responses: { ...state.value.responses },
      principles: { ...state.value.principles },
      revealed: [ ...state.value.revealed ]
    };
  }

  function restore( saved ) {
    sequence += 1;
    loading.value = false;
    error.value = '';
    state.value = emptyState();

    if ( !example.value || !saved ) {
      return;
    }

    const ids = new Set( example.value.steps.map( ( step ) => step.id ) );
    const responses = ( values ) => Object.fromEntries( Object.entries( values ?? {}).filter(
      ([ id, value ]) => ids.has( id ) && typeof value === 'string'
    ) );

    state.value = {
      mode: [ 'workpad', 'example', 'completion' ].includes( saved.mode ) ? saved.mode : 'workpad',
      started: saved.started === true,
      responses: responses( saved.responses ),
      principles: responses( saved.principles ),
      revealed: ( saved.revealed ?? []).filter( ( id ) => ids.has( id ) )
    };
  }

  async function validate() {
    const card = currentCard.value;

    if ( !state.value.started || !card ) {
      return true;
    }

    const token = ++sequence;

    loading.value = true;
    error.value = '';

    try {
      const concept = await invoke( 'get_concept', { conceptId: card.conceptId });

      return !disposed && token === sequence && !concept.archived
        && concept.lastChangeId === card.conceptLastChangeId
        && concept.cards.some( ( item ) => item.id === card.id && item.problem?.workedExample );
    } catch ( cause ) {
      if ( cause?.code !== 'notFound' && !disposed && token === sequence ) {
        error.value = 'Your work is preserved. Try resuming again.';
      }

      return false;
    } finally {
      if ( token === sequence ) {
        loading.value = false;
      }
    }
  }

  watch( () => currentCard.value?.id, () => restore(), { flush: 'sync' });
  onBeforeUnmount( () => {
    disposed = true;
    sequence += 1;
  });

  return { state, loading, error, chooseMode, toggleSolution, setResponse, snapshot, restore, validate };
}
