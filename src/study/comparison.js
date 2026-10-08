import { collectAnswerParts, supportsAnswerParts } from '../answer-parts/documents';

export const comparisonChoices = [
  { label: 'Not checked', value: 'unchecked' },
  { label: 'Present', value: 'present' },
  { label: 'Incomplete', value: 'partial' },
  { label: 'Missing', value: 'missing' }
];

const recallComparisonChoices = [
  { label: 'Not checked', value: 'unchecked' },
  { label: 'Remembered', value: 'present' },
  { label: 'Partially remembered', value: 'partial' },
  { label: 'Forgot', value: 'missing' }
];

export function comparisonChoicesForResponse( writtenResponse ) {
  return writtenResponse ? comparisonChoices : recallComparisonChoices;
}

export function comparisonTargetIds( card ) {
  const ids = [];

  if ( card?.retrievalKind === 'explain' ) {
    const keyPoints = card.explain?.keyPoints ?? [];

    ids.push( ...keyPoints.map( ( _, index ) => `idea:${ index }` ) );
  }

  if ( card?.retrievalKind === 'problem' ) {
    const checkpoints = card.problem?.checkpoints ?? [];

    ids.push( ...checkpoints.map( ( _, index ) => `checkpoint:${ index }` ) );
  }

  if ( !ids.length && supportsAnswerParts( card ) ) {
    ids.push( ...collectAnswerParts( card.content.answer ).map( ( part ) => `part:${ part.id }` ) );
  }

  return ids;
}

export function usesAnswerPartComparison( card ) {
  return comparisonTargetIds( card ).some( ( id ) => id.startsWith( 'part:' ) );
}
