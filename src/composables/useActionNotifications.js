import { useToast } from '@nuxt/ui/composables';
import { nextTick } from 'vue';

const pendingTitles = new Set();

export function useActionNotifications() {
  const toast = useToast();

  function notifySuccess( title ) {
    if ( !title || pendingTitles.has( title ) || toast.toasts.value.some( ( item ) => (
      item.open && item.title === title
    ) ) ) {
      return;
    }

    pendingTitles.add( title );

    toast.add({
      title,
      icon: 'i-lucide-check',
      color: 'neutral',
      type: 'background',
      orientation: 'horizontal',
      close: { 'aria-label': 'Dismiss notification', size: 'sm' }
    });

    void nextTick( () => pendingTitles.delete( title ) );
  }

  return { notifySuccess };
}
