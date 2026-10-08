<script setup>
import { computed, nextTick, onBeforeUnmount, ref, shallowRef, watch } from 'vue';

import { conceptLibraryErrorMessage } from '../composables/useConceptLibrary';
import { useAuthoringMedia } from '../composables/useAuthoringMedia';
import { useCommands } from '../composables/useCommands';
import { COMMAND_IDS, isApplePlatform } from '../commands/registry';
import { createUuid } from '../lib/identifiers';
import { changeClozeOmissions, clozeEditingTarget } from '../cloze/editing';
import { answerPartRevealGroups, collectAnswerParts } from '../answer-parts/documents';
import {
  answerPartEditingTarget,
  setAnswerPartGroup,
  toggleAnswerPart
} from '../answer-parts/editing';
import {
  collectClozeGroups,
  createClozeGroupId,
  MAXIMUM_CLOZE_GROUPS
} from '../cloze/documents';
import {
  createRichContentExtensions,
  richContentStarterKit
} from '../rich-content/schema';

const props = defineProps({
  label: {
    type: String,
    required: true
  },
  clozeEnabled: {
    type: Boolean,
    default: false
  },
  answerPartsEnabled: {
    type: Boolean,
    default: false
  },
  disabled: {
    type: Boolean,
    default: false
  },
  imageOcclusionEnabled: {
    type: Boolean,
    default: false
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

const authoringMedia = useAuthoringMedia();
const applePlatform = isApplePlatform();
const commands = useCommands();

const clozeMenuOpen = ref( false );
const clozeTarget = shallowRef( null );
const answerPartTarget = shallowRef( null );
const currentEditor = shallowRef( null );
const editorFocused = ref( false );
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
let activeImport = null;

onBeforeUnmount( cancelImport );

function cancelImport() {
  activeImport?.finish();
  activeImport = null;
  imageImporting.value = false;
}

function preserveHistoryBoundary( event ) {
  if ( event.defaultPrevented || event.isComposing || event.keyCode === 229
    || event.altKey || !event.target.closest( '.ProseMirror' )
    || event.target.closest( 'input, textarea, select' ) ) {
    return;
  }

  if ( event.type === 'beforeinput' ) {
    const command = {
      historyUndo: 'undo',
      historyRedo: 'redo'
    }[ event.inputType ];

    if ( command && event.cancelable ) {
      event.preventDefault();

      if ( !props.disabled ) {
        currentEditor.value?.commands[ command ]();
      }
    }

    return;
  }

  const modifier = applePlatform ? event.metaKey && !event.ctrlKey : event.ctrlKey && !event.metaKey;
  const key = event.key.toLowerCase();
  const historyShortcut = modifier && ( key === 'z' || ( !applePlatform && key === 'y' ) );

  if ( historyShortcut ) {
    // An empty editor history must not fall through to another field's native history
    event.preventDefault();
  }
}

watch( () => props.disabled, ( disabled ) => {
  currentEditor.value?.setEditable( !disabled, false );

  if ( disabled ) {
    cancelImport();
    clozeMenuOpen.value = false;
    linkDialogOpen.value = false;
    mathDialogOpen.value = false;
  }
});

const document = computed({
  get: () => props.modelValue,
  set: ( value ) => emit( 'update:modelValue', value )
});

const clozeGroups = computed( () => collectClozeGroups( document.value ) );
const answerParts = computed( () => collectAnswerParts( document.value ) );
const clozeActive = computed( () => Boolean( clozeTarget.value?.omissions.length ) );
const clozeAtLimit = computed( () => clozeGroups.value.length >= MAXIMUM_CLOZE_GROUPS );
const clozeToggleLabel = computed( () => clozeActive.value
  ? 'Remove cloze omission'
  : clozeAtLimit.value ? `Limit of ${ MAXIMUM_CLOZE_GROUPS } cloze cards reached` : 'Add cloze omission' );
const clozeCommand = computed( () => commands.command( COMMAND_IDS.conceptToggleCloze ) );

watch( () => props.clozeEnabled, ( enabled, previous, onCleanup ) => {
  if ( !enabled ) {
    return;
  }

  const unregister = commands.register( COMMAND_IDS.conceptToggleCloze, {
    enabled: computed( () => (
      editorFocused.value && !props.disabled && Boolean( clozeTarget.value )
      && ( clozeActive.value || !clozeAtLimit.value )
    ) ),
    execute: () => toggleCloze( currentEditor.value )
  });

  onCleanup( unregister );
}, { immediate: true });

const clozeGroupItems = computed( () => {
  const items = clozeGroups.value.map( ( group, index ) => ({
    label: `Card ${ index + 1 } — ${ clozeGroupSummary( group ) }`,
    onSelect: () => assignClozeGroup( group.id )
  }) );

  if ( clozeGroups.value.length < MAXIMUM_CLOZE_GROUPS ) {
    items.unshift({
      label: 'Separate card',
      onSelect: () => assignClozeGroup( createClozeGroupId() )
    });
  }

  return items;
});

const extensions = createRichContentExtensions({
  codeBlockEditingEnabled: () => !props.disabled,
  imageEditingEnabled: () => !props.disabled,
  imageOcclusionDocument: () => document.value,
  imageOcclusionEnabled: () => props.imageOcclusionEnabled,
  onEditMath
});
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
  clozeTarget.value = props.clozeEnabled ? clozeEditingTarget( editor ) : null;
  answerPartTarget.value = props.answerPartsEnabled ? answerPartEditingTarget( editor ) : null;
}

function answerPartGroupItems( part ) {
  const groups = new Map();

  for ( const other of answerParts.value ) {
    if ( other.id !== part.id && !groups.has( other.groupId ) ) {
      groups.set( other.groupId, {
        label: `With ${ other.label.toLowerCase() }`,
        value: other.groupId
      });
    }
  }

  const defaultGroup = answerPartRevealGroups( props.modelValue ).some( ( group ) => group.default );

  return [{ label: defaultGroup ? 'Default group' : 'No group', value: 'separate' }, ...groups.values() ];
}

function answerPartGroupValue( part ) {
  return answerParts.value.some( ( other ) => other.id !== part.id && other.groupId === part.groupId )
    ? part.groupId
    : 'separate';
}

function changeAnswerPartGroup( part, value ) {
  if ( props.disabled || !props.answerPartsEnabled || !currentEditor.value ) {
    return;
  }

  const groupId = value === 'separate' ? createUuid() : value;

  if ( value !== 'separate' && !answerParts.value.some( ( other ) => other.groupId === groupId ) ) {
    return;
  }

  setAnswerPartGroup( currentEditor.value, part.id, groupId );
}

function changeAnswerPart( editor ) {
  if ( !props.disabled && props.answerPartsEnabled ) {
    toggleAnswerPart( editor );
  }
}

watch( () => props.clozeEnabled, () => {
  clozeMenuOpen.value = false;

  if ( currentEditor.value ) {
    syncEditorState({ editor: currentEditor.value });
  }
});

function clozeGroupSummary( group ) {
  const summary = group.passages.join( ' + ' );
  const characters = Array.from( summary );

  return characters.length > 48
    ? `${ characters.slice( 0, 47 ).join( '' ) }…`
    : summary;
}

function toggleCloze( editor ) {
  if ( props.disabled || !props.clozeEnabled ) {
    return;
  }

  const target = clozeEditingTarget( editor );

  if ( target?.omissions.length ) {
    changeClozeOmissions( editor, target );
  } else if ( !clozeAtLimit.value ) {
    changeClozeOmissions( editor, target, createClozeGroupId() );
  }
}

function assignClozeGroup( groupId ) {
  const editor = currentEditor.value;

  if ( !editor || props.disabled || !props.clozeEnabled
    || ( clozeAtLimit.value && !clozeGroups.value.some( group => group.id === groupId ) ) ) {
    return;
  }

  changeClozeOmissions( editor, clozeEditingTarget( editor ), groupId );
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
  if ( props.disabled || imageImporting.value ) {
    return;
  }

  currentEditor.value = editor;
  imageError.value = '';
  fileInput.value?.click();
}

async function insertImage( event ) {
  const [ file ] = event.target.files;

  event.target.value = '';

  if ( !file || !currentEditor.value || props.disabled || imageImporting.value ) {
    return;
  }

  imageError.value = '';

  if ( file.size > 20 * 1024 * 1024 ) {
    imageError.value = 'Images cannot be larger than 20 MB.';
    return;
  }

  const editor = currentEditor.value;
  const request = authoringMedia.beginImport();

  if ( !request ) {
    return;
  }

  activeImport = request;
  imageImporting.value = true;

  const isCurrent = () => (
    request.isCurrent()
    && editor === currentEditor.value
    && !editor.isDestroyed
  );

  try {
    const bytes = new Uint8Array( await file.arrayBuffer() );

    if ( !isCurrent() ) {
      return;
    }

    const media = await authoringMedia.importImage( bytes, request.sessionId );

    if ( !isCurrent() ) {
      return;
    }

    editor
      .chain()
      .insertContent({
        type: 'mediaImage',
        attrs: {
          mediaId: media.id,
          alt: Array.from( file.name ).slice( 0, 500 ).join( '' ),
          title: null,
          occlusionRegions: []
        }
      })
      .run();
  } catch ( cause ) {
    if ( isCurrent() ) {
      imageError.value = conceptLibraryErrorMessage( cause );
    }
  } finally {
    // Let document updates reach the form and draft before enabling Save or Leave
    await nextTick();
    request.finish();

    if ( activeImport === request ) {
      activeImport = null;
      imageImporting.value = false;
    }
  }
}
</script>

<template>
  <div class="rich-editor-field">
    <div class="rich-editor-field__heading">
      <label>{{ label }}</label>
    </div>

    <div
      class="rich-editor"
      @focusin="editorFocused = true"
      @focusout="editorFocused = $event.currentTarget.contains( $event.relatedTarget )"
      @keydown="preserveHistoryBoundary"
      @beforeinput="preserveHistoryBoundary"
    >
      <UEditor
        v-model="document"
        :aria-label="label"
        :editable="!disabled"
        :extensions="extensions"
        :handlers="editorHandlers"
        :image="false"
        :mention="false"
        :placeholder="{ placeholder, mode: 'firstLine' }"
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
              :items="[ ...toolbarItems, [{ slot: 'inserts' }] ]"
              size="sm"
              class="rich-editor__formatting"
            >
              <template #inserts>
                <UTooltip
                  v-if="answerPartsEnabled"
                  :text="answerPartTarget?.active ? 'Remove answer part' : 'Reveal selected blocks as an answer part'"
                >
                  <UButton
                    type="button"
                    icon="i-lucide-square-stack"
                    aria-label="Answer part"
                    :aria-pressed="answerPartTarget?.active === true"
                    :color="answerPartTarget?.active ? 'primary' : 'neutral'"
                    :variant="answerPartTarget?.active ? 'subtle' : 'ghost'"
                    size="sm"
                    :disabled="disabled || !answerPartTarget"
                    @click="changeAnswerPart( editor )"
                  />
                </UTooltip>

                <UTooltip text="Equation">
                  <UButton
                    type="button"
                    icon="i-lucide-sigma"
                    aria-label="Add equation"
                    color="neutral"
                    variant="ghost"
                    size="sm"
                    :disabled="disabled"
                    @click="openMathDialog( editor )"
                  />
                </UTooltip>

                <UTooltip
                  v-if="clozeEnabled"
                  :text="`${ clozeToggleLabel } (${ clozeCommand.shortcutLabel })`"
                >
                  <UButton
                    type="button"
                    icon="i-lucide-text-select"
                    aria-label="Cloze omission"
                    :aria-keyshortcuts="clozeCommand.ariaKeyshortcuts"
                    :aria-pressed="clozeActive"
                    :color="clozeActive ? 'primary' : 'neutral'"
                    :variant="clozeActive ? 'subtle' : 'ghost'"
                    size="sm"
                    :disabled="disabled || !clozeTarget || ( clozeAtLimit && !clozeActive )"
                    @click="toggleCloze( editor )"
                  />
                </UTooltip>

                <UDropdownMenu
                  v-if="clozeEnabled"
                  v-model:open="clozeMenuOpen"
                  :items="clozeGroupItems"
                  :content="{ align: 'start' }"
                >
                  <UTooltip text="Hide together on a card">
                    <UButton
                      type="button"
                      icon="i-lucide-chevron-down"
                      aria-label="Group cloze omissions"
                      color="neutral"
                      variant="ghost"
                      size="sm"
                      :disabled="disabled || !clozeTarget || !clozeGroups.length"
                    />
                  </UTooltip>
                </UDropdownMenu>

                <UTooltip text="Image">
                  <UButton
                    type="button"
                    icon="i-lucide-image-plus"
                    aria-label="Add image"
                    :label="imageOcclusionEnabled ? 'Choose image' : undefined"
                    color="neutral"
                    variant="ghost"
                    size="sm"
                    :disabled="disabled || imageImporting"
                    :loading="imageImporting"
                    @click="chooseImage( editor )"
                  />
                </UTooltip>
              </template>
            </UEditorToolbar>
          </div>
        </template>
      </UEditor>

      <input
        ref="fileInput"
        type="file"
        accept="image/gif,image/jpeg,image/png,image/webp"
        class="sr-only"
        tabindex="-1"
        :disabled="disabled || imageImporting"
        @change="insertImage"
      >

      <p
        v-if="imageError"
        class="rich-editor-field__error"
        role="alert"
      >
        {{ imageError }}
      </p>

      <div
        v-if="answerPartsEnabled && answerParts.length"
        class="answer-part-groups"
        data-twill-answer-parts
        aria-label="Answer reveal groups"
      >
        <div
          v-for="part in answerParts"
          :key="part.id"
          class="answer-part-groups__row"
        >
          <span>{{ part.label }}</span>
          <USelect
            :model-value="answerPartGroupValue( part )"
            :items="answerPartGroupItems( part )"
            :aria-label="`Reveal ${ part.label.toLowerCase() }`"
            :disabled="disabled"
            size="sm"
            class="answer-part-groups__select"
            @update:model-value="changeAnswerPartGroup( part, $event )"
          />
        </div>
      </div>
    </div>

    <UModal
      v-model:open="linkDialogOpen"
      title="Link"
      description="Use an http, https, or mailto address."
    >
      <template #body>
        <UFormField
          label="Address"
          :error="linkError || false"
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
            variant="link"
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
            :error="mathError || false"
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
            variant="link"
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
