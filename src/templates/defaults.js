export const MAXIMUM_TEMPLATE_BLOCKS_PER_SIDE = 50;

export const templateFields = [
  {
    description: 'The short name used to identify the concept.',
    icon: 'i-lucide-heading',
    label: 'Title',
    value: 'title'
  },

  {
    description: 'The question or cue written on the concept.',
    icon: 'i-lucide-message-circle-question',
    label: 'Prompt',
    value: 'prompt'
  },

  {
    description: 'The response revealed after an attempt.',
    icon: 'i-lucide-message-circle-check',
    label: 'Answer',
    value: 'answer'
  }
];

export function createDefaultTemplateContent() {
  return {
    schemaVersion: 1,
    mode: 'visual',
    visual: {
      front: {
        blocks: [
          { type: 'field', field: 'prompt' }
        ]
      },
      answer: {
        blocks: [
          { type: 'field', field: 'prompt' },
          { type: 'field', field: 'answer' }
        ]
      },
      appearance: {
        alignment: 'left',
        showFieldLabels: true
      }
    },
    custom: {
      frontHtml: [
        '<main class="card">',
        '  {{ prompt }}',
        '</main>'
      ].join( '\n' ),
      answerHtml: [
        '<main class="card">',
        '  <section class="prompt">{{ prompt }}</section>',
        '  <hr>',
        '  <section class="answer">{{ answer }}</section>',
        '</main>'
      ].join( '\n' ),
      css: [
        '.card {',
        '  max-width: 42rem;',
        '  margin: 0 auto;',
        '  color: #172019;',
        '  font-family: system-ui, sans-serif;',
        '  line-height: 1.6;',
        '}',
        '',
        'hr {',
        '  margin: 2rem 0;',
        '  border: 0;',
        '  border-top: 1px solid #d7ded5;',
        '}'
      ].join( '\n' )
    }
  };
}

export function cloneTemplateContent( content ) {
  return JSON.parse( JSON.stringify( content ?? createDefaultTemplateContent() ) );
}
