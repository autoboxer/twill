<script setup>
import { NodeViewContent, NodeViewWrapper } from '@tiptap/vue-3';
import { closeHistory } from '@tiptap/pm/history';
import { computed, ref, watch } from 'vue';

import { codeLanguageItems, normalizeCodeLanguage } from '../rich-content/codeLanguages';

const props = defineProps({
  editor: {
    type: Object,
    required: true
  },
  extension: {
    type: Object,
    required: true
  },
  node: {
    type: Object,
    required: true
  },
  getPos: {
    type: Function,
    required: true
  }
});

const language = computed( () => normalizeCodeLanguage( props.node.attrs.language ) );
const editingEnabled = computed( () => props.extension.options.editingEnabled() );
const menuOpen = ref( false );
const searchTerm = ref( '' );
const matchingLanguages = computed( () => {
  const query = searchTerm.value.trim().toLowerCase();
  const matches = codeLanguageItems.filter( item => [ item.label, item.value, item.aliases ]
    .some( text => text?.toLowerCase().includes( query ) ) );

  return matches.length ? matches : codeLanguageItems.filter( item => item.value === 'plaintext' );
});

watch( editingEnabled, ( enabled ) => {
  if ( !enabled ) {
    menuOpen.value = false;
  }
});

function setLanguage( value ) {
  if ( !editingEnabled.value || !props.editor.isEditable || value === language.value
    || !codeLanguageItems.some( item => item.value === value ) ) {
    return;
  }

  const position = props.getPos();

  if ( typeof position !== 'number' ) {
    return;
  }

  const { editor } = props;
  const transaction = closeHistory( editor.state.tr ).setNodeMarkup( position, undefined, {
    ...props.node.attrs,
    language: value
  });

  // Keep language changes separate from adjacent typing in undo history
  editor.view.dispatch( transaction );
  editor.view.dispatch( closeHistory( editor.state.tr ) );
}
</script>

<template>
  <NodeViewWrapper class="rich-code-block">
    <div
      class="rich-code-block__header"
      contenteditable="false"
    >
      <USelectMenu
        v-model:open="menuOpen"
        v-model:search-term="searchTerm"
        :model-value="language"
        :items="matchingLanguages"
        ignore-filter
        :disabled="!editingEnabled"
        :search-input="{ placeholder: 'Find language', 'aria-label': 'Find code language' }"
        :content="{ align: 'end' }"
        value-key="value"
        aria-label="Code language"
        color="neutral"
        variant="ghost"
        size="xs"
        class="rich-code-block__language"
        @update:model-value="setLanguage"
      />
    </div>

    <pre><NodeViewContent
      as="code"
      :class="`language-${ language }`"
      style="white-space: pre"
    /></pre>
  </NodeViewWrapper>
</template>
