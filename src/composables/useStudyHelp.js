import { inject, provide, ref, shallowRef } from 'vue';

const STUDY_HELP_KEY = Symbol( 'study-help' );

export function provideStudyHelp() {
  const isOpen = ref( false );
  const requested = ref( false );
  const topicId = ref( 'start' );
  const returnTarget = shallowRef( null );

  const help = {
    isOpen,
    requested,
    topicId,
    show( topic = 'start', target = document.activeElement ) {
      topicId.value = topic;
      returnTarget.value = target;
      requested.value = true;
      isOpen.value = true;
    },
    restoreFocus( event ) {
      event.preventDefault();

      if ( isOpen.value ) {
        return;
      }

      const target = returnTarget.value;

      if ( target instanceof HTMLElement && target.isConnected && target.getClientRects().length ) {
        target.focus({ preventScroll: true });
      } else {
        document.getElementById( 'main-content' )?.focus({ preventScroll: true });
      }
    }
  };

  provide( STUDY_HELP_KEY, help );

  return help;
}

export function useStudyHelp() {
  const help = inject( STUDY_HELP_KEY );

  if ( !help ) {
    throw new Error( 'Study help is not available.' );
  }

  return help;
}
