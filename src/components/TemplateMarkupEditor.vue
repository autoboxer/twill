<script setup>
import { onBeforeUnmount, onMounted, ref, useId, watch } from 'vue';
import { Compartment, EditorState } from '@codemirror/state';
import { EditorView, keymap } from '@codemirror/view';
import {
  defaultKeymap,
  history,
  historyKeymap,
  isolateHistory,
  redo,
  redoDepth,
  undo,
  undoDepth
} from '@codemirror/commands';

import { templateFields } from '../templates/defaults';

const props = defineProps({
  description: {
    type: String,
    default: ''
  },
  disabled: {
    type: Boolean,
    default: false
  },
  error: {
    type: String,
    default: ''
  },
  label: {
    type: String,
    required: true
  },
  modelValue: {
    type: String,
    required: true
  },
  rows: {
    type: Number,
    default: 10
  },
  showFields: {
    type: Boolean,
    default: true
  }
});

const emit = defineEmits([ 'update:modelValue' ]);

const input = ref( null );
const canUndo = ref( false );
const canRedo = ref( false );
const inputId = useId();
const configuration = new Compartment();
let editor = null;

onMounted( () => {
  editor = new EditorView({
    parent: input.value,
    state: createState()
  });
});

onBeforeUnmount( () => editor?.destroy() );

watch( () => props.modelValue, ( value ) => {
  if ( editor && value !== editor.state.doc.toString() ) {
    // External content is a new editing baseline, not an undoable user change
    editor.setState( createState() );
    updateHistory();
  }
});

watch( () => [ props.disabled, props.error, props.label ], () => {
  editor?.dispatch({ effects: configuration.reconfigure( editorConfiguration() ) });
});

function editorConfiguration() {
  return [
    EditorState.readOnly.of( props.disabled ),
    EditorView.editable.of( !props.disabled ),
    EditorView.contentAttributes.of({
      id: inputId,
      'aria-label': props.label,
      'aria-invalid': String( Boolean( props.error ) ),
      'aria-describedby': props.error ? `${ inputId }-error` : '',
      'aria-disabled': String( props.disabled ),
      spellcheck: 'false'
    })
  ];
}

function createState() {
  return EditorState.create({
    doc: props.modelValue,
    extensions: [
      history(),
      EditorState.tabSize.of( 2 ),
      keymap.of([
        ...historyKeymap.filter( ( binding ) => binding.run === undo || binding.run === redo ),
        ...defaultKeymap
      ]),
      EditorView.lineWrapping,
      configuration.of( editorConfiguration() ),
      EditorView.updateListener.of( ( update ) => {
        updateHistory();

        if ( update.docChanged ) {
          emit( 'update:modelValue', update.state.doc.toString() );
        }
      })
    ]
  });
}

function updateHistory() {
  canUndo.value = Boolean( editor && undoDepth( editor.state ) );
  canRedo.value = Boolean( editor && redoDepth( editor.state ) );
}

function editHistory( command ) {
  if ( !editor || props.disabled || editor.composing ) {
    return;
  }

  command( editor );
  editor.focus();
}

function insertField( field ) {
  if ( !editor || props.disabled || editor.composing ) {
    return;
  }

  editor.dispatch({
    ...editor.state.replaceSelection( `{{ ${ field } }}` ),
    annotations: isolateHistory.of( 'full' ),
    userEvent: 'input',
    scrollIntoView: true
  });
  editor.focus();
}
</script>

<template>
  <div class="template-markup-editor">
    <div class="template-markup-editor__heading">
      <div>
        <span class="template-markup-editor__label">{{ label }}</span>
        <p v-if="description">{{ description }}</p>
      </div>

      <div
        class="template-token-buttons"
        role="group"
        :aria-label="`${ label } editing actions`"
      >
        <UButton
          type="button"
          icon="i-lucide-undo-2"
          :aria-label="`Undo ${ label }`"
          title="Undo"
          color="neutral"
          variant="ghost"
          size="xs"
          :disabled="disabled || !canUndo"
          @click="editHistory( undo )"
        />
        <UButton
          type="button"
          icon="i-lucide-redo-2"
          :aria-label="`Redo ${ label }`"
          title="Redo"
          color="neutral"
          variant="ghost"
          size="xs"
          :disabled="disabled || !canRedo"
          @click="editHistory( redo )"
        />
        <UButton
          v-for="field in showFields ? templateFields : []"
          :key="field.value"
          type="button"
          :label="field.label"
          :leading-icon="field.icon"
          color="neutral"
          variant="ghost"
          size="xs"
          :disabled="disabled"
          @click="insertField( field.value )"
        />
      </div>
    </div>

    <div
      ref="input"
      :class="{ 'template-code-input--error': error }"
      :style="{ '--template-editor-rows': rows }"
      class="template-code-input"
    />

    <p
      v-if="error"
      :id="`${ inputId }-error`"
      class="template-field-error"
    >
      {{ error }}
    </p>
  </div>
</template>
