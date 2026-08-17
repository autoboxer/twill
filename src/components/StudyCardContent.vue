<script setup>
import { AnimatePresence, m } from 'motion-v';
import { computed, onBeforeUnmount, ref, watch } from 'vue';

import { useConceptLibrary } from '../composables/useConceptLibrary';
import { createCustomTemplateDocument } from '../templates/customFrame';
import { templateFields } from '../templates/defaults';
import { createStudyTemplateFields } from '../templates/studyFields';
import RichContentRenderer from './RichContentRenderer.vue';

const props = defineProps({
  answerRevealed: {
    type: Boolean,
    required: true
  },

  card: {
    type: Object,
    required: true
  },

  media: {
    type: Array,
    required: true
  }
});

const { readMedia } = useConceptLibrary();

const customDocuments = ref({ answer: '', front: '' });
const customPending = ref( false );
const mediaWarning = ref( false );
const renderError = ref( '' );
const root = ref( null );
let renderRequestSequence = 0;

const side = computed( () => props.answerRevealed ? 'answer' : 'front' );
const isStandard = computed( () => !props.card.template );
const isVisual = computed( () => {
  return props.card.template?.content.mode === 'visual';
});
const visualAppearance = computed( () => {
  if ( isStandard.value ) {
    return {
      alignment: 'left',
      showFieldLabels: true
    };
  }

  return props.card.template.content.visual.appearance;
});
const visibleBlocks = computed( () => {
  if ( isStandard.value ) {
    return props.answerRevealed
      ? [
        { type: 'field', field: 'prompt' },
        { type: 'field', field: 'answer' }
      ]
      : [{ type: 'field', field: 'prompt' }];
  }

  return props.card.template.content.visual[ side.value ].blocks;
});

watch( () => props.card, prepareCustomDocuments, { immediate: true });

onBeforeUnmount( () => {
  renderRequestSequence += 1;
});

defineExpose({ focus });

function fieldDetails( value ) {
  return templateFields.find( ( field ) => field.value === value );
}

function fieldValue( field ) {
  if ( field === 'title' ) {
    return props.card.conceptTitle;
  }

  return props.card.content[ field ];
}

function focus() {
  root.value?.focus();
}

async function prepareCustomDocuments() {
  const request = ++renderRequestSequence;
  const card = props.card;
  const custom = card.template?.content.mode === 'custom'
    ? card.template.content.custom
    : null;

  customDocuments.value = { answer: '', front: '' };
  customPending.value = Boolean( custom );
  mediaWarning.value = false;
  renderError.value = '';

  if ( !custom ) {
    return;
  }

  try {
    const mediaById = new Map( props.media.map( ( media ) => [ media.id, media ]) );
    const mediaIds = customMediaIds( card, custom );
    const mediaResults = await Promise.all( mediaIds.map( async ( mediaId ) => {
      const media = mediaById.get( mediaId );

      if ( !media ) {
        return { id: mediaId, url: '' };
      }

      try {
        const bytes = await readMedia( mediaId );
        const url = await bytesToDataUrl( bytes, media.mimeType );

        return { id: mediaId, url };
      } catch {
        return { id: mediaId, url: '' };
      }
    }) );

    if ( request !== renderRequestSequence ) {
      return;
    }

    const mediaUrls = new Map(
      mediaResults
        .filter( ( media ) => media.url )
        .map( ( media ) => [ media.id, media.url ])
    );
    const fields = createStudyTemplateFields( card, mediaUrls );

    mediaWarning.value = mediaResults.some( ( media ) => !media.url );
    customDocuments.value = {
      front: createCustomTemplateDocument( custom.frontHtml, custom.css, fields ),
      answer: createCustomTemplateDocument( custom.answerHtml, custom.css, fields )
    };
  } catch {
    if ( request === renderRequestSequence ) {
      renderError.value = 'This retrieval form could not be rendered.';
    }
  } finally {
    if ( request === renderRequestSequence ) {
      customPending.value = false;
    }
  }
}

function customMediaIds( card, custom ) {
  const sources = `${ custom.frontHtml }\n${ custom.answerHtml }`;
  const ids = new Set();

  for ( const field of [ 'prompt', 'answer' ]) {
    const fieldPattern = new RegExp( `{{\\s*${ field }\\s*}}` );

    if ( fieldPattern.test( sources ) ) {
      collectMediaIds( card.content[ field ], ids );
    }
  }

  return [ ...ids ];
}

function collectMediaIds( node, ids ) {
  if ( node.type === 'mediaImage' && node.attrs?.mediaId ) {
    ids.add( node.attrs.mediaId );
  }

  for ( const child of node.content ?? []) {
    collectMediaIds( child, ids );
  }
}

function bytesToDataUrl( value, mimeType ) {
  const bytes = normalizeBytes( value );
  const blob = new Blob([ bytes ], { type: mimeType });

  return new Promise( ( resolve, reject ) => {
    const reader = new FileReader();

    reader.addEventListener( 'load', () => {
      if ( typeof reader.result === 'string' ) {
        resolve( reader.result );
      } else {
        reject( new TypeError( 'Media could not be encoded.' ) );
      }
    }, { once: true });
    reader.addEventListener( 'error', () => reject( reader.error ), { once: true });
    reader.readAsDataURL( blob );
  });
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
</script>

<template>
  <div
    ref="root"
    class="study-template"
    :class="{ 'study-template--standard': isStandard }"
    :aria-label="`${ side === 'front' ? 'Front' : 'Answer' } of ${ card.conceptTitle }`"
    role="group"
    tabindex="-1"
  >
    <AnimatePresence
      mode="wait"
      :initial="false"
    >
      <m.div
        v-if="isStandard || isVisual"
        :key="`${ card.id }-${ side }-visual`"
        class="study-template__visual"
        :class="`study-template__visual--${ visualAppearance.alignment }`"
        :initial="{ opacity: 0, y: 10 }"
        :animate="{ opacity: 1, y: 0 }"
        :exit="{ opacity: 0, y: -6 }"
      >
        <section
          v-for="( block, index ) in visibleBlocks"
          :key="`${ block.type }-${ block.field ?? index }-${ index }`"
          class="study-template-block"
        >
          <template v-if="block.type === 'field'">
            <span
              v-if="visualAppearance.showFieldLabels"
              class="study-template-block__label"
            >
              {{ fieldDetails( block.field )?.label }}
            </span>

            <strong
              v-if="block.field === 'title'"
              class="study-template-title"
            >
              {{ fieldValue( block.field ) }}
            </strong>

            <RichContentRenderer
              v-else
              :document="fieldValue( block.field )"
              :label="`${ fieldDetails( block.field )?.label } for ${ card.conceptTitle }`"
            />
          </template>

          <p
            v-else
            class="study-template-text"
          >
            {{ block.text }}
          </p>
        </section>
      </m.div>

      <m.div
        v-else
        :key="`${ card.id }-${ side }-custom`"
        class="study-template__custom"
        :initial="{ opacity: 0, y: 10 }"
        :animate="{ opacity: 1, y: 0 }"
        :exit="{ opacity: 0, y: -6 }"
      >
        <div
          v-if="customPending"
          class="study-template__state"
          role="status"
        >
          <UIcon name="i-lucide-loader-circle" />
          <span>Loading retrieval form</span>
        </div>

        <UAlert
          v-else-if="renderError"
          :description="renderError"
          icon="i-lucide-circle-alert"
          color="error"
          variant="soft"
        />

        <template v-else>
          <UAlert
            v-if="mediaWarning"
            class="study-template__warning"
            description="One or more images could not be loaded."
            icon="i-lucide-image-off"
            color="warning"
            variant="soft"
          />

          <iframe
            :title="`${ side === 'front' ? 'Front' : 'Answer' } for ${ card.conceptTitle }`"
            :srcdoc="customDocuments[ side ]"
            class="study-template__frame"
            sandbox=""
            referrerpolicy="no-referrer"
            tabindex="0"
          />
        </template>
      </m.div>
    </AnimatePresence>
  </div>
</template>
