import { collectClozeGroups } from '../cloze/documents';
import { collectImageOcclusionGroups } from '../image-occlusion/documents';
import { retrievalFormLabel } from '../retrieval-forms/catalog';
import { cardQualityKinds } from './options';

export function cardQualityLabel( item ) {
  return item.source === 'reviewPattern'
    ? 'Repeated difficulty'
    : cardQualityKinds.find( ( kind ) => kind.value === item.kind )?.label;
}

export function cardQualityFormName( card, prompt = null ) {
  const name = retrievalFormLabel( card );
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
