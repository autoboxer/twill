<script setup>
import { computed, onBeforeUnmount, ref, watch } from 'vue';

import { conceptLibraryErrorMessage } from '../composables/useConceptLibrary';
import { useTemplateLibrary } from '../composables/useTemplateLibrary';
import { cloneTemplateContent, templateFields } from '../templates/defaults';
import RichContentRenderer from './RichContentRenderer.vue';

const props = defineProps({
  content: {
    type: Object,
    required: true
  }
});

const { preparePreview } = useTemplateLibrary();

const preparedCustom = ref( cloneTemplateContent().custom );
const previewError = ref( '' );
const previewPending = ref( false );
let previewRequestSequence = 0;
let previewTimer = null;

const representativeContent = {
  title: 'Cell membrane',
  prompt: {
    type: 'doc',
    content: [{
      type: 'paragraph',
      content: [{
        type: 'text',
        text: 'What controls the movement of substances into and out of a cell?'
      }]
    }]
  },
  answer: {
    type: 'doc',
    content: [{
      type: 'paragraph',
      content: [{
        type: 'text',
        text: 'The selectively permeable cell membrane.'
      }]
    }]
  }
};

const frontDocument = computed( () => customPreviewDocument(
  preparedCustom.value.frontHtml,
  preparedCustom.value.css
) );
const answerDocument = computed( () => customPreviewDocument(
  preparedCustom.value.answerHtml,
  preparedCustom.value.css
) );

watch( () => props.content, scheduleCustomPreview, {
  deep: true,
  immediate: true
});

onBeforeUnmount( () => {
  previewRequestSequence += 1;
  clearTimeout( previewTimer );
});

function scheduleCustomPreview() {
  previewRequestSequence += 1;
  clearTimeout( previewTimer );
  previewError.value = '';

  if ( props.content.mode !== 'custom' ) {
    previewPending.value = false;
    return;
  }

  const request = previewRequestSequence;

  previewPending.value = true;
  previewTimer = setTimeout( () => prepareCustomPreview( request ), 180 );
}

async function prepareCustomPreview( request ) {
  try {
    const prepared = await preparePreview( cloneTemplateContent( props.content ) );

    if ( request !== previewRequestSequence ) {
      return;
    }

    preparedCustom.value = prepared.custom;
  } catch ( cause ) {
    if ( request === previewRequestSequence ) {
      previewError.value = conceptLibraryErrorMessage( cause );
    }
  } finally {
    if ( request === previewRequestSequence ) {
      previewPending.value = false;
    }
  }
}

function fieldDetails( value ) {
  return templateFields.find( ( field ) => field.value === value );
}

function customPreviewDocument( source, css ) {
  const content = source.replace(
    /{{\s*(title|prompt|answer)\s*}}/g,
    ( _, field ) => representativeHtml( field )
  );

  return [
    '<!doctype html>',
    '<html>',
    '<head>',
    '<meta charset="utf-8">',
    '<meta name="viewport" content="width=device-width, initial-scale=1">',
    '<meta http-equiv="Content-Security-Policy" content="default-src \'none\'; style-src \'unsafe-inline\'; img-src data:; font-src \'none\'; connect-src \'none\'; media-src \'none\'; object-src \'none\'; frame-src \'none\'; base-uri \'none\'; form-action \'none\'">',
    '<style>',
    basePreviewCss(),
    css,
    '</style>',
    '</head>',
    `<body>${ content }</body>`,
    '</html>'
  ].join( '' );
}

function representativeHtml( field ) {
  const values = {
    title: '<div class="twill-field twill-title">Cell membrane</div>',
    prompt: '<div class="twill-field"><p>What controls the movement of substances into and out of a cell?</p></div>',
    answer: '<div class="twill-field"><p>The selectively permeable cell membrane.</p></div>'
  };

  return values[ field ];
}

function basePreviewCss() {
  return [
    ':root { color-scheme: light; }',
    '* { box-sizing: border-box; }',
    'html, body { min-height: 100%; margin: 0; }',
    'body { padding: 24px; color: #273029; background: #fbfcfa; font-family: system-ui, sans-serif; overflow-wrap: anywhere; }',
    '.twill-field > :first-child { margin-top: 0; }',
    '.twill-field > :last-child { margin-bottom: 0; }',
    '.twill-title { font-size: 1.3rem; font-weight: 700; }'
  ].join( '' );
}
</script>

<template>
  <section class="template-preview-section">
    <header class="template-preview-section__header">
      <div>
        <h2>Preview</h2>
        <p>Representative content shows how both sides will be arranged.</p>
      </div>

      <span
        v-if="previewPending"
        class="template-preview-status"
        role="status"
      >
        <UIcon name="i-lucide-loader-circle" />
        Updating
      </span>
    </header>

    <UAlert
      v-if="previewError"
      :description="previewError"
      icon="i-lucide-circle-alert"
      color="error"
      variant="soft"
    />

    <div class="template-preview-grid">
      <article
        v-for="side in [ 'front', 'answer' ]"
        :key="side"
        class="template-preview"
      >
        <header class="template-preview__header">
          <span>{{ side === 'front' ? 'Front' : 'Answer' }}</span>
        </header>

        <div
          v-if="content.mode === 'visual'"
          class="template-preview__visual"
          :class="`template-preview__visual--${ content.visual.appearance.alignment }`"
        >
          <div
            v-for="( block, index ) in content.visual[ side ].blocks"
            :key="`${ block.type }-${ block.field ?? index }-${ index }`"
            class="template-preview-block"
          >
            <template v-if="block.type === 'field'">
              <span
                v-if="content.visual.appearance.showFieldLabels"
                class="template-preview-block__label"
              >
                {{ fieldDetails( block.field )?.label }}
              </span>

              <strong
                v-if="block.field === 'title'"
                class="template-preview-title"
              >
                {{ representativeContent.title }}
              </strong>

              <RichContentRenderer
                v-else
                :document="representativeContent[ block.field ]"
                :label="`${ fieldDetails( block.field )?.label } preview`"
              />
            </template>

            <p
              v-else
              class="template-preview-text"
            >
              {{ block.text || 'Text block' }}
            </p>
          </div>
        </div>

        <iframe
          v-else
          :title="`${ side === 'front' ? 'Front' : 'Answer' } custom template preview`"
          :srcdoc="side === 'front' ? frontDocument : answerDocument"
          class="template-preview__frame"
          sandbox=""
          referrerpolicy="no-referrer"
          tabindex="-1"
        />
      </article>
    </div>
  </section>
</template>
