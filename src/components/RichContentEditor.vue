<script setup>
import { computed, ref, shallowRef } from 'vue';

import {
  conceptLibraryErrorMessage,
  useConceptLibrary
} from '../composables/useConceptLibrary';
import {
  codeLanguageItems,
  createRichContentExtensions,
  richContentStarterKit
} from '../rich-content/schema';

const props = defineProps({
  label: {
    type: String,
    required: true
  },
  modelValue: {
    type: Object,
    required: true
  },
  placeholder: {
    type: String,
    default: ''
  }
});

const emit = defineEmits([ 'update:modelValue' ]);

const { importImage } = useConceptLibrary();

const activeCodeLanguage = ref( 'auto' );
const codeBlockActive = ref( false );
const currentEditor = shallowRef( null );
const fileInput = ref( null );
const imageError = ref( '' );
const imageImporting = ref( false );
const linkDialogOpen = ref( false );
const linkDraft = ref( '' );
const linkSubmitted = ref( false );
const mathDialogOpen = ref( false );
const mathDraft = ref( '' );
const mathMode = ref( 'inline' );
const mathPosition = ref( null );
const mathSubmitted = ref( false );

const document = computed({
  get: () => props.modelValue,
  set: ( value ) => emit( 'update:modelValue', value )
});
const extensions = createRichContentExtensions({ onEditMath });
const starterKit = richContentStarterKit( true );
const linkError = computed( () => {
  if ( !linkSubmitted.value ) {
    return '';
  }

  const link = linkDraft.value.trim();

  if ( !link ) {
    return 'Enter a link.';
  }

  if ( new TextEncoder().encode( link ).length > 2_048
    || !/^(https?:\/\/|mailto:)/i.test( link ) ) {
    return 'Use an http, https, or mailto link.';
  }

  if ( Array.from( link ).some( ( character ) => {
    const codePoint = character.codePointAt( 0 );

    return codePoint <= 31 || codePoint === 127;
  }) ) {
    return 'The link contains an invalid character.';
  }

  return '';
});
const mathError = computed( () => {
  if ( !mathSubmitted.value ) {
    return '';
  }

  if ( !mathDraft.value.trim() ) {
    return 'Enter a LaTeX equation.';
  }

  if ( Array.from( mathDraft.value ).length > 10_000 ) {
    return 'The equation is too long.';
  }

  return '';
});
const toolbarItems = [
  [
    {
      kind: 'undo',
      icon: 'i-lucide-undo-2',
      'aria-label': 'Undo',
      tooltip: { text: 'Undo' }
    },
    {
      kind: 'redo',
      icon: 'i-lucide-redo-2',
      'aria-label': 'Redo',
      tooltip: { text: 'Redo' }
    }
  ],
  [
    {
      icon: 'i-lucide-pilcrow',
      'aria-label': 'Text style',
      tooltip: { text: 'Text style' },
      items: [
        {
          kind: 'paragraph',
          label: 'Paragraph',
          icon: 'i-lucide-pilcrow'
        },
        {
          kind: 'heading',
          level: 1,
          label: 'Heading 1',
          icon: 'i-lucide-heading-1'
        },
        {
          kind: 'heading',
          level: 2,
          label: 'Heading 2',
          icon: 'i-lucide-heading-2'
        },
        {
          kind: 'heading',
          level: 3,
          label: 'Heading 3',
          icon: 'i-lucide-heading-3'
        }
      ]
    },
    {
      kind: 'bulletList',
      icon: 'i-lucide-list',
      'aria-label': 'Bullet list',
      tooltip: { text: 'Bullet list' }
    },
    {
      kind: 'orderedList',
      icon: 'i-lucide-list-ordered',
      'aria-label': 'Numbered list',
      tooltip: { text: 'Numbered list' }
    },
    {
      kind: 'blockquote',
      icon: 'i-lucide-text-quote',
      'aria-label': 'Block quote',
      tooltip: { text: 'Block quote' }
    }
  ],
  [
    {
      kind: 'mark',
      mark: 'bold',
      icon: 'i-lucide-bold',
      'aria-label': 'Bold',
      tooltip: { text: 'Bold' }
    },
    {
      kind: 'mark',
      mark: 'italic',
      icon: 'i-lucide-italic',
      'aria-label': 'Italic',
      tooltip: { text: 'Italic' }
    },
    {
      kind: 'mark',
      mark: 'underline',
      icon: 'i-lucide-underline',
      'aria-label': 'Underline',
      tooltip: { text: 'Underline' }
    },
    {
      kind: 'mark',
      mark: 'strike',
      icon: 'i-lucide-strikethrough',
      'aria-label': 'Strikethrough',
      tooltip: { text: 'Strikethrough' }
    },
    {
      kind: 'mark',
      mark: 'code',
      icon: 'i-lucide-code',
      'aria-label': 'Inline code',
      tooltip: { text: 'Inline code' }
    },
    {
      kind: 'link',
      icon: 'i-lucide-link',
      'aria-label': 'Link',
      tooltip: { text: 'Link' }
    }
  ],
  [
    {
      kind: 'codeBlock',
      icon: 'i-lucide-square-code',
      'aria-label': 'Code block',
      tooltip: { text: 'Code block' }
    },
    {
      kind: 'horizontalRule',
      icon: 'i-lucide-minus',
      'aria-label': 'Divider',
      tooltip: { text: 'Divider' }
    },
    {
      kind: 'clearFormatting',
      icon: 'i-lucide-remove-formatting',
      'aria-label': 'Clear formatting',
      tooltip: { text: 'Clear formatting' }
    }
  ]
];
const editorHandlers = {
  link: {
    canExecute: ( editor ) => editor.can().setLink({ href: 'https://example.com' })
      || editor.can().unsetLink(),
    execute: ( editor ) => {
      openLinkDialog( editor );
      return editor.chain();
    },
    isActive: ( editor ) => editor.isActive( 'link' ),
    isDisabled: ( editor ) => editor.state.selection.empty && !editor.isActive( 'link' )
  }
};

function syncEditorState({ editor }) {
  currentEditor.value = editor;
  codeBlockActive.value = editor.isActive( 'codeBlock' );
  activeCodeLanguage.value = editor.getAttributes( 'codeBlock' ).language ?? 'auto';
}

function setCodeLanguage( editor, language ) {
  editor
    .chain()
    .focus()
    .updateAttributes( 'codeBlock', {
      language: language === 'auto' ? null : language
    })
    .run();

  activeCodeLanguage.value = language;
}

function openLinkDialog( editor ) {
  currentEditor.value = editor;
  linkDraft.value = editor.getAttributes( 'link' ).href ?? '';
  linkSubmitted.value = false;
  linkDialogOpen.value = true;
}

function applyLink() {
  linkSubmitted.value = true;

  if ( linkError.value ) {
    return;
  }

  currentEditor.value
    ?.chain()
    .focus()
    .extendMarkRange( 'link' )
    .setLink({ href: linkDraft.value.trim() })
    .run();

  linkDialogOpen.value = false;
}

function removeLink() {
  currentEditor.value
    ?.chain()
    .focus()
    .extendMarkRange( 'link' )
    .unsetLink()
    .run();

  linkDialogOpen.value = false;
}

function openMathDialog( editor, mode = 'inline' ) {
  currentEditor.value = editor;
  mathDraft.value = '';
  mathMode.value = mode;
  mathPosition.value = null;
  mathSubmitted.value = false;
  mathDialogOpen.value = true;
}

function onEditMath({ latex, mode, position }) {
  if ( !currentEditor.value ) {
    return;
  }

  mathDraft.value = latex;
  mathMode.value = mode;
  mathPosition.value = position;
  mathSubmitted.value = false;
  mathDialogOpen.value = true;
}

function saveMath() {
  mathSubmitted.value = true;

  if ( mathError.value || !currentEditor.value ) {
    return;
  }

  const latex = mathDraft.value.trim();

  if ( mathPosition.value !== null ) {
    const command = mathMode.value === 'inline'
      ? 'updateInlineMath'
      : 'updateBlockMath';

    currentEditor.value.commands[ command ]({
      latex,
      pos: mathPosition.value
    });
  } else if ( mathMode.value === 'inline' ) {
    currentEditor.value.commands.insertInlineMath({ latex });
  } else {
    currentEditor.value.commands.insertBlockMath({ latex });
  }

  currentEditor.value.commands.focus();
  mathDialogOpen.value = false;
}

function removeMath() {
  if ( mathPosition.value === null || !currentEditor.value ) {
    return;
  }

  const command = mathMode.value === 'inline'
    ? 'deleteInlineMath'
    : 'deleteBlockMath';

  currentEditor.value.commands[ command ]({ pos: mathPosition.value });
  currentEditor.value.commands.focus();
  mathDialogOpen.value = false;
}

function chooseImage( editor ) {
  currentEditor.value = editor;
  imageError.value = '';
  fileInput.value?.click();
}

async function insertImage( event ) {
  const [ file ] = event.target.files;

  event.target.value = '';

  if ( !file || !currentEditor.value ) {
    return;
  }

  imageError.value = '';

  if ( file.size > 20 * 1024 * 1024 ) {
    imageError.value = 'Images cannot be larger than 20 MB.';
    return;
  }

  imageImporting.value = true;

  try {
    const bytes = new Uint8Array( await file.arrayBuffer() );
    const media = await importImage( bytes );

    currentEditor.value
      .chain()
      .focus()
      .insertContent({
        type: 'mediaImage',
        attrs: {
          mediaId: media.id,
          alt: Array.from( file.name ).slice( 0, 500 ).join( '' ),
          title: null
        }
      })
      .run();
  } catch ( cause ) {
    imageError.value = conceptLibraryErrorMessage( cause );
  } finally {
    imageImporting.value = false;
  }
}
</script>

<template>
  <div class="rich-editor-field">
    <div class="rich-editor-field__heading">
      <label>{{ label }}</label>
    </div>

    <div class="rich-editor">
      <UEditor
        v-model="document"
        :aria-label="label"
        :extensions="extensions"
        :handlers="editorHandlers"
        :image="false"
        :mention="false"
        :placeholder="placeholder"
        :starter-kit="starterKit"
        content-type="json"
        class="rich-editor__surface"
        :on-mount="syncEditorState"
        :on-selection-update="syncEditorState"
        :on-transaction="syncEditorState"
      >
        <template #default="{ editor }">
          <div class="rich-editor__toolbar">
            <UEditorToolbar
              :editor="editor"
              :items="toolbarItems"
              size="sm"
              class="rich-editor__formatting"
            />

            <div class="rich-editor__inserts">
              <USelect
                v-if="codeBlockActive"
                :model-value="activeCodeLanguage"
                :items="codeLanguageItems"
                value-key="value"
                aria-label="Code language"
                leading-icon="i-lucide-braces"
                size="sm"
                class="rich-editor__language"
                @update:model-value="setCodeLanguage( editor, $event )"
              />

              <UTooltip text="Equation">
                <UButton
                  type="button"
                  icon="i-lucide-sigma"
                  aria-label="Add equation"
                  color="neutral"
                  variant="ghost"
                  size="sm"
                  @click="openMathDialog( editor )"
                />
              </UTooltip>

              <UTooltip text="Image">
                <UButton
                  type="button"
                  icon="i-lucide-image-plus"
                  aria-label="Add image"
                  color="neutral"
                  variant="ghost"
                  size="sm"
                  :loading="imageImporting"
                  @click="chooseImage( editor )"
                />
              </UTooltip>
            </div>
          </div>
        </template>
      </UEditor>

      <input
        ref="fileInput"
        type="file"
        accept="image/gif,image/jpeg,image/png,image/webp"
        class="sr-only"
        tabindex="-1"
        @change="insertImage"
      >
    </div>

    <p
      v-if="imageError"
      class="rich-editor-field__error"
      role="alert"
    >
      {{ imageError }}
    </p>

    <UModal
      v-model:open="linkDialogOpen"
      title="Link"
      description="Use an http, https, or mailto address."
    >
      <template #body>
        <UFormField
          label="Address"
          :error="linkError"
        >
          <UInput
            v-model="linkDraft"
            placeholder="https://example.com"
            autocomplete="off"
            autofocus
            class="w-full"
            @keydown.enter.prevent="applyLink"
          />
        </UFormField>
      </template>

      <template #footer>
        <div class="dialog-actions dialog-actions--split">
          <UButton
            v-if="currentEditor?.isActive( 'link' )"
            color="error"
            variant="ghost"
            @click="removeLink"
          >
            Remove link
          </UButton>

          <span />

          <UButton
            color="neutral"
            variant="ghost"
            @click="linkDialogOpen = false"
          >
            Cancel
          </UButton>

          <UButton @click="applyLink">
            Apply
          </UButton>
        </div>
      </template>
    </UModal>

    <UModal
      v-model:open="mathDialogOpen"
      :title="mathPosition === null ? 'Add equation' : 'Edit equation'"
      description="Enter LaTeX without delimiter characters."
    >
      <template #body>
        <div class="math-editor-dialog">
          <div
            v-if="mathPosition === null"
            class="segmented-control"
          >
            <button
              type="button"
              class="segmented-control__button"
              :class="{ 'segmented-control__button--active': mathMode === 'inline' }"
              @click="mathMode = 'inline'"
            >
              Inline
            </button>

            <button
              type="button"
              class="segmented-control__button"
              :class="{ 'segmented-control__button--active': mathMode === 'block' }"
              @click="mathMode = 'block'"
            >
              Block
            </button>
          </div>

          <UFormField
            label="LaTeX"
            :error="mathError"
          >
            <UTextarea
              v-model="mathDraft"
              placeholder="E = mc^2"
              :rows="4"
              :maxlength="10000"
              autofocus
              class="w-full math-editor-dialog__input"
            />
          </UFormField>
        </div>
      </template>

      <template #footer>
        <div class="dialog-actions dialog-actions--split">
          <UButton
            v-if="mathPosition !== null"
            color="error"
            variant="ghost"
            @click="removeMath"
          >
            Remove equation
          </UButton>

          <span />

          <UButton
            color="neutral"
            variant="ghost"
            @click="mathDialogOpen = false"
          >
            Cancel
          </UButton>

          <UButton @click="saveMath">
            {{ mathPosition === null ? 'Add' : 'Save' }}
          </UButton>
        </div>
      </template>
    </UModal>
  </div>
</template>
