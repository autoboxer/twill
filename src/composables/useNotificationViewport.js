import { onBeforeUnmount, onMounted, ref } from 'vue';

export function useNotificationViewport() {
  const position = ref( 'bottom-right' );
  const viewport = window.visualViewport;

  function updateViewport() {
    position.value = window.innerWidth >= 1024 ? 'bottom-right' : 'bottom-center';

    const bottomInset = viewport
      ? Math.max( 0, window.innerHeight - viewport.height - viewport.offsetTop )
      : 0;

    document.documentElement.style.setProperty(
      '--notification-bottom-inset',
      `${ bottomInset }px`
    );
  }

  onMounted( () => {
    updateViewport();
    window.addEventListener( 'resize', updateViewport );
    viewport?.addEventListener( 'resize', updateViewport );
    viewport?.addEventListener( 'scroll', updateViewport );
  });

  onBeforeUnmount( () => {
    window.removeEventListener( 'resize', updateViewport );
    viewport?.removeEventListener( 'resize', updateViewport );
    viewport?.removeEventListener( 'scroll', updateViewport );
    document.documentElement.style.removeProperty( '--notification-bottom-inset' );
  });

  return { position };
}
