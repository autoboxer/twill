import { cardQualityKinds } from './options';

export { retrievalFormName as cardQualityFormName } from '../retrieval-forms/catalog';

export function cardQualityLabel( item ) {
  return item.source === 'reviewPattern'
    ? 'Repeated difficulty'
    : cardQualityKinds.find( ( kind ) => kind.value === item.kind )?.label;
}

export function cardQualityItemKey( item ) {
  return item.concernId ?? `${ item.cardId }-review-pattern`;
}
