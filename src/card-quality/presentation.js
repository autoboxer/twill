import { collectClozeGroups } from '../cloze/documents';
import { collectImageOcclusionGroups } from '../image-occlusion/documents';
import { cardQualityKinds } from './options';

export function cardQualityLabel( item ) {
  return item.source === 'reviewPattern'
    ? 'Repeated difficulty'
    : cardQualityKinds.find( ( kind ) => kind.value === item.kind )?.label;
}

export function cardQualityFormName( card, prompt = null ) {
  const names = {
    recall: 'Standard recall',
    typeAnswer: 'Type answer',
    explain: 'Explain',
    problem: 'Problem',
    cloze: 'Cloze',
    imageOcclusion: 'Image occlusion'
  };
  const name = card.template?.name ?? names[ card.retrievalKind ];
  let groupIndex = -1;

  if ( prompt && card.cloze ) {
    groupIndex = collectClozeGroups( prompt )
      .findIndex( ( group ) => group.id === card.cloze.groupId );
  } else if ( prompt && card.imageOcclusion ) {
    groupIndex = collectImageOcclusionGroups( prompt )
      .findIndex( ( group ) => group.id === card.imageOcclusion.groupId );
  }

  return groupIndex < 0 ? name : `${ name } ${ groupIndex + 1 }`;
}

export function cardQualityItemKey( item ) {
  return item.concernId ?? `${ item.cardId }-review-pattern`;
}
