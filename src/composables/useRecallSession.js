import { computed, ref } from 'vue';

const NEEDS_WORK = 'needsWork';
const RECALLED = 'recalled';

export function useRecallSession() {
  const answerRevealed = ref( false );
  const cards = ref([]);
  const currentIndex = ref( 0 );
  const needsWorkCount = ref( 0 );
  const recalledCount = ref( 0 );

  const completedCount = computed( () => currentIndex.value );
  const currentCard = computed( () => cards.value[ currentIndex.value ] ?? null );
  const hasCards = computed( () => cards.value.length > 0 );
  const isComplete = computed( () => {
    return hasCards.value && currentIndex.value >= cards.value.length;
  });
  const position = computed( () => {
    return Math.min( currentIndex.value + 1, cards.value.length );
  });
  const progress = computed( () => {
    if ( !cards.value.length ) {
      return 0;
    }

    return completedCount.value / cards.value.length * 100;
  });
  const totalCards = computed( () => cards.value.length );

  function begin( studyCards ) {
    cards.value = [ ...studyCards ];
    restart();
  }

  function revealAnswer() {
    if ( currentCard.value ) {
      answerRevealed.value = true;
    }
  }

  function assess( outcome ) {
    if ( !answerRevealed.value || !currentCard.value ) {
      return false;
    }

    if ( outcome === NEEDS_WORK ) {
      needsWorkCount.value += 1;
    } else if ( outcome === RECALLED ) {
      recalledCount.value += 1;
    } else {
      return false;
    }

    currentIndex.value += 1;
    answerRevealed.value = false;

    return true;
  }

  function restart() {
    answerRevealed.value = false;
    currentIndex.value = 0;
    needsWorkCount.value = 0;
    recalledCount.value = 0;
  }

  return {
    answerRevealed,
    assess,
    begin,
    completedCount,
    currentCard,
    hasCards,
    isComplete,
    needsWorkCount,
    position,
    progress,
    recalledCount,
    restart,
    revealAnswer,
    totalCards
  };
}
