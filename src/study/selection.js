import { parseLibraryQuery, serializeLibraryQuery } from '../library/search';

export function emptyStudySelection() {
  return {
    query: '',
    deckId: null,
    tagId: null,
    cardType: 'all',
    state: 'all',
    cardLimit: null
  };
}

export function studySelectionFromLibrary( query ) {
  const input = parseLibraryQuery( query );

  return {
    query: input.query,
    deckId: input.deckId,
    tagId: input.tagId,
    cardType: input.cardType,
    state: input.state === 'due' ? 'all' : input.state,
    cardLimit: null
  };
}

export function studyBuilderLocation( query ) {
  const input = {
    ...parseLibraryQuery({}),
    ...studySelectionFromLibrary( query )
  };

  return { name: 'study', query: { ...serializeLibraryQuery( input ), build: '1' } };
}

export function studyQuery( selection ) {
  return {
    ...selection,
    query: selection.query.trim(),
    cardType: selection.cardType === 'all' ? null : selection.cardType,
    state: selection.state === 'all' ? null : selection.state
  };
}

export function hasStudySelection( selection ) {
  return Boolean(
    selection.query.trim() || selection.deckId || selection.tagId
    || selection.cardType !== 'all' || selection.state !== 'all'
    || selection.cardLimit !== null
  );
}
