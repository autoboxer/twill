import { retrievalForms } from '../retrieval-forms/catalog';

export const libraryCardTypeOptions = [
  { label: 'All types', value: 'all' },
  ...Object.entries( retrievalForms ).map( ([ value, form ]) => ({
    label: value === 'recall' ? 'Recall' : form.label,
    value
  }) )
];

export const libraryStateOptions = [
  { label: 'All states', value: 'all' },
  { label: 'New', value: 'new' },
  { label: 'Learning', value: 'learning' },
  { label: 'Review', value: 'review' },
  { label: 'Relearning', value: 'relearning' },
  { label: 'Due now', value: 'due' }
];

export const librarySortOptions = [
  { label: 'Relevance', value: 'relevance' },
  { label: 'Title', value: 'title' },
  { label: 'Recently updated', value: 'updated' }
];

function text( value, maximum = 250 ) {
  return typeof value === 'string' ? Array.from( value ).slice( 0, maximum ).join( '' ) : '';
}

function choice( value, options ) {
  return options.some( ( option ) => option.value === value ) ? value : options[ 0 ].value;
}

export function parseLibraryQuery( query ) {
  const page = typeof query.page === 'string' ? Number( query.page ) : 1;

  return {
    query: text( query.q ),
    deckId: text( query.deck ).trim() || null,
    tagId: text( query.tag ).trim() || null,
    cardType: choice( query.type, libraryCardTypeOptions ),
    state: choice( query.state, libraryStateOptions ),
    sort: choice( query.sort, librarySortOptions ),
    includeArchived: query.archived === '1',
    page: Number.isInteger( page ) && page > 0 && page <= 4_294_967_295 ? page : 1
  };
}

export function serializeLibraryQuery( input ) {
  const query = {};

  if ( input.query ) {
    query.q = input.query;
  }

  if ( input.deckId ) {
    query.deck = input.deckId;
  }

  if ( input.tagId ) {
    query.tag = input.tagId;
  }

  if ( input.cardType !== 'all' ) {
    query.type = input.cardType;
  }

  if ( input.state !== 'all' ) {
    query.state = input.state;
  }

  if ( input.sort !== 'relevance' ) {
    query.sort = input.sort;
  }

  if ( input.includeArchived ) {
    query.archived = '1';
  }

  if ( input.page > 1 ) {
    query.page = String( input.page );
  }

  return query;
}

export function libraryNavigationQuery( query, includeCard = false ) {
  const context = serializeLibraryQuery( parseLibraryQuery( query ) );
  const card = text( query.card ).trim();

  if ( includeCard && card ) {
    context.card = card;
  }

  return context;
}
