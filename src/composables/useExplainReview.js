import { computed, ref } from 'vue';

import { explainFocusDetails } from '../explain/focus';
import { exampleFollowUp } from '../explain/followUp';

function emptyReview() {
  return {
    open: false,
    correction: '',
    followUpOpen: false,
    followUpResponse: '',
    followUpRevealed: false
  };
}

export function useExplainReview({ currentCard, answerRevealed, actionsBlocked }) {
  const state = ref( emptyReview() );
  const available = computed( () => {
    const focus = explainFocusDetails( currentCard.value?.explain?.focus );

    return focus.reconstruction || focus.generation;
  });
  const followUp = computed( () => exampleFollowUp( currentCard.value?.explain ) );

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

  function toggleFollowUp() {
    if ( canChange() && followUp.value ) {
      state.value.followUpOpen = !state.value.followUpOpen;
    }
  }

  function setFollowUpResponse( value ) {
    if ( canChange() && followUp.value && state.value.followUpOpen
      && !state.value.followUpRevealed && typeof value === 'string' ) {
      state.value.followUpResponse = value;
    }
  }

  function toggleFollowUpReference() {
    if ( canChange() && followUp.value && state.value.followUpOpen ) {
      state.value.followUpRevealed = !state.value.followUpRevealed;
    }
  }

  function restore( saved ) {
    state.value = available.value && saved ? {
      open: saved.open === true,
      correction: typeof saved.correction === 'string' ? saved.correction : '',
      followUpOpen: Boolean( followUp.value && saved.followUpOpen ),
      followUpResponse: followUp.value && typeof saved.followUpResponse === 'string' ? saved.followUpResponse : '',
      followUpRevealed: Boolean( followUp.value && saved.followUpRevealed )
    } : emptyReview();
  }

  return {
    state, toggle, setCorrection, snapshot, restore,
    toggleFollowUp, setFollowUpResponse, toggleFollowUpReference
  };
}
