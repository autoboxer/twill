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
  const masteryIndex = ref( 0 );
  const masteryItems = ref([]);
  const masteryResults = ref([]);
  const masteryStarted = ref( false );
  const ratingCounts = ref( emptyRatingCounts() );

  const completedCount = computed( () => currentIndex.value );
  const hasCards = computed( () => cards.value.length > 0 );
  const reviewComplete = computed( () => (
    hasCards.value && currentIndex.value >= cards.value.length
  ) );
  const masteryActive = computed( () => (
    masteryStarted.value && masteryIndex.value < masteryItems.value.length
  ) );
  const masteryReady = computed( () => (
    reviewComplete.value
    && masteryItems.value.length > 0
    && !masteryStarted.value
  ) );
  const currentCard = computed( () => {
    if ( masteryActive.value ) {
      return masteryItems.value[ masteryIndex.value ]?.card ?? null;
    }

    if ( reviewComplete.value ) {
      return null;
    }

    return cards.value[ currentIndex.value ] ?? null;
  });

  const isComplete = computed( () => {
    if ( !reviewComplete.value ) {
      return false;
    }

    return !masteryItems.value.length || (
      masteryStarted.value
      && masteryIndex.value >= masteryItems.value.length
    );
  });

  const lastAssessment = computed( () => assessments.value.at( -1 ) ?? null );
  const masteryCompletedCount = computed( () => masteryIndex.value );
  const masteryMissedCount = computed( () => (
    masteryResults.value.filter( ( result ) => !result.recalled ).length
  ) );
  const masteryRecalledCount = computed( () => (
    masteryResults.value.filter( ( result ) => result.recalled ).length
  ) );
  const masteryTotal = computed( () => masteryItems.value.length );

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
    if (
      masteryStarted.value
      || reviewComplete.value
      || !answerRevealed.value
      || !currentCard.value
    ) {
      return false;
    }

    if (
      !REVIEW_RATINGS.includes( rating )
      || typeof reviewId !== 'string'
      || !reviewId
    ) {
      return false;
    }

    const card = currentCard.value;

    assessments.value.push({
      cardId: card.id,
      conceptId: card.conceptId,
      rating,
      response,
      reviewId
    });

    if ( rating === 'again' ) {
      masteryItems.value.push({ card });
    }

    ratingCounts.value[ rating ] += 1;
    currentIndex.value += 1;
    answerRevealed.value = false;
    correctionPending.value = false;

    return true;
  }

  function startMastery() {
    if ( !masteryReady.value ) {
      return false;
    }

    masteryItems.value = orderMasteryItems(
      masteryItems.value,
      cards.value.at( -1 )?.conceptId
    );
    masteryIndex.value = 0;
    masteryResults.value = [];
    masteryStarted.value = true;
    answerRevealed.value = false;
    correctionPending.value = false;

    return true;
  }

  function assessMastery({ recalled, response }) {
    const item = masteryItems.value[ masteryIndex.value ];

    if (
      !masteryActive.value
      || !answerRevealed.value
      || !item
      || typeof recalled !== 'boolean'
    ) {
      return false;
    }

    masteryResults.value.push({
      cardId: item.card.id,
      conceptId: item.card.conceptId,
      recalled,
      response
    });
    masteryIndex.value += 1;
    answerRevealed.value = false;

    return true;
  }

  function restoreLastAssessment( reviewId ) {
    const assessment = lastAssessment.value;
    const previousIndex = currentIndex.value - 1;

    if (
      correctionPending.value
      || masteryStarted.value
      || !assessment
      || assessment.reviewId !== reviewId
      || previousIndex < 0
      || cards.value[ previousIndex ]?.id !== assessment.cardId
    ) {
      return null;
    }

    assessments.value.pop();
    ratingCounts.value[ assessment.rating ] -= 1;

    if ( assessment.rating === 'again' ) {
      masteryItems.value = masteryItems.value.filter( ( item ) => (
        item.card.id !== assessment.cardId
      ) );
    }

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
    masteryIndex.value = 0;
    masteryItems.value = [];
    masteryResults.value = [];
    masteryStarted.value = false;
    ratingCounts.value = emptyRatingCounts();
  }

  function createSnapshot() {
    return {
      answerRevealed: answerRevealed.value,
      assessments: assessments.value.map( ( assessment ) => ({ ...assessment }) ),
      cards: [ ...cards.value ],
      correctionPending: correctionPending.value,
      currentIndex: currentIndex.value,
      masteryIndex: masteryIndex.value,
      masteryItems: masteryItems.value.map( ( item ) => ({ ...item }) ),
      masteryResults: masteryResults.value.map( ( result ) => ({ ...result }) ),
      masteryStarted: masteryStarted.value,
      ratingCounts: { ...ratingCounts.value }
    };
  }

  function restoreSnapshot( snapshot ) {
    answerRevealed.value = snapshot.answerRevealed;
    assessments.value = snapshot.assessments.map( ( assessment ) => ({
      ...assessment
    }) );
    cards.value = [ ...snapshot.cards ];
    correctionPending.value = snapshot.correctionPending;
    currentIndex.value = snapshot.currentIndex;
    masteryIndex.value = snapshot.masteryIndex ?? 0;
    masteryItems.value = ( snapshot.masteryItems ?? []).map( ( item ) => ({
      ...item
    }) );
    masteryResults.value = ( snapshot.masteryResults ?? []).map( ( result ) => ({
      ...result
    }) );
    masteryStarted.value = Boolean( snapshot.masteryStarted );
    ratingCounts.value = { ...snapshot.ratingCounts };
  }

  return {
    answerRevealed,
    assess,
    assessMastery,
    begin,
    completedCount,
    correctionPending,
    createSnapshot,
    currentCard,
    hasCards,
    isComplete,
    lastAssessment,
    masteryActive,
    masteryCompletedCount,
    masteryMissedCount,
    masteryReady,
    masteryRecalledCount,
    masteryStarted,
    masteryTotal,
    position,
    progress,
    ratingCounts,
    revealAnswer,
    restoreSnapshot,
    restoreLastAssessment,
    startMastery,
    totalCards
  };
}

function orderMasteryItems( items, precedingConceptId ) {
  const conceptQueues = new Map();
  const ordered = [];
  let previousConceptId = precedingConceptId;

  items.forEach( ( item, index ) => {
    const conceptId = item.card.conceptId;

    if ( !conceptQueues.has( conceptId ) ) {
      conceptQueues.set( conceptId, {
        conceptId,
        firstMissIndex: index,
        items: [],
        nextItemIndex: 0
      });
    }

    conceptQueues.get( conceptId ).items.push( item );
  });

  const priorityQueue = createPriorityQueue([ ...conceptQueues.values() ]);

  while ( priorityQueue.length ) {
    let selected = popPriorityQueue( priorityQueue );

    if (
      selected.conceptId === previousConceptId
      && priorityQueue.length
    ) {
      const alternative = popPriorityQueue( priorityQueue );

      pushPriorityQueue( priorityQueue, selected );
      selected = alternative;
    }

    ordered.push( selected.items[ selected.nextItemIndex ]);
    selected.nextItemIndex += 1;

    if ( masteryItemsRemaining( selected ) ) {
      pushPriorityQueue( priorityQueue, selected );
    }

    previousConceptId = selected.conceptId;
  }

  return ordered;
}

function createPriorityQueue( conceptQueues ) {
  const priorityQueue = [ ...conceptQueues ];

  for ( let index = Math.floor( priorityQueue.length / 2 ) - 1; index >= 0; index -= 1 ) {
    siftPriorityQueueDown( priorityQueue, index );
  }

  return priorityQueue;
}

function popPriorityQueue( priorityQueue ) {
  const selected = priorityQueue[ 0 ];
  const last = priorityQueue.pop();

  if ( priorityQueue.length ) {
    priorityQueue[ 0 ] = last;
    siftPriorityQueueDown( priorityQueue, 0 );
  }

  return selected;
}

function pushPriorityQueue( priorityQueue, conceptQueue ) {
  priorityQueue.push( conceptQueue );

  let index = priorityQueue.length - 1;

  while ( index > 0 ) {
    const parentIndex = Math.floor( ( index - 1 ) / 2 );

    if ( !masteryQueuePrecedes(
      priorityQueue[ index ],
      priorityQueue[ parentIndex ]
    ) ) {
      break;
    }

    [ priorityQueue[ index ], priorityQueue[ parentIndex ] ] = [
      priorityQueue[ parentIndex ],
      priorityQueue[ index ]
    ];
    index = parentIndex;
  }
}

function siftPriorityQueueDown( priorityQueue, startIndex ) {
  let index = startIndex;

  while ( true ) {
    const leftIndex = index * 2 + 1;
    const rightIndex = leftIndex + 1;
    let selectedIndex = index;

    if (
      leftIndex < priorityQueue.length
      && masteryQueuePrecedes(
        priorityQueue[ leftIndex ],
        priorityQueue[ selectedIndex ]
      )
    ) {
      selectedIndex = leftIndex;
    }

    if (
      rightIndex < priorityQueue.length
      && masteryQueuePrecedes(
        priorityQueue[ rightIndex ],
        priorityQueue[ selectedIndex ]
      )
    ) {
      selectedIndex = rightIndex;
    }

    if ( selectedIndex === index ) {
      return;
    }

    [ priorityQueue[ index ], priorityQueue[ selectedIndex ] ] = [
      priorityQueue[ selectedIndex ],
      priorityQueue[ index ]
    ];
    index = selectedIndex;
  }
}

function masteryQueuePrecedes( first, second ) {
  const firstCount = masteryItemsRemaining( first );
  const secondCount = masteryItemsRemaining( second );

  if ( firstCount !== secondCount ) {
    return firstCount > secondCount;
  }

  return first.firstMissIndex < second.firstMissIndex;
}

function masteryItemsRemaining( conceptQueue ) {
  return conceptQueue.items.length - conceptQueue.nextItemIndex;
}
