<script setup>
import { computed, nextTick, reactive, ref, watch } from 'vue';

import { retrievalForms } from '../retrieval-forms/catalog';

import ClozePreview from './ClozePreview.vue';
import ImageOcclusionPreview from './ImageOcclusionPreview.vue';
import RichContentEditor from './RichContentEditor.vue';
import ConceptCardPreview from './ConceptCardPreview.vue';
import {
  collectClozeGroups,
  removeAllClozeMarks
} from '../cloze/documents';
import {
  collectImageOcclusionGroups,
  removeAllImageOcclusionRegions
} from '../image-occlusion/documents';
import {
  cloneConceptContent,
  createEmptyConceptContent,
  richDocumentHasContent
} from '../rich-content/schema';
import {
  captureConceptEditorState,
  cloneConceptEditorState,
  conceptRetrievalFormId,
  createConceptEditorState
} from '../drafts/conceptDraft';

const CLOZE_ID = 'cloze';
const EXPLAIN_ID = 'explain';
const IMAGE_OCCLUSION_ID = 'image-occlusion';
const PROBLEM_ID = 'problem';
const STANDARD_RECALL_ID = 'standard-recall';
const TYPE_ANSWER_ID = 'type-answer';
const MAXIMUM_ACCEPTED_ANSWERS = 20;
const MAXIMUM_ACCEPTED_ANSWER_LENGTH = 500;
const MAXIMUM_EXPLAIN_KEY_POINTS = 12;
const MAXIMUM_EXPLAIN_KEY_POINT_LENGTH = 280;
const MAXIMUM_PROBLEM_CHECKPOINTS = 12;
const MAXIMUM_PROBLEM_CHECKPOINT_LENGTH = 280;

const explainFocusItems = [
  {
    label: 'Why',
    value: 'why'
  },
  {
    label: 'How',
    value: 'how'
  },
  {
    label: 'Cause and effect',
    value: 'causeAndEffect'
  },
  {
    label: 'Compare and contrast',
    value: 'compareAndContrast'
  }
];

const props = defineProps({
  concept: {
    type: Object,
    default: null
  },
  decks: {
    type: Array,
    default: () => []
  },
  disabled: {
    type: Boolean,
    default: false
  },
  error: {
    type: String,
    default: ''
  },
  editorState: {
    type: Object,
    default: null
  },
  loading: {
    type: Boolean,
    default: false
  },
  media: {
    type: Array,
    default: () => []
  },
  importsPending: {
    type: Boolean,
    default: false
  },
  mode: {
    type: String,
    default: 'create',
    validator: ( value ) => [ 'create', 'edit' ].includes( value )
  },
  saveCommand: {
    type: Object,
    required: true
  },
  tags: {
    type: Array,
    default: () => []
  },
  templates: {
    type: Array,
    default: () => []
  }
});

const emit = defineEmits([ 'cancel', 'change', 'manage', 'submit' ]);

const form = reactive({
  content: createEmptyConceptContent(),
  deckIds: [],
  explainFocus: 'why',
  explainKeyPoints: [ '' ],
  problemCheckpoints: [ '' ],
  retrievalFormIds: [ STANDARD_RECALL_ID ],
  tagIds: [],
  typeAnswerAcceptedAnswers: [ '' ],
  typeAnswerInitialized: false,
  title: ''
});

const feedbackFieldsOpen = ref( false );
const feedbackFieldsMounted = ref( false );
const editorSession = ref( 0 );
const submitted = ref( false );
const descriptionsOpen = ref( false );
const activeSetupId = ref( STANDARD_RECALL_ID );
const previewOpen = ref( false );
const editorElement = ref( null );

const deckItems = computed( () => props.decks.map( ( deck ) => ({
  label: deck.name,
  value: deck.id
}) ) );

const tagItems = computed( () => props.tags.map( ( tag ) => ({
  label: tag.name,
  value: tag.id
}) ) );

const builtInRetrievalFormItems = [
  {
    description: 'Shows the prompt first and the answer after reveal.',
    label: retrievalForms.recall.label,
    value: STANDARD_RECALL_ID
  },
  {
    description: 'Requires a typed response before the answer is revealed.',
    label: retrievalForms.typeAnswer.label,
    value: TYPE_ANSWER_ID
  },
  {
    description: 'Builds an explanation and compares it with key points.',
    label: retrievalForms.explain.label,
    value: EXPLAIN_ID
  },
  {
    description: 'Works through a problem and checks the solution steps.',
    label: retrievalForms.problem.label,
    value: PROBLEM_ID
  },
  {
    description: 'Hides marked Prompt passages and reveals them in context.',
    label: retrievalForms.cloze.label,
    value: CLOZE_ID
  },
  {
    description: 'Hides selected regions of a Prompt image.',
    label: retrievalForms.imageOcclusion.label,
    value: IMAGE_OCCLUSION_ID
  }
];

const templateChoices = computed( () => props.templates.map( ( template ) => ({
  description: template.mode === 'custom' ? 'HTML & CSS template' : 'Visual template',
  label: template.name,
  value: template.id
}) ) );
const retrievalFormItems = computed( () => [ ...builtInRetrievalFormItems, ...templateChoices.value ]);

const cardChoices = computed( () => builtInRetrievalFormItems.map( ( item ) => ({
  ...item,
  description: descriptionsOpen.value ? item.description : undefined
}) ) );
const selectedTemplateIds = computed({
  get: () => form.retrievalFormIds.filter( ( id ) => templateChoices.value.some( ( item ) => item.value === id ) ),
  set: ( ids ) => updateRetrievalForms([
    ...form.retrievalFormIds.filter( ( id ) => !templateChoices.value.some( ( item ) => item.value === id ) ),
    ...ids
  ])
});
const selectedSetups = computed( () => retrievalFormItems.value.filter( ( item ) => (
  form.retrievalFormIds.includes( item.value )
) ).map( ( item ) => ({ ...item, count: cardCount( item.value ) }) ) );
const activeSetup = computed( () => selectedSetups.value.find( ( item ) => (
  item.value === activeSetupId.value
) ) );
const activeSetupIndex = computed( () => selectedSetups.value.indexOf( activeSetup.value ) );
const generatedCardCount = computed( () => selectedSetups.value.reduce( ( sum, item ) => (
  sum + item.count
), 0 ) );
const cardCountLabel = computed( () => `${ generatedCardCount.value } ${
  generatedCardCount.value === 1 ? 'card' : 'cards'
}` );
const setupVisible = computed( () => selectedSetups.value.some( ( item ) => (
  item.value !== STANDARD_RECALL_ID
) ) || previewOpen.value );

watch( selectedSetups, ( items ) => {
  if ( !items.some( ( item ) => item.value === activeSetupId.value ) ) {
    activeSetupId.value = items[ 0 ]?.value ?? '';
  }
});

function cardCount( id ) {
  if ( id === CLOZE_ID ) {
    return clozeGroups.value.length;
  }

  if ( id === IMAGE_OCCLUSION_ID ) {
    return imageOcclusionGroups.value.length;
  }

  return 1;
}

const titleError = computed( () => {
  if ( !submitted.value || form.title.trim() ) {
    return '';
  }

  return 'Concept title cannot be empty.';
});

const titleLength = computed( () => Array.from( form.title ).length );
const clozeGroups = computed( () => collectClozeGroups( form.content.prompt ) );
const clozeSelected = computed( () => form.retrievalFormIds.includes( CLOZE_ID ) );
const explainSelected = computed( () => form.retrievalFormIds.includes( EXPLAIN_ID ) );
const problemSelected = computed( () => form.retrievalFormIds.includes( PROBLEM_ID ) );
const hasAnswerFeedback = computed( () => answerFeedbackHasContent(
  form.content.feedback
) );
const feedbackButtonLabel = computed( () => {
  if ( feedbackFieldsOpen.value ) {
    return 'Hide fields';
  }

  return hasAnswerFeedback.value ? 'Edit feedback' : 'Add feedback';
});
const imageOcclusionGroups = computed( () => (
  collectImageOcclusionGroups( form.content.prompt )
) );
const imageOcclusionSelected = computed( () => (
  form.retrievalFormIds.includes( IMAGE_OCCLUSION_ID )
) );
const typeAnswerSelected = computed( () => form.retrievalFormIds.includes( TYPE_ANSWER_ID ) );
const selectedContent = computed( () => {
  let prompt = form.content.prompt;

  if ( !clozeSelected.value && clozeGroups.value.length ) {
    prompt = removeAllClozeMarks( prompt );
  }

  if ( !imageOcclusionSelected.value && imageOcclusionGroups.value.length ) {
    prompt = removeAllImageOcclusionRegions( prompt );
  }

  return { ...form.content, prompt };
});
const atAcceptedAnswerLimit = computed( () => (
  form.typeAnswerAcceptedAnswers.length >= MAXIMUM_ACCEPTED_ANSWERS
) );
const atExplainKeyPointLimit = computed( () => (
  form.explainKeyPoints.length >= MAXIMUM_EXPLAIN_KEY_POINTS
) );
const atProblemCheckpointLimit = computed( () => (
  form.problemCheckpoints.length >= MAXIMUM_PROBLEM_CHECKPOINTS
) );
const acceptedAnswerErrors = computed( () => {
  if ( !submitted.value || !typeAnswerSelected.value ) {
    return form.typeAnswerAcceptedAnswers.map( () => '' );
  }

  const normalizedAnswers = form.typeAnswerAcceptedAnswers.map( normalizeAcceptedAnswer );

  return normalizedAnswers.map( ( answer, index ) => {
    if ( !answer ) {
      return 'Enter an accepted answer.';
    }

    if ( Array.from( answer ).length > MAXIMUM_ACCEPTED_ANSWER_LENGTH ) {
      return `Accepted answers cannot exceed ${ MAXIMUM_ACCEPTED_ANSWER_LENGTH } characters.`;
    }

    const comparisonAnswer = answer.toLowerCase();
    const duplicateIndex = normalizedAnswers.findIndex( ( candidate ) => (
      candidate.toLowerCase() === comparisonAnswer
    ) );

    if ( duplicateIndex !== index ) {
      return 'Accepted answers must be unique.';
    }

    return '';
  });
});
const acceptedAnswersValid = computed( () => (
  acceptedAnswerErrors.value.every( ( error ) => !error )
) );
const explainKeyPointErrors = computed( () => {
  if ( !submitted.value || !explainSelected.value ) {
    return form.explainKeyPoints.map( () => '' );
  }

  const normalizedPoints = form.explainKeyPoints.map( normalizeExplainKeyPoint );

  return normalizedPoints.map( ( keyPoint, index ) => {
    if ( !keyPoint ) {
      return 'Enter a key point.';
    }

    if ( Array.from( keyPoint ).length > MAXIMUM_EXPLAIN_KEY_POINT_LENGTH ) {
      return `Key points cannot exceed ${ MAXIMUM_EXPLAIN_KEY_POINT_LENGTH } characters.`;
    }

    const comparisonPoint = keyPoint.toLowerCase();
    const duplicateIndex = normalizedPoints.findIndex( ( candidate ) => (
      candidate.toLowerCase() === comparisonPoint
    ) );

    if ( duplicateIndex !== index ) {
      return 'Key points must be unique.';
    }

    return '';
  });
});
const explainKeyPointsValid = computed( () => (
  explainKeyPointErrors.value.every( ( error ) => !error )
) );
const problemCheckpointErrors = computed( () => {
  if ( !submitted.value || !problemSelected.value ) {
    return form.problemCheckpoints.map( () => '' );
  }

  const normalizedCheckpoints = form.problemCheckpoints.map( normalizeProblemCheckpoint );

  return normalizedCheckpoints.map( ( checkpoint, index ) => {
    if ( !checkpoint ) {
      return 'Enter a checkpoint.';
    }

    if ( Array.from( checkpoint ).length > MAXIMUM_PROBLEM_CHECKPOINT_LENGTH ) {
      return `Checkpoints cannot exceed ${ MAXIMUM_PROBLEM_CHECKPOINT_LENGTH } characters.`;
    }

    const comparisonCheckpoint = checkpoint.toLowerCase();
    const duplicateIndex = normalizedCheckpoints.findIndex( ( candidate ) => (
      candidate.toLowerCase() === comparisonCheckpoint
    ) );

    if ( duplicateIndex !== index ) {
      return 'Checkpoints must be unique.';
    }

    return '';
  });
});
const problemCheckpointsValid = computed( () => (
  problemCheckpointErrors.value.every( ( error ) => !error )
) );
const problemPromptError = computed( () => {
  if (
    !submitted.value
    || !problemSelected.value
    || richDocumentHasContent( form.content.prompt )
  ) {
    return '';
  }

  return 'Write the problem in Prompt.';
});

const retrievalFormsError = computed( () => {
  if ( !submitted.value || form.retrievalFormIds.length ) {
    return '';
  }

  return 'Select at least one card type.';
});

const clozeError = computed( () => {
  if ( !submitted.value || !clozeSelected.value || clozeGroups.value.length ) {
    return '';
  }

  return 'Mark at least one Prompt passage as an omission.';
});

const imageOcclusionError = computed( () => {
  if (
    !submitted.value
    || !imageOcclusionSelected.value
    || imageOcclusionGroups.value.length
  ) {
    return '';
  }

  return 'Add at least one mask to a Prompt image.';
});

const removedRetrievalForms = computed( () => {
  if ( props.mode !== 'edit' || !props.concept ) {
    return [];
  }

  return props.concept.cards.filter( ( card ) => {
    const id = conceptRetrievalFormId( card );

    return !form.retrievalFormIds.includes( id );
  });
});

const submitLabel = computed( () => props.mode === 'edit' ? 'Save changes' : 'Create concept' );

watch([ () => props.concept, () => props.editorState ], ([ concept, editorState ]) => {
  const state = editorState
    ? cloneConceptEditorState( editorState )
    : createConceptEditorState( concept );

  form.content = state.content;
  editorSession.value += 1;
  feedbackFieldsOpen.value = answerFeedbackHasContent( state.content.feedback );
  feedbackFieldsMounted.value = feedbackFieldsOpen.value;
  form.title = state.title;
  form.deckIds = state.deckIds;
  form.explainFocus = state.explainFocus;
  form.explainKeyPoints = state.explainKeyPoints;
  form.problemCheckpoints = state.problemCheckpoints;
  form.retrievalFormIds = state.retrievalFormIds;
  form.tagIds = state.tagIds;
  form.typeAnswerAcceptedAnswers = state.typeAnswerAcceptedAnswers;
  form.typeAnswerInitialized = state.typeAnswerInitialized;
  activeSetupId.value = state.retrievalFormIds[ 0 ] ?? '';
  previewOpen.value = false;
  submitted.value = false;
}, { immediate: true });

watch( feedbackFieldsOpen, ( open ) => {
  if ( open ) {
    feedbackFieldsMounted.value = true;
  }
});

watch( () => captureConceptEditorState( form ), ( state ) => {
  emit( 'change', state );
});

function updateRetrievalForms( retrievalFormIds ) {
  const added = retrievalFormIds.find( ( id ) => !form.retrievalFormIds.includes( id ) );

  if ( added === TYPE_ANSWER_ID && !form.typeAnswerInitialized ) {
    const blocks = form.content.answer.content ?? [];
    const paragraph = blocks.length === 1 && blocks[ 0 ].type === 'paragraph'
      ? blocks[ 0 ]
      : null;
    const plainText = paragraph?.content?.every( ( node ) => node.type === 'text' );
    const answer = plainText
      ? normalizeAcceptedAnswer( paragraph.content.map( ( node ) => node.text ).join( '' ) )
      : '';

    if ( answer && Array.from( answer ).length <= MAXIMUM_ACCEPTED_ANSWER_LENGTH ) {
      form.typeAnswerAcceptedAnswers = [ answer ];
    }

    form.typeAnswerInitialized = true;
  }

  form.retrievalFormIds = retrievalFormIds;

  if ( added ) {
    activeSetupId.value = added;
  }
}

function normalizeAcceptedAnswer( answer ) {
  return answer.trim().replace( /\s+/gu, ' ' );
}

function answerFeedbackHasContent( feedback ) {
  return richDocumentHasContent( feedback?.explanation )
    || richDocumentHasContent( feedback?.commonMistakes );
}

function acceptedAnswerLength( answer ) {
  return Array.from( normalizeAcceptedAnswer( answer ) ).length;
}

function normalizeExplainKeyPoint( keyPoint ) {
  return keyPoint.trim().replace( /\s+/gu, ' ' );
}

function explainKeyPointLength( keyPoint ) {
  return Array.from( normalizeExplainKeyPoint( keyPoint ) ).length;
}

function normalizeProblemCheckpoint( checkpoint ) {
  return checkpoint.trim().replace( /\s+/gu, ' ' );
}

function problemCheckpointLength( checkpoint ) {
  return Array.from( normalizeProblemCheckpoint( checkpoint ) ).length;
}

function addAcceptedAnswer() {
  if ( atAcceptedAnswerLimit.value ) {
    return;
  }

  form.typeAnswerAcceptedAnswers.push( '' );
}

function removeAcceptedAnswer( index ) {
  if ( form.typeAnswerAcceptedAnswers.length === 1 ) {
    return;
  }

  form.typeAnswerAcceptedAnswers.splice( index, 1 );
}

function addExplainKeyPoint() {
  if ( atExplainKeyPointLimit.value ) {
    return;
  }

  form.explainKeyPoints.push( '' );
}

function removeExplainKeyPoint( index ) {
  if ( form.explainKeyPoints.length === 1 ) {
    return;
  }

  form.explainKeyPoints.splice( index, 1 );
}

function addProblemCheckpoint() {
  if ( atProblemCheckpointLimit.value ) {
    return;
  }

  form.problemCheckpoints.push( '' );
}

function removeProblemCheckpoint( index ) {
  if ( form.problemCheckpoints.length === 1 ) {
    return;
  }

  form.problemCheckpoints.splice( index, 1 );
}

function moveProblemCheckpoint( index, offset ) {
  const nextIndex = index + offset;

  if (
    nextIndex < 0
    || nextIndex >= form.problemCheckpoints.length
  ) {
    return;
  }

  const [ checkpoint ] = form.problemCheckpoints.splice( index, 1 );

  form.problemCheckpoints.splice( nextIndex, 0, checkpoint );
}

async function submit() {
  if ( props.disabled || props.importsPending ) {
    return;
  }

  submitted.value = true;

  if (
    !form.title.trim()
    || !form.retrievalFormIds.length
    || Boolean( clozeError.value )
    || Boolean( imageOcclusionError.value )
    || Boolean( problemPromptError.value )
    || !explainKeyPointsValid.value
    || !problemCheckpointsValid.value
    || !acceptedAnswersValid.value
  ) {
    if ( !acceptedAnswersValid.value ) {
      activeSetupId.value = TYPE_ANSWER_ID;
    } else if ( !explainKeyPointsValid.value ) {
      activeSetupId.value = EXPLAIN_ID;
    } else if ( !problemCheckpointsValid.value || problemPromptError.value ) {
      activeSetupId.value = PROBLEM_ID;
    } else if ( clozeError.value ) {
      activeSetupId.value = CLOZE_ID;
    } else if ( imageOcclusionError.value ) {
      activeSetupId.value = IMAGE_OCCLUSION_ID;
    }

    await nextTick();
    editorElement.value?.querySelector( '[aria-invalid="true"], .editor-field-error' )
      ?.scrollIntoView({ block: 'center' });
    return;
  }

  const typeAnswer = typeAnswerSelected.value
    ? {
      acceptedAnswers: form.typeAnswerAcceptedAnswers.map( normalizeAcceptedAnswer )
    }
    : null;
  const explain = explainSelected.value
    ? {
      focus: form.explainFocus,
      keyPoints: form.explainKeyPoints.map( normalizeExplainKeyPoint )
    }
    : null;
  const problem = problemSelected.value
    ? {
      checkpoints: form.problemCheckpoints.map( normalizeProblemCheckpoint )
    }
    : null;

  emit( 'submit', {
    content: cloneConceptContent( selectedContent.value ),
    deckIds: [ ...form.deckIds ],
    explain,
    includeStandardRecall: form.retrievalFormIds.includes( STANDARD_RECALL_ID ),
    problem,
    tagIds: [ ...form.tagIds ],
    templateIds: form.retrievalFormIds.filter( ( id ) => (
      id !== CLOZE_ID
      && id !== EXPLAIN_ID
      && id !== IMAGE_OCCLUSION_ID
      && id !== PROBLEM_ID
      && id !== STANDARD_RECALL_ID
      && id !== TYPE_ANSWER_ID
    ) ),
    typeAnswer,
    title: form.title
  });
}

defineExpose({ submit });
</script>

<template>
  <form
    ref="editorElement"
    class="concept-editor"
    @submit.prevent="submit"
  >
    <UAlert
      v-if="error"
      :description="error"
      icon="i-lucide-circle-alert"
      color="error"
      variant="soft"
    />

    <section
      class="editor-section"
      data-twill-editor-section="concept-basics"
    >
      <UFormField
        label="Title"
        :error="titleError || false"
        :hint="`${ titleLength } / 200`"
        required
      >
        <UInput
          v-model="form.title"
          placeholder="Concept title"
          :maxlength="200"
          autocomplete="off"
          autofocus
          class="w-full"
          :disabled="disabled"
        />
      </UFormField>
    </section>

    <section class="editor-section card-type-picker" aria-label="Card types">
      <div class="editor-section__heading">
        <h2>Card types</h2>

        <UButton
          type="button"
          color="neutral"
          variant="link"
          size="sm"
          :aria-expanded="descriptionsOpen"
          :disabled="disabled"
          @click="descriptionsOpen = !descriptionsOpen"
        >
          {{ descriptionsOpen ? 'Hide descriptions' : 'Show descriptions' }}
        </UButton>
      </div>

      <UCheckboxGroup
        :model-value="form.retrievalFormIds"
        :items="cardChoices"
        legend="Card types"
        value-key="value"
        class="retrieval-form-options"
        :ui="{ legend: 'sr-only', fieldset: 'card-type-options' }"
        :disabled="disabled"
        @update:model-value="updateRetrievalForms"
      />

      <div class="card-type-summary">
        <USelectMenu
          v-if="templateChoices.length"
          v-model="selectedTemplateIds"
          :items="templateChoices"
          value-key="value"
          multiple
          placeholder="Templates"
          aria-label="Template cards"
          :disabled="disabled"
          class="card-type-templates"
        />
        <span role="status">{{ cardCountLabel }}</span>
        <span v-if="!generatedCardCount">Select a card type and finish its setup.</span>
        <UButton
          v-if="selectedSetups.length === 1 && activeSetupId === STANDARD_RECALL_ID"
          type="button"
          color="neutral"
          variant="link"
          size="sm"
          :aria-expanded="previewOpen"
          :disabled="disabled"
          @click="previewOpen = !previewOpen"
        >
          {{ previewOpen ? 'Hide preview' : 'Preview' }}
        </UButton>
      </div>

      <p v-if="retrievalFormsError" class="editor-field-error">{{ retrievalFormsError }}</p>

      <UAlert
        v-if="removedRetrievalForms.length"
        title="Scheduling progress will be removed"
        description="Saving removes deselected cards and their scheduling progress. Adding them again starts them as new."
        icon="i-lucide-calendar-x-2"
        color="warning"
        variant="subtle"
      />
    </section>

    <section
      class="editor-section"
      data-twill-editor-section="concept-content"
    >
      <div class="concept-content-editors">
        <RichContentEditor
          :key="`prompt-${ editorSession }`"
          v-model="form.content.prompt"
          label="Prompt"
          placeholder="Write a prompt"
          :cloze-enabled="clozeSelected"
          :disabled="disabled"
          :image-occlusion-enabled="imageOcclusionSelected"
        />

        <RichContentEditor
          :key="`answer-${ editorSession }`"
          v-model="form.content.answer"
          label="Answer"
          placeholder="Write an answer"
          :disabled="disabled"
        />
      </div>
      <p v-if="problemPromptError" class="editor-field-error">{{ problemPromptError }}</p>
    </section>

    <section
      class="editor-section answer-feedback-section"
      data-twill-editor-section="answer-feedback"
    >
      <div class="editor-section__heading">
        <div>
          <h2 class="sr-only">Answer feedback</h2>
        </div>

        <UButton
          type="button"
          :leading-icon="feedbackFieldsOpen
            ? 'i-lucide-panel-top-close'
            : 'i-lucide-message-square-plus'"
          color="neutral"
          variant="subtle"
          :disabled="disabled"
          :aria-expanded="feedbackFieldsOpen"
          aria-controls="answer-feedback-fields"
          @click="feedbackFieldsOpen = !feedbackFieldsOpen"
        >
          {{ feedbackButtonLabel }}
        </UButton>
      </div>

      <div
        v-if="feedbackFieldsMounted"
        v-show="feedbackFieldsOpen"
        id="answer-feedback-fields"
        class="concept-content-editors answer-feedback-editors"
      >
        <RichContentEditor
          :key="`explanation-${ editorSession }`"
          v-model="form.content.feedback.explanation"
          label="Explanation and context"
          placeholder="Explain why the answer is correct or add useful context"
          :disabled="disabled"
        />

        <RichContentEditor
          :key="`mistakes-${ editorSession }`"
          v-model="form.content.feedback.commonMistakes"
          label="Common mistakes"
          placeholder="Describe likely mistakes or misconceptions"
          :disabled="disabled"
        />
      </div>
    </section>

    <section
      v-if="setupVisible && activeSetup"
      class="editor-section card-setup"
      data-twill-editor-section="concept-retrieval-forms"
    >
      <nav v-if="selectedSetups.length > 1" class="card-setup__steps" aria-label="Card setup">
        <UButton
          v-for="item in selectedSetups"
          :key="item.value"
          type="button"
          color="neutral"
          :variant="item.value === activeSetupId ? 'subtle' : 'ghost'"
          :aria-current="item.value === activeSetupId ? 'step' : undefined"
          :disabled="disabled"
          @click="activeSetupId = item.value"
        >
          {{ item.label }} <span class="card-setup__count">{{ item.count }}</span>
        </UButton>
      </nav>

      <div class="editor-section__heading">
        <h2>{{ activeSetup.label }}</h2>
        <span class="card-setup__count">{{ activeSetup.count }} {{ activeSetup.count === 1 ? 'card' : 'cards' }}</span>
      </div>

      <p
        v-if="clozeError && activeSetupId === CLOZE_ID"
        class="editor-field-error"
      >
        {{ clozeError }}
      </p>

      <p
        v-if="imageOcclusionError && activeSetupId === IMAGE_OCCLUSION_ID"
        class="editor-field-error"
      >
        {{ imageOcclusionError }}
      </p>

      <ClozePreview v-if="activeSetupId === CLOZE_ID" :document="form.content.prompt" />
      <ImageOcclusionPreview v-else-if="activeSetupId === IMAGE_OCCLUSION_ID" :document="form.content.prompt" />

      <div class="card-setup__workspace">
        <div
          v-if="activeSetupId === TYPE_ANSWER_ID"
          class="type-answer-settings"
        >
          <div class="type-answer-settings__heading">
            <div>
              <h3>Accepted answers</h3>
              <p>Add alternatives that should count as an exact match.</p>
            </div>

            <span>
              {{ form.typeAnswerAcceptedAnswers.length }} / {{ MAXIMUM_ACCEPTED_ANSWERS }}
            </span>
          </div>

          <div class="accepted-answer-list">
            <UFormField
              v-for="( answer, index ) in form.typeAnswerAcceptedAnswers"
              :key="index"
              :label="index === 0 ? 'Answer' : `Alternative ${ index }`"
              :error="acceptedAnswerErrors[ index ] || false"
              :hint="`${ acceptedAnswerLength( answer ) } / ${ MAXIMUM_ACCEPTED_ANSWER_LENGTH }`"
              required
            >
              <div class="accepted-answer-row">
                <UInput
                  v-model="form.typeAnswerAcceptedAnswers[ index ]"
                  :placeholder="index === 0 ? 'Accepted answer' : 'Alternative answer'"
                  autocomplete="off"
                  class="accepted-answer-row__input"
                  size="lg"
                  :disabled="disabled"
                />

                <UButton
                  type="button"
                  icon="i-lucide-x"
                  :aria-label="`Remove ${ index === 0 ? 'answer' : `alternative ${ index }` }`"
                  color="neutral"
                  variant="ghost"
                  size="lg"
                  square
                  :disabled="disabled || form.typeAnswerAcceptedAnswers.length === 1"
                  @click="removeAcceptedAnswer( index )"
                />
              </div>
            </UFormField>
          </div>

          <UButton
            type="button"
            label="Add alternative"
            leading-icon="i-lucide-plus"
            color="neutral"
            variant="subtle"
            :disabled="disabled || atAcceptedAnswerLimit"
            class="type-answer-settings__add"
            @click="addAcceptedAnswer"
          />
        </div>

        <div
          v-if="activeSetupId === EXPLAIN_ID"
          class="explain-settings"
        >
          <div class="explain-settings__heading">
            <div>
              <h3>Explanation guide</h3>
            </div>

            <span>
              {{ form.explainKeyPoints.length }} / {{ MAXIMUM_EXPLAIN_KEY_POINTS }}
            </span>
          </div>

          <UFormField
            label="Prompt focus"
            required
          >
            <USelect
              v-model="form.explainFocus"
              :items="explainFocusItems"
              value-key="value"
              class="explain-settings__focus"
              size="lg"
              :disabled="disabled"
            />
          </UFormField>

          <div class="explain-key-point-list">
            <UFormField
              v-for="( keyPoint, index ) in form.explainKeyPoints"
              :key="index"
              :label="`Key point ${ index + 1 }`"
              :error="explainKeyPointErrors[ index ] || false"
              :hint="`${ explainKeyPointLength( keyPoint ) } / ${ MAXIMUM_EXPLAIN_KEY_POINT_LENGTH }`"
              required
            >
              <div class="explain-key-point-row">
                <UTextarea
                  v-model="form.explainKeyPoints[ index ]"
                  placeholder="Expected idea"
                  :rows="2"
                  autoresize
                  :maxrows="5"
                  class="explain-key-point-row__input"
                  :disabled="disabled"
                />

                <UButton
                  type="button"
                  icon="i-lucide-x"
                  :aria-label="`Remove key point ${ index + 1 }`"
                  color="neutral"
                  variant="ghost"
                  size="lg"
                  square
                  :disabled="disabled || form.explainKeyPoints.length === 1"
                  @click="removeExplainKeyPoint( index )"
                />
              </div>
            </UFormField>
          </div>

          <UButton
            type="button"
            label="Add key point"
            leading-icon="i-lucide-plus"
            color="neutral"
            variant="subtle"
            :disabled="disabled || atExplainKeyPointLimit"
            class="explain-settings__add"
            @click="addExplainKeyPoint"
          />
        </div>

        <div
          v-if="activeSetupId === PROBLEM_ID"
          class="problem-settings"
        >
          <div class="problem-settings__heading">
            <div>
              <h3>Solution checkpoints</h3>
            </div>

            <span>
              {{ form.problemCheckpoints.length }} / {{ MAXIMUM_PROBLEM_CHECKPOINTS }}
            </span>
          </div>

          <div class="problem-checkpoint-list">
            <UFormField
              v-for="( checkpoint, index ) in form.problemCheckpoints"
              :key="index"
              :label="`Checkpoint ${ index + 1 }`"
              :error="problemCheckpointErrors[ index ] || false"
              :hint="`${ problemCheckpointLength( checkpoint ) } / ${ MAXIMUM_PROBLEM_CHECKPOINT_LENGTH }`"
              required
            >
              <div class="problem-checkpoint-row">
                <UTextarea
                  v-model="form.problemCheckpoints[ index ]"
                  placeholder="Expected step or criterion"
                  :rows="2"
                  autoresize
                  :maxrows="5"
                  class="problem-checkpoint-row__input"
                  :disabled="disabled"
                />

                <div class="problem-checkpoint-row__actions">
                  <UButton
                    type="button"
                    icon="i-lucide-chevron-up"
                    :aria-label="`Move checkpoint ${ index + 1 } up`"
                    color="neutral"
                    variant="ghost"
                    size="sm"
                    square
                    :disabled="disabled || index === 0"
                    @click="moveProblemCheckpoint( index, -1 )"
                  />

                  <UButton
                    type="button"
                    icon="i-lucide-chevron-down"
                    :aria-label="`Move checkpoint ${ index + 1 } down`"
                    color="neutral"
                    variant="ghost"
                    size="sm"
                    square
                    :disabled="disabled || index === form.problemCheckpoints.length - 1"
                    @click="moveProblemCheckpoint( index, 1 )"
                  />

                  <UButton
                    type="button"
                    icon="i-lucide-x"
                    :aria-label="`Remove checkpoint ${ index + 1 }`"
                    color="neutral"
                    variant="ghost"
                    size="sm"
                    square
                    :disabled="disabled || form.problemCheckpoints.length === 1"
                    @click="removeProblemCheckpoint( index )"
                  />
                </div>
              </div>
            </UFormField>
          </div>

          <UButton
            type="button"
            label="Add checkpoint"
            leading-icon="i-lucide-plus"
            color="neutral"
            variant="subtle"
            :disabled="disabled || atProblemCheckpointLimit"
            class="problem-settings__add"
            @click="addProblemCheckpoint"
          />
        </div>

        <ConceptCardPreview
          v-if="activeSetupId !== CLOZE_ID && activeSetupId !== IMAGE_OCCLUSION_ID"
          :key="activeSetupId"
          :type-id="activeSetupId"
          :content="selectedContent"
          :title="form.title"
          :media="media"
        />
      </div>

      <div v-if="selectedSetups.length > 1" class="card-setup__navigation">
        <UButton
          type="button"
          color="neutral"
          variant="ghost"
          leading-icon="i-lucide-chevron-left"
          :disabled="disabled || activeSetupIndex === 0"
          @click="activeSetupId = selectedSetups[ activeSetupIndex - 1 ].value"
        >
          Back
        </UButton>
        <span>{{ activeSetupIndex + 1 }} / {{ selectedSetups.length }}</span>
        <UButton
          type="button"
          color="neutral"
          variant="subtle"
          trailing-icon="i-lucide-chevron-right"
          :disabled="disabled || activeSetupIndex === selectedSetups.length - 1"
          @click="activeSetupId = selectedSetups[ activeSetupIndex + 1 ].value"
        >
          Next
        </UButton>
      </div>
    </section>

    <section
      class="editor-section"
      data-twill-editor-section="concept-organization"
    >
      <div class="editor-section__heading">
        <div>
          <h2>Organization</h2>
        </div>

        <UButton
          type="button"
          leading-icon="i-lucide-settings-2"
          color="neutral"
          variant="subtle"
          :disabled="disabled"
          @click="emit( 'manage' )"
        >
          Manage
        </UButton>
      </div>

      <div class="editor-selection-grid">
        <div class="editor-selection">
          <h3>Decks</h3>

          <USelectMenu
            v-if="deckItems.length"
            v-model="form.deckIds"
            :items="deckItems"
            value-key="value"
            multiple
            placeholder="Choose decks"
            aria-label="Decks"
            class="w-full"
            :disabled="disabled"
          />

          <p
            v-else
            class="editor-selection__empty"
          >
            No decks yet.
          </p>
        </div>

        <div class="editor-selection">
          <h3>Tags</h3>

          <USelectMenu
            v-if="tagItems.length"
            v-model="form.tagIds"
            :items="tagItems"
            value-key="value"
            multiple
            placeholder="Choose tags"
            aria-label="Tags"
            class="w-full"
            :disabled="disabled"
          />

          <p
            v-else
            class="editor-selection__empty"
          >
            No tags yet.
          </p>
        </div>
      </div>
    </section>

    <footer class="editor-actions">
      <p
        v-if="importsPending"
        class="mr-auto self-center text-sm text-muted"
        role="status"
      >
        Importing images. Save will be available when they finish.
      </p>

      <UButton
        type="button"
        color="neutral"
        variant="link"
        :disabled="disabled"
        @click="emit( 'cancel' )"
      >
        Cancel
      </UButton>

      <UButton
        type="submit"
        leading-icon="i-lucide-check"
        :disabled="disabled || importsPending"
        :loading="loading"
        :aria-keyshortcuts="saveCommand.ariaKeyshortcuts"
        :title="saveCommand.tooltip"
      >
        {{ submitLabel }}
      </UButton>
    </footer>
  </form>
</template>
