import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue';

import { conceptLibraryErrorMessage, useConceptLibrary } from './useConceptLibrary';
import { useDevicePreferences } from './useDevicePreferences';
import { useRecallSession } from './useRecallSession';
import { richDocumentHasContent } from '../rich-content/schema';
import { gradingOptionsByMode } from '../study/grading';
import { takeStudySession } from '../study/resume';
import { emptyStudySelection, hasStudySelection, studyQuery } from '../study/selection';
import { normalizeTypeAnswer } from '../type-answer/comparison';

export function useStudySession({
  onStateChanged,
  onAnswerRevealed,
  onFeedbackContinued
}) {
  const {
    clearError,
    getStudyQueue,
    recordPretest,
    recordReview,
    reverseReview
  } = useConceptLibrary();

  const {
    getDevicePreferences,
    setGradingMode
  } = useDevicePreferences();

  const {
    answerRevealed,
    assess,
    assessMastery,
    begin,
    beginPretestTeaching,
    completedCount,
    completePretestTeaching,
    correctionPending,
    createSnapshot,
    currentCard,
    hasCards,
    isComplete,
    lastAssessment,
    lastAssessmentCanBeRestored,
    masteryActive,
    masteryCompletedCount,
    masteryMissedCount,
    masteryReady,
    masteryRecalledCount,
    masteryStarted,
    masteryTotal,
    pretestActive,
    pretestAttemptedCount,
    pretestSkippedCount,
    pretestTeachingActive,
    pretestTotal,
    position,
    ratingCounts,
    revealAnswer,
    restoreLastAssessment,
    restoreSnapshot,
    skipPretest,
    startMastery,
    totalCards
  } = useRecallSession();

  const initialLoading = ref( true );
  const loadError = ref( '' );
  let loadRequestSequence = 0;
  let viewActive = true;

  const gradingMode = ref( 'simple' );
  const gradingModeError = ref( '' );
  const gradingModePending = ref( false );
  const sessionGradingMode = ref( 'simple' );

  const assessmentError = ref( '' );
  const assessmentPending = ref( false );
  const pendingAssessment = ref( '' );
  const recoveryError = ref( '' );
  const undoPending = ref( false );

  const pretestPending = ref( false );
  const pendingPretestOutcome = ref( '' );

  const answerFeedbackReviewed = ref( false );
  const studyResponse = ref( '' );
  const pausedResponses = new Map();

  const studyMedia = ref([]);
  const mixedPracticeEnabled = ref( false );
  const nextDueAt = ref( null );
  const totalAvailableCards = ref( 0 );
  const matchingDueCards = ref( 0 );
  const selectedCardCount = ref( 0 );
  const sessionSelection = ref( emptyStudySelection() );
  const focusedSession = computed( () => hasStudySelection( sessionSelection.value ) );

  const sessionChangedConceptIds = ref( new Set() );
  const sessionResumeNotice = ref( '' );

  const gradingModeLocked = computed( () => {
    return (
      completedCount.value > 0
      || correctionPending.value
    ) && !isComplete.value;
  });

  const canUndoLastGrade = computed( () => (
    lastAssessmentCanBeRestored.value
    && !sessionChangedConceptIds.value.has( lastAssessment.value?.conceptId )
    && !correctionPending.value
    && !masteryStarted.value
    && !assessmentPending.value
    && !gradingModePending.value
    && !pretestPending.value
    && !undoPending.value
  ) );

  const typeAnswerSettings = computed( () => {
    if ( currentCard.value?.retrievalKind !== 'typeAnswer' ) {
      return null;
    }

    return currentCard.value.typeAnswer;
  });

  const explainSettings = computed( () => {
    if ( currentCard.value?.retrievalKind !== 'explain' ) {
      return null;
    }

    return currentCard.value.explain;
  });

  const problemSettings = computed( () => {
    if ( currentCard.value?.retrievalKind !== 'problem' ) {
      return null;
    }

    return currentCard.value.problem;
  });

  const canRevealAnswer = computed( () => (
    !typeAnswerSettings.value || Boolean( normalizeTypeAnswer( studyResponse.value ) )
  ) );

  const currentAnswerFeedback = computed( () => {
    const feedback = currentCard.value?.content.feedback;

    if (
      !richDocumentHasContent( feedback?.explanation )
      && !richDocumentHasContent( feedback?.commonMistakes )
    ) {
      return null;
    }

    return feedback;
  });

  const answerFeedbackPending = computed( () => (
    answerRevealed.value
    && Boolean( currentAnswerFeedback.value )
    && !answerFeedbackReviewed.value
  ) );

  onMounted( async () => {
    const resumableSession = takeStudySession();

    if ( resumableSession ) {
      restoreStudySession( resumableSession );
      initialLoading.value = false;
      sessionResumeNotice.value = resumableSession.changedConceptIds?.length
        ? 'Your completed session was restored. Edited concepts will use their new content next time; their earlier grades cannot be undone here.'
        : 'Your study session was restored.';
      await nextTick();
      onStateChanged();
    } else {
      void loadStudyQueue();
    }
  });

  onBeforeUnmount( () => {
    viewActive = false;
    loadRequestSequence += 1;
  });

  async function loadStudyQueue( selection = sessionSelection.value, preserveCurrent = false ) {
    if ( gradingModePending.value || assessmentPending.value || pretestPending.value || undoPending.value ) {
      return false;
    }

    const request = ++loadRequestSequence;

    clearError();
    gradingModeError.value = '';

    if ( !preserveCurrent ) {
      loadError.value = '';
    }

    initialLoading.value = true;

    try {
      const [ queue, preferences ] = await Promise.all([
        getStudyQueue( studyQuery( selection ) ),
        getDevicePreferences()
      ]);

      if ( request !== loadRequestSequence ) {
        return false;
      }

      studyMedia.value = queue.media;
      mixedPracticeEnabled.value = Boolean( queue.mixedPracticeEnabled );
      begin( queue.cards, {
        pretestingEnabled: preferences.pretestingEnabled
      });
      pausedResponses.clear();
      sessionChangedConceptIds.value = new Set();
      studyResponse.value = '';
      answerFeedbackReviewed.value = false;
      gradingMode.value = preferences.gradingMode;
      nextDueAt.value = queue.nextDueAt;
      sessionGradingMode.value = preferences.gradingMode;
      totalAvailableCards.value = queue.totalCards;
      matchingDueCards.value = queue.dueCards ?? queue.cards.length;
      selectedCardCount.value = queue.cards.length;
      sessionSelection.value = { ...selection };
      sessionResumeNotice.value = '';
      loadError.value = '';
      assessmentError.value = '';
      recoveryError.value = '';

      return true;
    } catch ( cause ) {
      if ( request === loadRequestSequence ) {
        if ( preserveCurrent ) {
          throw cause;
        }

        loadError.value = conceptLibraryErrorMessage( cause );
      }

      return false;
    } finally {
      if ( request === loadRequestSequence ) {
        initialLoading.value = false;
      }
    }
  }

  async function showAnswer() {
    if (
      !canRevealAnswer.value
      || assessmentPending.value
      || gradingModePending.value
      || pretestPending.value
      || undoPending.value
    ) {
      return;
    }

    if ( pretestActive.value ) {
      await recordCurrentPretest( 'attempted' );
      return;
    }

    answerFeedbackReviewed.value = false;
    revealAnswer();
    await nextTick();

    onAnswerRevealed();
  }

  async function skipCurrentPretest() {
    if ( !pretestActive.value || pretestPending.value ) {
      return;
    }

    await recordCurrentPretest( 'skipped' );
  }

  async function recordCurrentPretest( requestedOutcome ) {
    const card = currentCard.value;

    if (
      !card
      || !pretestActive.value
      || pretestPending.value
      || ![ 'attempted', 'skipped' ].includes( requestedOutcome )
    ) {
      return;
    }

    const response = studyResponse.value;

    assessmentError.value = '';
    pretestPending.value = true;
    pendingPretestOutcome.value = requestedOutcome;

    try {
      const pretest = await recordPretest( card.id, requestedOutcome );

      if ( !viewActive || currentCard.value?.id !== card.id ) {
        return;
      }

      answerFeedbackReviewed.value = false;

      if ( pretest.outcome === 'attempted' ) {
        beginPretestTeaching({
          pretestId: pretest.pretestId,
          response
        });
      } else {
        skipPretest({ pretestId: pretest.pretestId });
      }

      await nextTick();
      onStateChanged();
    } catch ( cause ) {
      if ( viewActive ) {
        assessmentError.value = conceptLibraryErrorMessage( cause );
      }
    } finally {
      if ( viewActive ) {
        pendingPretestOutcome.value = '';
        pretestPending.value = false;
      }
    }
  }

  async function finishCurrentPretest() {
    if ( !pretestTeachingActive.value || answerFeedbackPending.value ) {
      return;
    }

    if ( !completePretestTeaching() ) {
      return;
    }

    answerFeedbackReviewed.value = false;
    studyResponse.value = takePausedResponse( currentCard.value?.id );
    await nextTick();
    onStateChanged();
  }

  async function recordAssessment( rating ) {
    const visibleRating = gradingOptionsByMode[ gradingMode.value ].some( ( option ) => {
      return option.rating === rating;
    });

    if (
      !visibleRating
      || assessmentPending.value
      || gradingModePending.value
      || pretestPending.value
      || undoPending.value
      || answerFeedbackPending.value
      || pretestTeachingActive.value
      || !answerRevealed.value
      || !currentCard.value
    ) {
      return;
    }

    const cardId = currentCard.value.id;
    const response = studyResponse.value;

    assessmentError.value = '';
    recoveryError.value = '';
    assessmentPending.value = true;
    pendingAssessment.value = rating;

    if ( completedCount.value === 0 ) {
      sessionGradingMode.value = gradingMode.value;
    }

    try {
      const review = await recordReview( cardId, rating );

      if ( !viewActive || currentCard.value?.id !== cardId ) {
        return;
      }

      assess({
        rating,
        response,
        reviewId: review.reviewId
      });
      answerFeedbackReviewed.value = false;
      studyResponse.value = takePausedResponse( currentCard.value?.id );
      await nextTick();

      onStateChanged();
    } catch ( cause ) {
      if ( viewActive ) {
        assessmentError.value = conceptLibraryErrorMessage( cause );
      }
    } finally {
      if ( viewActive ) {
        assessmentPending.value = false;
        pendingAssessment.value = '';
      }
    }
  }

  async function beginMasteryRound() {
    if ( !startMastery() ) {
      return;
    }

    answerFeedbackReviewed.value = false;
    studyResponse.value = '';
    await nextTick();
    onStateChanged();
  }

  async function recordMasteryAssessment( recalled ) {
    if ( !masteryActionEnabled() ) {
      return;
    }

    const response = studyResponse.value;
    const outcome = recalled ? 'recalled' : 'missed';

    assessmentPending.value = true;
    pendingAssessment.value = outcome;

    try {
      if ( !assessMastery({ recalled, response }) ) {
        return;
      }

      answerFeedbackReviewed.value = false;
      studyResponse.value = '';
      await nextTick();
      onStateChanged();
    } finally {
      if ( viewActive ) {
        assessmentPending.value = false;
        pendingAssessment.value = '';
      }
    }
  }

  async function undoLastGrade() {
    const assessment = lastAssessment.value;

    if ( !assessment || !canUndoLastGrade.value ) {
      return;
    }

    const visibleCard = currentCard.value;

    if ( visibleCard ) {
      pausedResponses.set( visibleCard.id, studyResponse.value );
    }

    assessmentError.value = '';
    recoveryError.value = '';
    undoPending.value = true;

    try {
      await reverseReview( assessment.reviewId );

      if (
        !viewActive
        || lastAssessment.value?.reviewId !== assessment.reviewId
      ) {
        return;
      }

      const restored = restoreLastAssessment( assessment.reviewId );

      if ( !restored ) {
        return;
      }

      studyResponse.value = restored.response ?? '';
      answerFeedbackReviewed.value = true;
    } catch ( cause ) {
      if ( viewActive ) {
        recoveryError.value = conceptLibraryErrorMessage( cause );
      }
    } finally {
      if ( viewActive ) {
        undoPending.value = false;
      }
    }
  }

  async function updateGradingMode( nextMode ) {
    if (
      gradingModePending.value
      || assessmentPending.value
      || pretestPending.value
      || undoPending.value
      || gradingModeLocked.value
      || !gradingOptionsByMode[ nextMode ]
      || nextMode === gradingMode.value
    ) {
      return;
    }

    gradingModeError.value = '';
    gradingModePending.value = true;

    try {
      const preferences = await setGradingMode( nextMode );

      if ( !viewActive ) {
        return;
      }

      gradingMode.value = preferences.gradingMode;

      if ( completedCount.value === 0 ) {
        sessionGradingMode.value = preferences.gradingMode;
      }
    } catch ( cause ) {
      if ( viewActive ) {
        gradingModeError.value = conceptLibraryErrorMessage( cause );
      }
    } finally {
      if ( viewActive ) {
        gradingModePending.value = false;
      }
    }
  }

  function masteryActionEnabled() {
    return masteryActive.value
      && answerRevealed.value
      && !answerFeedbackPending.value
      && !assessmentPending.value
      && !gradingModePending.value
      && !undoPending.value;
  }

  async function continueToGrading() {
    if ( !answerFeedbackPending.value ) {
      return;
    }

    answerFeedbackReviewed.value = true;

    if ( pretestTeachingActive.value ) {
      await finishCurrentPretest();
      return;
    }

    await nextTick();
    onFeedbackContinued();
  }

  function takePausedResponse( cardId ) {
    if ( !cardId ) {
      return '';
    }

    const response = pausedResponses.get( cardId ) ?? '';

    pausedResponses.delete( cardId );

    return response;
  }

  function createStudySessionSnapshot() {
    return {
      changedConceptIds: [ ...sessionChangedConceptIds.value ],
      gradingMode: gradingMode.value,
      mixedPracticeEnabled: mixedPracticeEnabled.value,
      nextDueAt: nextDueAt.value,
      pausedResponses: [ ...pausedResponses.entries() ],
      recall: createSnapshot(),
      answerFeedbackReviewed: answerFeedbackReviewed.value,
      sessionGradingMode: sessionGradingMode.value,
      studyMedia: [ ...studyMedia.value ],
      totalAvailableCards: totalAvailableCards.value,
      matchingDueCards: matchingDueCards.value,
      selectedCardCount: selectedCardCount.value,
      selection: { ...sessionSelection.value },
      response: studyResponse.value
    };
  }

  function restoreStudySession( session ) {
    restoreSnapshot( session.recall );
    answerFeedbackReviewed.value = Boolean( session.answerFeedbackReviewed );
    gradingMode.value = session.gradingMode;
    mixedPracticeEnabled.value = Boolean( session.mixedPracticeEnabled );
    nextDueAt.value = session.nextDueAt;
    sessionGradingMode.value = session.sessionGradingMode;
    sessionChangedConceptIds.value = new Set( session.changedConceptIds ?? []);
    studyMedia.value = [ ...session.studyMedia ];
    totalAvailableCards.value = session.totalAvailableCards;
    matchingDueCards.value = session.matchingDueCards ?? session.totalAvailableCards;
    selectedCardCount.value = session.selectedCardCount ?? session.recall.cards.length;
    sessionSelection.value = { ...emptyStudySelection(), ...session.selection };
    studyResponse.value = session.response ?? '';
    pausedResponses.clear();

    for ( const [ cardId, response ] of session.pausedResponses ) {
      pausedResponses.set( cardId, response );
    }
  }

  return {
    answerFeedbackPending,
    answerFeedbackReviewed,
    answerRevealed,
    assessmentError,
    assessmentPending,
    beginMasteryRound,
    canRevealAnswer,
    canUndoLastGrade,
    completedCount,
    continueToGrading,
    correctionPending,
    createStudySessionSnapshot,
    currentAnswerFeedback,
    currentCard,
    explainSettings,
    finishCurrentPretest,
    focusedSession,
    gradingMode,
    gradingModeError,
    gradingModeLocked,
    gradingModePending,
    hasCards,
    initialLoading,
    isComplete,
    loadError,
    loadStudyQueue,
    masteryActionEnabled,
    masteryActive,
    masteryCompletedCount,
    masteryMissedCount,
    masteryReady,
    masteryRecalledCount,
    masteryStarted,
    masteryTotal,
    matchingDueCards,
    mixedPracticeEnabled,
    nextDueAt,
    pendingAssessment,
    pendingPretestOutcome,
    position,
    pretestActive,
    pretestAttemptedCount,
    pretestPending,
    pretestSkippedCount,
    pretestTeachingActive,
    pretestTotal,
    problemSettings,
    ratingCounts,
    recordAssessment,
    recordMasteryAssessment,
    recoveryError,
    sessionGradingMode,
    selectedCardCount,
    sessionResumeNotice,
    sessionSelection,
    showAnswer,
    skipCurrentPretest,
    studyMedia,
    studyResponse,
    totalAvailableCards,
    totalCards,
    typeAnswerSettings,
    undoLastGrade,
    undoPending,
    updateGradingMode
  };
}
