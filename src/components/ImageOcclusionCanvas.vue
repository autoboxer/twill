<script setup>
import { computed, nextTick, ref, watch } from 'vue';

import {
  createImageOcclusionId,
  imageOcclusionGroupIds
} from '../image-occlusion/documents';

const MINIMUM_REGION_SIZE = 0.01;
const KEYBOARD_STEP = 0.01;

const props = defineProps({
  alt: {
    type: String,
    default: ''
  },
  canCreate: {
    type: Boolean,
    default: true
  },
  editable: {
    type: Boolean,
    default: false
  },
  imageUrl: {
    type: String,
    required: true
  },
  newRegionGroupId: {
    type: String,
    default: ''
  },
  regions: {
    type: Array,
    default: () => []
  },
  revealed: {
    type: Boolean,
    default: false
  },
  selectedRegionId: {
    type: String,
    default: ''
  },
  visibleGroupId: {
    type: String,
    default: ''
  }
});

const emit = defineEmits([ 'update:regions', 'update:selectedRegionId' ]);

const surface = ref( null );
let pointerAction = null;

const groupNumbers = computed( () => new Map(
  imageOcclusionGroupIds( props.regions ).map( ( groupId, index ) => [ groupId, index + 1 ])
) );
const visibleRegions = computed( () => props.visibleGroupId
  ? props.regions.filter( ( region ) => region.groupId === props.visibleGroupId )
  : props.regions
);

watch(
  () => props.selectedRegionId,
  focusRegion,
  { immediate: true }
);

function regionStyle( region ) {
  return {
    height: `${ region.height * 100 }%`,
    left: `${ region.x * 100 }%`,
    top: `${ region.y * 100 }%`,
    width: `${ region.width * 100 }%`
  };
}

function regionLabel( region ) {
  const regionNumber = props.regions.findIndex( ( candidate ) => candidate.id === region.id ) + 1;
  const cardNumber = groupNumbers.value.get( region.groupId ) ?? 1;

  return `Mask ${ regionNumber }, card ${ cardNumber }`;
}

function pointerPosition( event ) {
  const bounds = surface.value.getBoundingClientRect();

  return {
    x: clamp( ( event.clientX - bounds.left ) / bounds.width, 0, 1 ),
    y: clamp( ( event.clientY - bounds.top ) / bounds.height, 0, 1 )
  };
}

function startPointerAction( event ) {
  if ( !props.editable || !surface.value ) {
    return;
  }

  const regionElement = event.target.closest?.( '[data-region-id]' );
  const handleElement = event.target.closest?.( '[data-resize-handle]' );
  const position = pointerPosition( event );
  const regions = props.regions.map( ( region ) => ({ ...region }) );

  if ( regionElement ) {
    const region = regions.find( ( candidate ) => candidate.id === regionElement.dataset.regionId );

    if ( !region ) {
      return;
    }

    emit( 'update:selectedRegionId', region.id );
    regionElement.focus({ preventScroll: true });

    pointerAction = {
      before: regions,
      handle: handleElement?.dataset.resizeHandle ?? '',
      mode: handleElement ? 'resize' : 'move',
      pointerId: event.pointerId,
      region,
      regions,
      start: position
    };
  } else {
    if ( !props.canCreate ) {
      return;
    }

    const region = {
      groupId: props.newRegionGroupId || createImageOcclusionId(),
      height: 0,
      id: createImageOcclusionId(),
      width: 0,
      x: position.x,
      y: position.y
    };

    pointerAction = {
      before: regions,
      mode: 'draw',
      pointerId: event.pointerId,
      region,
      regions: [ ...regions, region ],
      start: position
    };

    emit( 'update:regions', pointerAction.regions );
    emit( 'update:selectedRegionId', region.id );
  }

  surface.value.setPointerCapture( event.pointerId );
  event.preventDefault();
}

function updatePointerAction( event ) {
  if ( !pointerAction || event.pointerId !== pointerAction.pointerId ) {
    return;
  }

  const position = pointerPosition( event );
  const region = transformedRegion( pointerAction, position );

  pointerAction.latestRegion = region;
  emit( 'update:regions', pointerAction.regions.map( ( candidate ) => (
    candidate.id === region.id ? region : candidate
  ) ) );
  event.preventDefault();
}

function finishPointerAction( event ) {
  if ( !pointerAction || event.pointerId !== pointerAction.pointerId ) {
    return;
  }

  const action = pointerAction;
  const region = action.latestRegion ?? action.region;

  pointerAction = null;

  if (
    action.mode === 'draw'
    && ( region.width < MINIMUM_REGION_SIZE || region.height < MINIMUM_REGION_SIZE )
  ) {
    emit( 'update:regions', action.before );
    emit( 'update:selectedRegionId', '' );
  }

  event.preventDefault();
}

function cancelPointerAction( event ) {
  if ( !pointerAction || event.pointerId !== pointerAction.pointerId ) {
    return;
  }

  emit( 'update:regions', pointerAction.before );
  pointerAction = null;
}

function transformedRegion( action, position ) {
  if ( action.mode === 'draw' ) {
    return normalizedRegion({
      ...action.region,
      height: Math.abs( position.y - action.start.y ),
      width: Math.abs( position.x - action.start.x ),
      x: Math.min( action.start.x, position.x ),
      y: Math.min( action.start.y, position.y )
    });
  }

  if ( action.mode === 'move' ) {
    return normalizedRegion({
      ...action.region,
      x: clamp(
        action.region.x + position.x - action.start.x,
        0,
        1 - action.region.width
      ),
      y: clamp(
        action.region.y + position.y - action.start.y,
        0,
        1 - action.region.height
      )
    });
  }

  let left = action.region.x;
  let right = action.region.x + action.region.width;
  let top = action.region.y;
  let bottom = action.region.y + action.region.height;

  if ( action.handle.includes( 'w' ) ) {
    left = clamp( position.x, 0, right - MINIMUM_REGION_SIZE );
  }

  if ( action.handle.includes( 'e' ) ) {
    right = clamp( position.x, left + MINIMUM_REGION_SIZE, 1 );
  }

  if ( action.handle.includes( 'n' ) ) {
    top = clamp( position.y, 0, bottom - MINIMUM_REGION_SIZE );
  }

  if ( action.handle.includes( 's' ) ) {
    bottom = clamp( position.y, top + MINIMUM_REGION_SIZE, 1 );
  }

  return normalizedRegion({
    ...action.region,
    height: bottom - top,
    width: right - left,
    x: left,
    y: top
  });
}

function handleRegionKeydown( event, region ) {
  if ( !props.editable ) {
    return;
  }

  if ( event.key === 'Delete' || event.key === 'Backspace' ) {
    emit(
      'update:regions',
      props.regions.filter( ( candidate ) => candidate.id !== region.id )
    );
    emit( 'update:selectedRegionId', '' );
    event.preventDefault();
    return;
  }

  const direction = {
    ArrowDown: [ 0, KEYBOARD_STEP ],
    ArrowLeft: [ -KEYBOARD_STEP, 0 ],
    ArrowRight: [ KEYBOARD_STEP, 0 ],
    ArrowUp: [ 0, -KEYBOARD_STEP ]
  }[ event.key ];

  if ( !direction ) {
    return;
  }

  const [ xDelta, yDelta ] = direction;
  const updated = event.shiftKey
    ? normalizedRegion({
      ...region,
      height: clamp( region.height + yDelta, MINIMUM_REGION_SIZE, 1 - region.y ),
      width: clamp( region.width + xDelta, MINIMUM_REGION_SIZE, 1 - region.x )
    })
    : normalizedRegion({
      ...region,
      x: clamp( region.x + xDelta, 0, 1 - region.width ),
      y: clamp( region.y + yDelta, 0, 1 - region.height )
    });

  emit( 'update:regions', props.regions.map( ( candidate ) => (
    candidate.id === region.id ? updated : candidate
  ) ) );
  event.preventDefault();
}

function normalizedRegion( region ) {
  return {
    ...region,
    height: normalizedNumber( region.height ),
    width: normalizedNumber( region.width ),
    x: normalizedNumber( region.x ),
    y: normalizedNumber( region.y )
  };
}

function normalizedNumber( value ) {
  return Number( value.toFixed( 6 ) );
}

async function focusRegion( regionId ) {
  if ( !props.editable || !regionId ) {
    return;
  }

  await nextTick();

  const regionElements = surface.value?.querySelectorAll( '[data-region-id]' ) ?? [];
  const regionElement = [ ...regionElements ]
    .find( ( element ) => element.dataset.regionId === regionId );

  regionElement?.focus({ preventScroll: true });
}

function clamp( value, minimum, maximum ) {
  return Math.min( Math.max( value, minimum ), maximum );
}
</script>

<template>
  <div
    ref="surface"
    class="image-occlusion-canvas"
    :class="{
      'image-occlusion-canvas--editable': editable,
      'image-occlusion-canvas--revealed': revealed
    }"
    @pointerdown="startPointerAction"
    @pointermove="updatePointerAction"
    @pointerup="finishPointerAction"
    @pointercancel="cancelPointerAction"
  >
    <img
      :src="imageUrl"
      :alt="alt"
      draggable="false"
    >

    <template v-if="editable">
      <button
        v-for="region in visibleRegions"
        :key="region.id"
        type="button"
        class="image-occlusion-region image-occlusion-region--editable"
        :class="{
          'image-occlusion-region--selected': region.id === selectedRegionId
        }"
        :style="regionStyle( region )"
        :data-region-id="region.id"
        :aria-label="regionLabel( region )"
        :aria-pressed="region.id === selectedRegionId"
        @focus="emit( 'update:selectedRegionId', region.id )"
        @keydown="handleRegionKeydown( $event, region )"
      >
        <span class="image-occlusion-region__label">
          {{ groupNumbers.get( region.groupId ) }}
        </span>

        <template v-if="region.id === selectedRegionId">
          <span
            v-for="handle in [ 'nw', 'ne', 'sw', 'se' ]"
            :key="handle"
            class="image-occlusion-region__handle"
            :class="`image-occlusion-region__handle--${ handle }`"
            :data-resize-handle="handle"
            aria-hidden="true"
          />
        </template>
      </button>
    </template>

    <template v-else>
      <span
        v-for="region in visibleRegions"
        :key="region.id"
        class="image-occlusion-region"
        :style="regionStyle( region )"
        aria-hidden="true"
      />
    </template>
  </div>
</template>
