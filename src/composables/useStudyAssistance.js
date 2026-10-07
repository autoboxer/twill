import { ref } from 'vue';

import { assistanceDocuments } from '../study/assistance';
import { collectAnswerParts, supportsAnswerParts } from '../answer-parts/documents';

export function useStudyAssistance({
  currentCard,
  answerRevealed,
  actionsBlocked,
  pretestActive,
  response
}) {
  const revealedAssistance = ref([]);
  const revealedAnswerParts = ref([]);
  const assisted = ref( false );
  const responseBeforeAssistance = ref( '' );

  function assistanceSnapshot() {
    return {
      revealed: [ ...revealedAssistance.value ],
      parts: [ ...revealedAnswerParts.value ],
      used: assisted.value,
      response: responseBeforeAssistance.value
    };
  }

  function restoreAssistance( saved = null ) {
    revealedAssistance.value = [ ...( saved?.revealed ?? []) ];
    const parts = collectAnswerParts( currentCard.value?.content.answer );

    revealedAnswerParts.value = ( saved?.parts ?? []).filter( ( id ) => parts.some( ( part ) => part.id === id ) );
    assisted.value = saved?.used === true;
    responseBeforeAssistance.value = saved?.response ?? '';
  }

  function markAssistance() {
    if ( !answerRevealed.value && !assisted.value ) {
      assisted.value = true;
      responseBeforeAssistance.value = response.value;
    }
  }

  function toggleAnswerParts( ids ) {
    const parts = collectAnswerParts( currentCard.value?.content.answer );

    if ( actionsBlocked.value || pretestActive.value || answerRevealed.value
      || !supportsAnswerParts( currentCard.value ) || !Array.isArray( ids ) || !ids.length
      || ids.some( ( id ) => !parts.some( ( part ) => part.id === id ) ) ) {
      return;
    }

    if ( ids.every( ( id ) => revealedAnswerParts.value.includes( id ) ) ) {
      revealedAnswerParts.value = revealedAnswerParts.value.filter( ( id ) => !ids.includes( id ) );
    } else {
      markAssistance();
      revealedAnswerParts.value = [ ...new Set([ ...revealedAnswerParts.value, ...ids ]) ];
    }
  }

  function toggleAssistance( id ) {
    if ( actionsBlocked.value || pretestActive.value
      || !assistanceDocuments( currentCard.value?.content ).some( ( item ) => item.id === id ) ) {
      return;
    }

    if ( revealedAssistance.value.includes( id ) ) {
      revealedAssistance.value = revealedAssistance.value.filter( ( value ) => value !== id );
      return;
    }

    markAssistance();

    revealedAssistance.value = [ ...revealedAssistance.value, id ];
  }

  return {
    assisted,
    assistanceSnapshot,
    responseBeforeAssistance,
    restoreAssistance,
    revealedAssistance,
    revealedAnswerParts,
    toggleAnswerParts,
    toggleAssistance
  };
}
