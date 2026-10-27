const explanation = {
  reconstruction: false,
  generation: false,
  guide: 'Explanation guide',
  criteria: 'Key points',
  criterion: 'Key point',
  criterionPlaceholder: 'Expected idea',
  addCriterion: 'Add key point',
  response: 'Your explanation',
  inputLabel: 'Scratchpad',
  placeholder: 'Write an explanation',
  comparison: 'Compare your explanation',
  reveal: 'Compare explanation'
};

const reconstruction = {
  ...explanation,
  reconstruction: true,
  guide: 'Reference structure',
  inputLabel: 'Your reconstruction',
  response: 'Your reconstruction',
  comparison: 'Compare your reconstruction',
  reveal: 'Compare reconstruction'
};

const generation = {
  ...explanation,
  generation: true,
  guide: 'Example criteria',
  criteria: 'Criteria',
  criterion: 'Criterion',
  criterionPlaceholder: 'What a valid response must demonstrate',
  addCriterion: 'Add criterion'
};

const focuses = {
  why: { ...explanation, label: 'Why', instruction: 'Explain why' },
  how: { ...explanation, label: 'How', instruction: 'Explain how' },
  causeAndEffect: {
    ...explanation,
    label: 'Cause and effect',
    instruction: 'Explain the causes and effects'
  },
  compareAndContrast: {
    ...explanation,
    label: 'Compare and contrast',
    instruction: 'Compare and contrast'
  },
  outline: {
    ...reconstruction,
    label: 'Reconstruct an outline',
    instruction: 'Reconstruct the outline',
    criteria: 'Essential ideas',
    criterion: 'Essential idea',
    criterionPlaceholder: 'An idea and where it belongs in the outline',
    addCriterion: 'Add essential idea',
    placeholder: 'Write the main ideas and their supporting details.'
  },
  sequence: {
    ...reconstruction,
    label: 'Reconstruct a sequence',
    instruction: 'Reconstruct the sequence',
    criteria: 'Reference sequence',
    criterion: 'Step',
    criterionPlaceholder: 'A step in the expected order',
    addCriterion: 'Add step',
    placeholder: 'Write the steps in order, with one step on each line.'
  },
  relationships: {
    ...reconstruction,
    label: 'Reconstruct relationships',
    instruction: 'Reconstruct the relationships',
    criteria: 'Essential relationships',
    criterion: 'Relationship',
    criterionPlaceholder: 'Two ideas and how they are connected',
    addCriterion: 'Add relationship',
    placeholder: 'Name the ideas or parts and describe how they connect.'
  },
  example: {
    ...generation,
    label: 'Create an example',
    instruction: 'Create an example',
    authoring: 'Describe the rule and task in Prompt. In Answer, give a checked example and explain why it fits.',
    inputLabel: 'Your example',
    response: 'Your example',
    placeholder: 'Describe a specific case and explain how it illustrates the rule.',
    comparison: 'Compare your example',
    reveal: 'Compare example'
  },
  nonExample: {
    ...generation,
    label: 'Create a non-example',
    instruction: 'Create a non-example',
    authoring: 'Describe the rule and task in Prompt. In Answer, give a checked non-example and explain which condition it fails.',
    inputLabel: 'Your non-example',
    response: 'Your non-example',
    placeholder: 'Describe a case that does not fit and explain which condition it fails.',
    comparison: 'Compare your non-example',
    reveal: 'Compare non-example'
  },
  changedSituation: {
    ...generation,
    label: 'Change the situation',
    instruction: 'Change the situation',
    authoring: 'Describe the original situation and task in Prompt. In Answer, give a checked change and explain its effect.',
    inputLabel: 'Your changed situation',
    response: 'Your changed situation',
    placeholder: 'Change a condition and explain how it changes the result.',
    comparison: 'Compare your changed situation',
    reveal: 'Compare situation'
  }
};

export const explainFocusItems = Object.entries( focuses ).map( ([ value, focus ]) => ({
  value,
  label: focus.label
}) );

export function explainFocusDetails( focus ) {
  return focuses[ focus ] ?? focuses.why;
}
