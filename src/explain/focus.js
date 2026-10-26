const explanation = {
  reconstruction: false,
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
  }
};

export const explainFocusItems = Object.entries( focuses ).map( ([ value, focus ]) => ({
  value,
  label: focus.label
}) );

export function explainFocusDetails( focus ) {
  return focuses[ focus ] ?? focuses.why;
}
