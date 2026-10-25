<script setup>
import { computed, onBeforeUnmount, ref } from 'vue';
import { imageTargetDescription } from '../image-labeling/targets';

const props = defineProps({
  imageUrl: { type: String, required: true },
  alt: { type: String, default: '' },
  target: { type: Object, default: null },
  number: { type: Number, default: 1 },
  editable: { type: Boolean, default: false }
});
const emit = defineEmits([ 'position' ]);
const surface = ref( null );
let drag = null;

const markerStyle = computed( () => props.target ? {
  left: `clamp(16px, ${ props.target.x * 100 }%, calc(100% - 16px))`,
  top: `clamp(16px, ${ props.target.y * 100 }%, calc(100% - 16px))`
} : {});

onBeforeUnmount( cancelDrag );

function position( event ) {
  const bounds = surface.value.getBoundingClientRect();

  return {
    x: coordinate( ( event.clientX - bounds.left ) / bounds.width ),
    y: coordinate( ( event.clientY - bounds.top ) / bounds.height )
  };
}

function coordinate( value ) {
  return Number( Math.max( 0, Math.min( 1, value ) ).toFixed( 6 ) );
}

function place( event ) {
  if ( props.editable && props.target ) {
    emit( 'position', position( event ) );
  }
}

function startDrag( event ) {
  if ( !props.editable || event.button !== 0 || !props.target ) {
    return;
  }

  drag = {
    element: event.currentTarget,
    pointerId: event.pointerId,
    before: { x: props.target.x, y: props.target.y }
  };
  event.currentTarget.setPointerCapture( event.pointerId );
  event.currentTarget.focus({ preventScroll: true });
  event.preventDefault();
}

function moveDrag( event ) {
  if ( drag?.pointerId === event.pointerId ) {
    emit( 'position', position( event ) );
    event.preventDefault();
  }
}

function finishDrag( event ) {
  if ( drag?.pointerId !== event.pointerId ) {
    return;
  }

  const previous = drag;

  drag = null;

  if ( previous.element.hasPointerCapture( previous.pointerId ) ) {
    previous.element.releasePointerCapture( previous.pointerId );
  }
}

function cancelDrag() {
  if ( drag ) {
    emit( 'position', drag.before );
    finishDrag({ pointerId: drag.pointerId });
  }
}

function moveByKey( event ) {
  const direction = {
    ArrowLeft: [ -1, 0 ], ArrowRight: [ 1, 0 ], ArrowUp: [ 0, -1 ], ArrowDown: [ 0, 1 ]
  }[ event.key ];

  if ( !direction || !props.editable || !props.target ) {
    return;
  }

  const step = event.shiftKey ? 0.05 : 0.01;

  emit( 'position', {
    x: coordinate( props.target.x + direction[ 0 ] * step ),
    y: coordinate( props.target.y + direction[ 1 ] * step )
  });
  event.preventDefault();
}
</script>

<template>
  <div ref="surface" class="image-label-canvas" @click="place">
    <img :src="imageUrl" :alt="alt" draggable="false">

    <template v-if="target">
      <span
        class="image-label-canvas__point"
        :style="{ left: `${ target.x * 100 }%`, top: `${ target.y * 100 }%` }"
        aria-hidden="true"
      />
      <button
        v-if="editable"
        type="button"
        class="image-label-canvas__marker"
        :style="markerStyle"
        :aria-label="imageTargetDescription( target, number )"
        @click.stop
        @pointerdown.stop="startDrag"
        @pointermove="moveDrag"
        @pointerup="finishDrag"
        @pointercancel="cancelDrag"
        @lostpointercapture="cancelDrag"
        @keydown="moveByKey"
        @keydown.esc.stop.prevent="cancelDrag"
      >
        {{ number }}
      </button>
      <span
        v-else
        class="image-label-canvas__marker"
        :style="markerStyle"
        aria-hidden="true"
      >
        {{ number }}
      </span>
    </template>
  </div>
</template>

<style scoped>
.image-label-canvas {
  position: relative;
  width: 100%;
  min-width: 32px;
}

.image-label-canvas > img {
  display: block;
  width: 100%;
  height: auto;
  max-height: none;
}

.image-label-canvas__point {
  position: absolute;
  width: 6px;
  height: 6px;
  border: 1px solid white;
  border-radius: 50%;
  background: black;
  pointer-events: none;
  transform: translate(-50%, -50%);
}

.image-label-canvas__marker {
  position: absolute;
  display: grid;
  width: 28px;
  height: 28px;
  place-items: center;
  padding: 0;
  border: 1px solid var(--ui-primary);
  border-radius: 4px;
  background: var(--ui-bg);
  color: var(--ui-text-highlighted);
  font: 650 0.8rem / 1 var(--font-sans);
  transform: translate(-50%, -50%);
}

button.image-label-canvas__marker {
  cursor: move;
  touch-action: none;
}

.image-label-canvas__marker:focus-visible {
  outline: 2px solid var(--ui-primary);
  outline-offset: 2px;
}
</style>
