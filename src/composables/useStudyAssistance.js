import { ref } from 'vue';

import { assistanceDocuments } from '../study/assistance';

export function useStudyAssistance({
  currentCard,
  answerRevealed,
  actionsBlocked,
  pretestActive,
  response
}) {
  const revealedAssistance = ref([]);
  const assisted = ref( false );
  const responseBeforeAssistance = ref( '' );

  function assistanceSnapshot() {
    return {
      revealed: [ ...revealedAssistance.value ],
      used: assisted.value,
      response: responseBeforeAssistance.value
    };
  }

  function restoreAssistance( saved = null ) {
    revealedAssistance.value = [ ...( saved?.revealed ?? []) ];
    assisted.value = saved?.used === true;
    responseBeforeAssistance.value = saved?.response ?? '';
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

    if ( !answerRevealed.value && !assisted.value ) {
      assisted.value = true;
      responseBeforeAssistance.value = response.value;
    }

    revealedAssistance.value = [ ...revealedAssistance.value, id ];
  }

  return {
    assisted,
    assistanceSnapshot,
    responseBeforeAssistance,
    restoreAssistance,
    revealedAssistance,
    toggleAssistance
  };
}
