import { nextTick, onBeforeUnmount, onMounted, unref, watch } from 'vue';

import { completeStartup, isStartupPending } from '../startup';

export function useStartupReady( loading ) {
  if ( !isStartupPending() ) {
    return;
  }

  let active = true;
  let stop;

  onMounted( () => {
    stop = watch( () => unref( loading ), async ( pending ) => {
      if ( pending ) {
        return;
      }

      await nextTick();

      if ( active && !unref( loading ) ) {
        completeStartup();
        stop?.();
      }
    }, { immediate: true, flush: 'post' });
  });

  onBeforeUnmount( () => {
    active = false;
    stop?.();
  });
}
