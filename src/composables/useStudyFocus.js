import { ref, watch } from 'vue';

export function useStudyFocus( session ) {
  const {
    answerFeedbackPending,
    answerRevealed,
    correctionPending,
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

  watch([
    completionHeading,
    masteryHeading,
    revealButton,
    studyContent,
    typeAnswerResponse,
    explainResponse,
    problemResponse
  ], ( elements, previous ) => {
    if ( elements.some( ( element, index ) => element && element !== previous[ index ]) ) {
      focusCurrentState();
    }
  }, { flush: 'post' });

  watch([
    session.assessmentPending,
    session.pretestPending,
    session.undoPending
  ], ( pending, previous ) => {
    if ( previous.some( Boolean ) && !pending.some( Boolean ) ) {
      focusCurrentState();
    }
  }, { flush: 'post' });

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
    gradingActions.value?.querySelector( 'button:not(:disabled)' )?.focus();
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
    focusRevealedAnswer,
    gradingActions,
    masteryHeading,
    problemResponse,
    revealButton,
    studyContent,
    typeAnswerResponse
  };
}
