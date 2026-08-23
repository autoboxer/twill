import { onBeforeUnmount, ref, watch } from 'vue';

import { useConceptLibrary } from './useConceptLibrary';

export function useMediaImage( mediaId ) {
  const { readMedia } = useConceptLibrary();

  const imageError = ref( false );
  const imageLoading = ref( true );
  const imageUrl = ref( '' );
  let loadRequestSequence = 0;

  watch( mediaId, loadImage, { immediate: true });

  onBeforeUnmount( () => {
    loadRequestSequence += 1;
    releaseImageUrl();
  });

  async function loadImage() {
    const request = ++loadRequestSequence;
    const requestedMediaId = mediaId.value;

    releaseImageUrl();
    imageError.value = false;
    imageLoading.value = true;

    if ( !requestedMediaId ) {
      imageError.value = true;
      imageLoading.value = false;
      return;
    }

    try {
      const response = await readMedia( requestedMediaId );

      if ( request !== loadRequestSequence ) {
        return;
      }

      imageUrl.value = URL.createObjectURL( new Blob([ normalizeBytes( response ) ]) );
    } catch {
      if ( request === loadRequestSequence ) {
        imageError.value = true;
      }
    } finally {
      if ( request === loadRequestSequence ) {
        imageLoading.value = false;
      }
    }
  }

  function releaseImageUrl() {
    if ( imageUrl.value ) {
      URL.revokeObjectURL( imageUrl.value );
      imageUrl.value = '';
    }
  }

  return {
    imageError,
    imageLoading,
    imageUrl
  };
}

function normalizeBytes( value ) {
  if ( value instanceof ArrayBuffer || ArrayBuffer.isView( value ) ) {
    return value;
  }

  if ( Array.isArray( value ) ) {
    return Uint8Array.from( value );
  }

  throw new TypeError( 'Media response was not binary.' );
}
