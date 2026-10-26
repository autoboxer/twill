import { collectClozeGroups } from '../cloze/documents';
import { collectImageOcclusionGroups } from '../image-occlusion/documents';
import { collectImageLabelTargets } from '../image-labeling/targets';

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
  },
  imageLabeling: {
    label: 'Image labeling',
    icon: 'i-lucide-map-pin'
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

export function retrievalFormName( card, prompt = null ) {
  const name = retrievalFormLabel( card );
  let targetIndex = -1;

  if ( prompt && card.cloze ) {
    targetIndex = collectClozeGroups( prompt )
      .findIndex( group => group.id === card.cloze.groupId );
  } else if ( prompt && card.imageOcclusion ) {
    targetIndex = collectImageOcclusionGroups( prompt )
      .findIndex( group => group.id === card.imageOcclusion.groupId );
  } else if ( prompt && card.imageLabeling ) {
    targetIndex = collectImageLabelTargets( prompt )
      .findIndex( target => target.id === card.imageLabeling.targetId );
  }

  return targetIndex < 0 ? name : `${ name } ${ targetIndex + 1 }`;
}
