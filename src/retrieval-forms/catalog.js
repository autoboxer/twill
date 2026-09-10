export const retrievalForms = {
  recall: {
    label: 'Standard recall',
    icon: 'i-lucide-rotate-ccw'
  },
  typeAnswer: {
    label: 'Type answer',
    icon: 'i-lucide-keyboard'
  },
  explain: {
    label: 'Explain',
    icon: 'i-lucide-message-square-text'
  },
  problem: {
    label: 'Problem',
    icon: 'i-lucide-list-ordered'
  },
  cloze: {
    label: 'Cloze',
    icon: 'i-lucide-text-select'
  },
  imageOcclusion: {
    label: 'Image occlusion',
    icon: 'i-lucide-scan'
  }
};

export function retrievalFormLabel( card ) {
  return card.template?.name ?? retrievalForms[ card.retrievalKind ]?.label;
}

export function retrievalFormIcon( card ) {
  return card.template
    ? 'i-lucide-panels-top-left'
    : retrievalForms[ card.retrievalKind ]?.icon;
}
