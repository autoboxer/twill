import { ref } from 'vue';

export function useStudyFocus( session ) {
  const {
    answerFeedbackPending,
    answerFeedbackReviewed,
    answerRevealed,
    correctionPending,
    currentAnswerFeedback,
    currentCard,
    explainSettings,
    isComplete,
    masteryReady,
    pretestTeachingActive,
    problemSettings,
    typeAnswerSettings
  } = session;

  const answerFeedback = ref( null );

  const completionHeading = ref( null );

  const gradingActions = ref( null );

  const masteryHeading = ref( null );

  const problemResponse = ref( null );

  const explainResponse = ref( null );

  const revealButton = ref( null );

  const studyContent = ref( null );

  const typeAnswerResponse = ref( null );

  function focusCurrentState() {
    if ( document.querySelector( '[role="dialog"]' ) ) {
      return;
    }

    if ( masteryReady.value ) {
      masteryHeading.value?.focus();
      return;
    }

    if ( isComplete.value ) {
      completionHeading.value?.focus();
      return;
    }

    if ( !currentCard.value ) {
      return;
    }

    if ( answerRevealed.value ) {
      if ( answerFeedbackPending.value ) {
        answerFeedback.value?.focus();
        return;
      }

      if ( pretestTeachingActive.value ) {
        focusRevealedAnswer();
        return;
      }

      if ( correctionPending.value ) {
        focusRevealedAnswer();
        return;
      }

      focusFirstGradingAction();
      return;
    }

    if ( typeAnswerSettings.value ) {
      typeAnswerResponse.value?.focus();
    } else if ( explainSettings.value ) {
      explainResponse.value?.focus();
    } else if ( problemSettings.value ) {
      problemResponse.value?.focus();
    } else {
      focusButton( revealButton.value );
    }
  }

  function focusFirstGradingAction() {
    const element = gradingActions.value?.$el ?? gradingActions.value;

    element?.querySelector( 'button:not(:disabled)' )?.focus();
  }

  function focusGradingAfterFeedback() {
    if (
      answerFeedbackReviewed.value
      && currentAnswerFeedback.value
      && !correctionPending.value
      && !pretestTeachingActive.value
    ) {
      focusFirstGradingAction();
    }
  }

  function focusRevealedAnswer() {
    if ( typeAnswerSettings.value ) {
      typeAnswerResponse.value?.focus();
    } else if ( explainSettings.value ) {
      explainResponse.value?.focus();
    } else if ( problemSettings.value ) {
      problemResponse.value?.focus();
    } else {
      studyContent.value?.focus();
    }
  }

  function focusButton( button ) {
    const element = button?.$el ?? button;

    element?.focus();
  }

  function focusAnswer() {
    if ( answerFeedbackPending.value ) {
      answerFeedback.value?.focus();
    } else {
      focusRevealedAnswer();
    }
  }

  return {
    answerFeedback,
    completionHeading,
    explainResponse,
    focusAnswer,
    focusCurrentState,
    focusFirstGradingAction,
    focusGradingAfterFeedback,
    focusRevealedAnswer,
    gradingActions,
    masteryHeading,
    problemResponse,
    revealButton,
    studyContent,
    typeAnswerResponse
  };
}
