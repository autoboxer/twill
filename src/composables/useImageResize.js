import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { closeHistory } from '@tiptap/pm/history';

import { imageDisplayWidth, MAXIMUM_IMAGE_WIDTH, MINIMUM_IMAGE_WIDTH } from '../rich-content/images';

export function useImageResize({ editor, getPos, node }, editingEnabled ) {
  const frame = ref( null );
  const previewWidth = ref( null );
  const widthError = ref( '' );
  const width = computed( () => imageDisplayWidth( node.value.attrs.width ) );
  const displayWidth = computed( () => previewWidth.value ?? width.value );
  let drag = null;

  watch( editingEnabled, ( enabled ) => {
    if ( !enabled ) {
      cancelResize();
    }
  });

  onBeforeUnmount( cancelResize );
  watch( width, cancelResize );

  function setWidth( value ) {
    if ( !editingEnabled.value || !editor.isEditable || value === width.value ) {
      return;
    }

    if ( value !== null && imageDisplayWidth( value ) === null ) {
      return;
    }

    const position = getPos();

    if ( typeof position !== 'number' ) {
      return;
    }

    const transaction = closeHistory( editor.state.tr ).setNodeMarkup( position, undefined, {
      ...node.value.attrs,
      width: value
    });

    editor.view.dispatch( transaction );
    editor.view.dispatch( closeHistory( editor.state.tr ) );
    widthError.value = '';
  }

  function applyWidth( event ) {
    const value = event.target.value.trim();
    const parsed = value === '' ? null : imageDisplayWidth( Number( value ) );

    if ( event.target.validity.badInput || ( value !== '' && parsed === null ) ) {
      widthError.value = `Use a width from ${ MINIMUM_IMAGE_WIDTH } to ${ MAXIMUM_IMAGE_WIDTH } pixels.`;
    } else {
      widthError.value = '';
      setWidth( parsed );
    }

    event.target.value = width.value ?? '';
  }

  function startResize( event ) {
    if ( drag || !editingEnabled.value || event.button !== 0 || !frame.value ) {
      return;
    }

    event.preventDefault();
    event.stopPropagation();
    event.currentTarget.focus({ preventScroll: true });
    event.currentTarget.setPointerCapture( event.pointerId );
    drag = {
      target: event.currentTarget,
      pointerId: event.pointerId,
      startX: event.clientX,
      startWidth: frame.value.getBoundingClientRect().width,
      limit: Math.min( MAXIMUM_IMAGE_WIDTH, frame.value.parentElement.clientWidth )
    };
  }

  function resizeBy( delta ) {
    if ( !frame.value ) {
      return;
    }

    cancelResize();

    setWidth( Math.round( Math.max( MINIMUM_IMAGE_WIDTH, Math.min(
      MAXIMUM_IMAGE_WIDTH, ( width.value ?? frame.value.getBoundingClientRect().width ) + delta
    ) ) ) );
  }

  function moveResize( event ) {
    if ( !drag || drag.pointerId !== event.pointerId ) {
      return;
    }

    // The centered image grows equally on both sides
    previewWidth.value = Math.round( Math.max( MINIMUM_IMAGE_WIDTH, Math.min(
      drag.limit, drag.startWidth + ( event.clientX - drag.startX ) * 2
    ) ) );
  }

  function finishResize( event ) {
    if ( !drag || drag.pointerId !== event.pointerId ) {
      return;
    }

    const value = previewWidth.value;

    cancelResize();

    if ( value !== null ) {
      setWidth( value );
    }
  }

  function cancelResize() {
    const previous = drag;

    drag = null;
    previewWidth.value = null;

    if ( previous?.target.hasPointerCapture( previous.pointerId ) ) {
      previous.target.releasePointerCapture( previous.pointerId );
    }
  }

  return {
    applyWidth,
    cancelResize,
    displayWidth,
    finishResize,
    frame,
    moveResize,
    resizeBy,
    setWidth,
    startResize,
    width,
    widthError
  };
}
