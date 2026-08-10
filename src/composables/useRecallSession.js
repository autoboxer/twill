import { computed, ref } from 'vue';

const REVIEW_RATINGS = [ 'again', 'hard', 'good', 'easy' ];

function emptyRatingCounts() {
  return {
    again: 0,
    hard: 0,
    good: 0,
    easy: 0
  };
}

export function useRecallSession() {
  const answerRevealed = ref( false );
  const cards = ref([]);
  const currentIndex = ref( 0 );
  const ratingCounts = ref( emptyRatingCounts() );

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

  function assess( rating ) {
    if ( !answerRevealed.value || !currentCard.value ) {
      return false;
    }

    if ( !REVIEW_RATINGS.includes( rating ) ) {
      return false;
    }

    ratingCounts.value[ rating ] += 1;
    currentIndex.value += 1;
    answerRevealed.value = false;

    return true;
  }

  function restart() {
    answerRevealed.value = false;
    currentIndex.value = 0;
    ratingCounts.value = emptyRatingCounts();
  }

  return {
    answerRevealed,
    assess,
    begin,
    completedCount,
    currentCard,
    hasCards,
    isComplete,
    position,
    progress,
    ratingCounts,
    revealAnswer,
    totalCards
  };
}
