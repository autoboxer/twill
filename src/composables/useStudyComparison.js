import { computed, ref } from 'vue';

import { comparisonChoices, comparisonTargetIds } from '../study/comparison';

export function useStudyComparison({ currentCard, answerRevealed, actionsBlocked }) {
  const comparisonChecks = ref({});
  const targetIds = computed( () => new Set( comparisonTargetIds( currentCard.value ) ) );

  function comparisonSnapshot() {
    return { ...comparisonChecks.value };
  }

  function restoreComparison( saved = {}) {
    comparisonChecks.value = Object.fromEntries(
      Object.entries( saved ).filter( ([ id, value ]) => (
        targetIds.value.has( id ) && isJudgment( value )
      ) )
    );
  }

  function isJudgment( value ) {
    return value !== 'unchecked' && comparisonChoices.some( ( choice ) => choice.value === value );
  }

  function setComparison( id, value ) {
    if ( actionsBlocked.value || !answerRevealed.value || !targetIds.value.has( id ) ) {
      return;
    }

    const checks = { ...comparisonChecks.value };

    if ( value === 'unchecked' ) {
      delete checks[ id ];
    } else if ( isJudgment( value ) ) {
      checks[ id ] = value;
    } else {
      return;
    }

    comparisonChecks.value = checks;
  }

  return {
    comparisonChecks,
    comparisonSnapshot,
    restoreComparison,
    setComparison
  };
}
