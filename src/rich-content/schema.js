import { Mark, mergeAttributes, Node } from '@tiptap/core';
import CodeBlockLowlight from '@tiptap/extension-code-block-lowlight';
import Mathematics from '@tiptap/extension-mathematics';
import { VueNodeViewRenderer } from '@tiptap/vue-3';
import bash from 'highlight.js/lib/languages/bash';
import css from 'highlight.js/lib/languages/css';
import javascript from 'highlight.js/lib/languages/javascript';
import json from 'highlight.js/lib/languages/json';
import markdown from 'highlight.js/lib/languages/markdown';
import plaintext from 'highlight.js/lib/languages/plaintext';
import python from 'highlight.js/lib/languages/python';
import rust from 'highlight.js/lib/languages/rust';
import sql from 'highlight.js/lib/languages/sql';
import typescript from 'highlight.js/lib/languages/typescript';
import xml from 'highlight.js/lib/languages/xml';
import { createLowlight } from 'lowlight';

import { normalizeCodeLanguage } from './codeLanguages';

import RichContentCodeBlock from '../components/RichContentCodeBlock.vue';
import RichContentImage from '../components/RichContentImage.vue';

export const RICH_CONTENT_SCHEMA_VERSION = 1;

const lowlight = createLowlight();

lowlight.register({
  bash,
  css,
  javascript,
  json,
  markdown,
  plaintext,
  python,
  rust,
  sql,
  typescript,
  xml
});

// Tiptap falls back to highlightAuto for unknown languages; keep that fallback plain
const codeHighlighter = {
  ...lowlight,
  highlight: ( language, code ) => lowlight.highlight( normalizeCodeLanguage( language ), code ),
  highlightAuto: code => lowlight.highlight( 'plaintext', code ),
  registered: language => lowlight.registered( normalizeCodeLanguage( language ) )
};

const MediaImage = Node.create({
  name: 'mediaImage',
  group: 'block',
  atom: true,
  draggable: true,
  isolating: true,

  addOptions() {
    return {
      imageOcclusionDocument: null,
      imageOcclusionDisplay: null,
      imageOcclusionEnabled: false
    };
  },

  addAttributes() {
    return {
      mediaId: {
        default: null,
        parseHTML: ( element ) => element.getAttribute( 'data-media-id' ),
        renderHTML: ( attributes ) => ({
          'data-media-id': attributes.mediaId
        })
      },
      alt: {
        default: null,
        parseHTML: ( element ) => element.getAttribute( 'data-alt' ),
        renderHTML: ( attributes ) => attributes.alt
          ? { 'data-alt': attributes.alt }
          : {}
      },
      title: {
        default: null,
        parseHTML: ( element ) => element.getAttribute( 'data-title' ),
        renderHTML: ( attributes ) => attributes.title
          ? { 'data-title': attributes.title }
          : {}
      },
      occlusionRegions: {
        default: [],
        rendered: false
      }
    };
  },

  parseHTML() {
    return [{ tag: 'figure[data-type="media-image"]' }];
  },

  renderHTML({ HTMLAttributes }) {
    return [
      'figure',
      mergeAttributes( HTMLAttributes, { 'data-type': 'media-image' })
    ];
  },

  addNodeView() {
    return VueNodeViewRenderer( RichContentImage );
  }
});

const Cloze = Mark.create({
  name: 'cloze',
  exitable: true,
  inclusive: false,

  addAttributes() {
    return {
      groupId: {
        default: null,
        parseHTML: ( element ) => element.getAttribute( 'data-cloze-group' ),
        renderHTML: ( attributes ) => attributes.groupId
          ? { 'data-cloze-group': attributes.groupId }
          : {}
      }
    };
  },

  parseHTML() {
    return [{ tag: 'span[data-cloze-group]' }];
  },

  renderHTML({ HTMLAttributes }) {
    return [
      'span',
      mergeAttributes( HTMLAttributes, { 'data-type': 'cloze' }),
      0
    ];
  }
});

const ClozeBlank = Node.create({
  name: 'clozeBlank',
  group: 'inline',
  inline: true,
  atom: true,
  selectable: false,

  parseHTML() {
    return [{ tag: 'span[data-type="cloze-blank"]' }];
  },

  renderHTML() {
    return [
      'span',
      {
        'aria-label': 'Missing passage',
        'data-type': 'cloze-blank',
        role: 'img'
      },
      '[…]'
    ];
  }
});

export function createEmptyRichDocument() {
  return {
    type: 'doc',
    content: [{ type: 'paragraph' }]
  };
}

export function createEmptyAnswerFeedback() {
  return {
    explanation: createEmptyRichDocument(),
    commonMistakes: createEmptyRichDocument()
  };
}

export function createEmptyConceptContent() {
  return {
    schemaVersion: RICH_CONTENT_SCHEMA_VERSION,
    prompt: createEmptyRichDocument(),
    answer: createEmptyRichDocument(),
    feedback: createEmptyAnswerFeedback()
  };
}

export function cloneConceptContent( content ) {
  const fallback = createEmptyConceptContent();

  return {
    schemaVersion: content?.schemaVersion ?? fallback.schemaVersion,
    prompt: cloneRichDocument( content?.prompt ?? fallback.prompt ),
    answer: cloneRichDocument( content?.answer ?? fallback.answer ),
    feedback: {
      explanation: cloneRichDocument(
        content?.feedback?.explanation ?? fallback.feedback.explanation
      ),
      commonMistakes: cloneRichDocument(
        content?.feedback?.commonMistakes ?? fallback.feedback.commonMistakes
      )
    }
  };
}

function cloneRichDocument( document ) {
  return JSON.parse( JSON.stringify( document ) );
}

export function richDocumentHasContent( document ) {
  function nodeHasContent( node ) {
    if ( node?.type === 'text' ) {
      return Boolean( node.text?.trim() );
    }

    if ([ 'inlineMath', 'blockMath' ].includes( node?.type ) ) {
      return Boolean( node.attrs?.latex?.trim() );
    }

    if ( node?.type === 'mediaImage' ) {
      return true;
    }

    return Array.isArray( node?.content )
      && node.content.some( nodeHasContent );
  }

  return nodeHasContent( document );
}

export function createRichContentExtensions({
  codeBlockEditingEnabled = null,
  imageOcclusionDocument = null,
  imageOcclusionDisplay = null,
  imageOcclusionEnabled = false,
  onEditMath
} = {}) {
  const editInlineMath = onEditMath
    ? ( node, position ) => onEditMath({
      latex: node.attrs.latex,
      mode: 'inline',
      position
    })
    : undefined;
  const editBlockMath = onEditMath
    ? ( node, position ) => onEditMath({
      latex: node.attrs.latex,
      mode: 'block',
      position
    })
    : undefined;

  return [
    CodeBlockLowlight.extend({
      addOptions() {
        return {
          ...this.parent?.(),
          editingEnabled: codeBlockEditingEnabled
        };
      },

      addNodeView() {
        return codeBlockEditingEnabled ? VueNodeViewRenderer( RichContentCodeBlock ) : null;
      }
    }).configure({
      defaultLanguage: 'plaintext',
      lowlight: codeHighlighter,
      enableTabIndentation: false
    }),
    Mathematics.configure({
      inlineOptions: {
        onClick: editInlineMath
      },
      blockOptions: {
        onClick: editBlockMath
      },
      katexOptions: {
        maxExpand: 1_000,
        maxSize: 20,
        strict: 'warn',
        throwOnError: false,
        trust: false
      }
    }),
    Cloze,
    ClozeBlank,
    MediaImage.configure({
      imageOcclusionDocument,
      imageOcclusionDisplay,
      imageOcclusionEnabled
    })
  ];
}

export function highlightCode( code, language ) {
  return codeHighlighter.highlight( language, code ).children;
}

export function richContentStarterKit( editable = true ) {
  return {
    codeBlock: false,
    heading: {
      levels: [ 1, 2, 3 ]
    },
    link: {
      autolink: true,
      linkOnPaste: true,
      openOnClick: !editable,
      protocols: [ 'http', 'https', 'mailto' ]
    }
  };
}
