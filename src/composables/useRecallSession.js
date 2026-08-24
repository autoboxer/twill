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
  const assessments = ref([]);
  const cards = ref([]);
  const correctionPending = ref( false );
  const currentIndex = ref( 0 );
  const ratingCounts = ref( emptyRatingCounts() );

  const completedCount = computed( () => currentIndex.value );
  const currentCard = computed( () => cards.value[ currentIndex.value ] ?? null );
  const hasCards = computed( () => cards.value.length > 0 );

  const isComplete = computed( () => {
    return hasCards.value && currentIndex.value >= cards.value.length;
  });

  const lastAssessment = computed( () => assessments.value.at( -1 ) ?? null );

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

  function assess({ rating, response, reviewId }) {
    if ( !answerRevealed.value || !currentCard.value ) {
      return false;
    }

    if (
      !REVIEW_RATINGS.includes( rating )
      || typeof reviewId !== 'string'
      || !reviewId
    ) {
      return false;
    }

    assessments.value.push({
      cardId: currentCard.value.id,
      rating,
      response,
      reviewId
    });
    ratingCounts.value[ rating ] += 1;
    currentIndex.value += 1;
    answerRevealed.value = false;
    correctionPending.value = false;

    return true;
  }

  function restoreLastAssessment( reviewId ) {
    const assessment = lastAssessment.value;
    const previousIndex = currentIndex.value - 1;

    if (
      correctionPending.value
      || !assessment
      || assessment.reviewId !== reviewId
      || previousIndex < 0
      || cards.value[ previousIndex ]?.id !== assessment.cardId
    ) {
      return null;
    }

    assessments.value.pop();
    ratingCounts.value[ assessment.rating ] -= 1;
    currentIndex.value = previousIndex;
    answerRevealed.value = true;
    correctionPending.value = true;

    return assessment;
  }

  function restart() {
    answerRevealed.value = false;
    assessments.value = [];
    correctionPending.value = false;
    currentIndex.value = 0;
    ratingCounts.value = emptyRatingCounts();
  }

  return {
    answerRevealed,
    assess,
    begin,
    completedCount,
    correctionPending,
    currentCard,
    hasCards,
    isComplete,
    lastAssessment,
    position,
    progress,
    ratingCounts,
    revealAnswer,
    restoreLastAssessment,
    totalCards
  };
}
