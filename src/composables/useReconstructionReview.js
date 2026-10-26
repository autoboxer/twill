import { computed, ref } from 'vue';

import { explainFocusDetails } from '../explain/focus';

function emptyReview() {
  return { open: false, correction: '' };
}

export function useReconstructionReview({ currentCard, answerRevealed, actionsBlocked }) {
  const state = ref( emptyReview() );
  const available = computed( () => Boolean(
    currentCard.value?.explain
    && explainFocusDetails( currentCard.value.explain.focus ).reconstruction
  ) );

  function canChange() {
    return available.value && answerRevealed.value && !actionsBlocked.value;
  }

  function toggle() {
    if ( canChange() ) {
      state.value.open = !state.value.open;
    }
  }

  function setCorrection( value ) {
    if ( canChange() && state.value.open && typeof value === 'string' ) {
      state.value.correction = value;
    }
  }

  function snapshot() {
    return { ...state.value };
  }

  function restore( saved ) {
    state.value = available.value && saved ? {
      open: saved.open === true,
      correction: typeof saved.correction === 'string' ? saved.correction : ''
    } : emptyReview();
  }

  return { state, toggle, setCorrection, snapshot, restore };
}
