import { invoke } from '@tauri-apps/api/core';
import { inject, onBeforeUnmount, provide, readonly, ref } from 'vue';

const AUTHORING_MEDIA = Symbol( 'authoring-media' );

export function provideAuthoringMedia() {
  const sessionId = ref( null );
  let generation = 0;

  async function close() {
    const previousSessionId = sessionId.value;

    generation += 1;
    sessionId.value = null;

    if ( previousSessionId ) {
      await endSession( previousSessionId );
    }
  }

  async function start( mediaIds ) {
    const closing = close();
    const request = generation;

    await closing;

    if ( request !== generation ) {
      return null;
    }

    const openedSessionId = await invoke( 'begin_authoring_media_session', {
      mediaIds: [ ...new Set( mediaIds ) ]
    });

    if ( request !== generation ) {
      await endSession( openedSessionId );
      return null;
    }

    sessionId.value = openedSessionId;

    return openedSessionId;
  }

  function endSession( id ) {
    return invoke( 'end_authoring_media_session', { sessionId: id });
  }

  function importImage( bytes, id ) {
    if ( !id ) {
      throw new Error( 'Reopen the editor before importing an image.' );
    }

    return invoke( 'import_image', bytes, {
      headers: { 'x-twill-media-session': id }
    });
  }

  const media = {
    close,
    importImage,
    sessionId: readonly( sessionId ),
    start
  };

  provide( AUTHORING_MEDIA, media );

  onBeforeUnmount( () => {
    // Failed release is recovered on the next native startup
    void close().catch( () => undefined );
  });

  return media;
}

export function useAuthoringMedia() {
  return inject( AUTHORING_MEDIA );
}
