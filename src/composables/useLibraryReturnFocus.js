import { watch } from 'vue';

let returnTarget = null;

export function useLibraryReturnFocus( loading, navigationQuery ) {
  function rememberConcept( id ) {
    returnTarget = { id, query: JSON.stringify( navigationQuery.value ) };
  }

  watch( loading, ( pending ) => {
    if ( pending || !returnTarget ) {
      return;
    }

    const target = returnTarget;

    returnTarget = null;

    if ( target.query === JSON.stringify( navigationQuery.value ) ) {
      document.getElementById( `library-concept-${ target.id }` )?.focus();
    }
  }, { flush: 'post' });

  return { rememberConcept };
}
