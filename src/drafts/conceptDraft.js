import {
  cloneConceptContent,
  createEmptyConceptContent
} from '../rich-content/schema';

const STANDARD_RECALL_ID = 'standard-recall';
const DEFAULT_EXPLAIN_FOCUS = 'why';

export function createConceptEditorState( concept = null ) {
  const explain = concept?.cards.find( ( card ) => (
    card.retrievalKind === 'explain'
  ) );
  const typeAnswer = concept?.cards.find( ( card ) => (
    card.retrievalKind === 'typeAnswer'
  ) );

  return {
    content: cloneConceptContent( concept?.content ),
    deckIds: concept?.decks.map( ( deck ) => deck.id ) ?? [],
    explainFocus: explain?.explain?.focus ?? DEFAULT_EXPLAIN_FOCUS,
    explainKeyPoints: explain?.explain?.keyPoints?.length
      ? [ ...explain.explain.keyPoints ]
      : [ '' ],
    retrievalFormIds: concept
      ? [ ...new Set( concept.cards.map( conceptRetrievalFormId ) ) ]
      : [ STANDARD_RECALL_ID ],
    tagIds: concept?.tags.map( ( tag ) => tag.id ) ?? [],
    typeAnswerAcceptedAnswers: typeAnswer?.typeAnswer?.acceptedAnswers.length
      ? [ ...typeAnswer.typeAnswer.acceptedAnswers ]
      : [ '' ],
    title: concept?.title ?? ''
  };
}

export function cloneConceptEditorState( state ) {
  const fallback = createConceptEditorState();

  if ( !state || typeof state !== 'object' || Array.isArray( state ) ) {
    return fallback;
  }

  return {
    content: state.content?.schemaVersion === createEmptyConceptContent().schemaVersion
      ? cloneConceptContent( state.content )
      : fallback.content,
    deckIds: stringArray( state.deckIds ),
    explainFocus: typeof state.explainFocus === 'string'
      ? state.explainFocus
      : fallback.explainFocus,
    explainKeyPoints: stringArray( state.explainKeyPoints, [ '' ]),
    retrievalFormIds: stringArray( state.retrievalFormIds ),
    tagIds: stringArray( state.tagIds ),
    typeAnswerAcceptedAnswers: stringArray( state.typeAnswerAcceptedAnswers, [ '' ]),
    title: typeof state.title === 'string' ? state.title : ''
  };
}

export function conceptEditorStateKey( state ) {
  return JSON.stringify( cloneConceptEditorState( state ) );
}

export function conceptDraftMediaIds( state ) {
  const mediaIds = new Set();

  collectMediaIds( state?.content?.prompt, mediaIds );
  collectMediaIds( state?.content?.answer, mediaIds );

  return [ ...mediaIds ].sort();
}

function stringArray( value, fallback = []) {
  if ( !Array.isArray( value ) || value.some( ( item ) => typeof item !== 'string' ) ) {
    return [ ...fallback ];
  }

  return [ ...value ];
}

function collectMediaIds( node, mediaIds ) {
  if ( !node || typeof node !== 'object' ) {
    return;
  }

  if ( node.type === 'mediaImage' && typeof node.attrs?.mediaId === 'string' ) {
    mediaIds.add( node.attrs.mediaId );
  }

  if ( Array.isArray( node.content ) ) {
    for ( const child of node.content ) {
      collectMediaIds( child, mediaIds );
    }
  }
}

export function conceptRetrievalFormId( card ) {
  if ( card.retrievalKind === 'explain' ) {
    return 'explain';
  }

  if ( card.retrievalKind === 'cloze' ) {
    return 'cloze';
  }

  if ( card.retrievalKind === 'typeAnswer' ) {
    return 'type-answer';
  }

  if ( card.retrievalKind === 'imageOcclusion' ) {
    return 'image-occlusion';
  }

  return card.template?.id ?? STANDARD_RECALL_ID;
}
