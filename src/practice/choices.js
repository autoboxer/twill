import { createUuid } from '../lib/identifiers';

export function emptyChoice() {
  return {
    options: [ emptyChoiceOption(), emptyChoiceOption() ],
    correctOptionId: '',
    shuffle: true
  };
}

export function emptyChoiceOption() {
  return { id: createUuid(), text: '', feedback: '' };
}

export function choiceError( choice ) {
  if ( !choice ) {
    return '';
  }

  if ( choice.options.length < 2 || choice.options.length > 6 ) {
    return 'Provide two to six answer options.';
  }

  const labels = new Set();

  for ( const option of choice.options ) {
    const text = option.text.trim();
    const feedback = option.feedback.trim();

    if ( !text || Array.from( text ).length > 300 ) {
      return 'Write each option in 1 to 300 characters.';
    }

    if ( !feedback || Array.from( feedback ).length > 1_000 ) {
      return 'Write feedback for each option in 1 to 1,000 characters.';
    }

    const label = text.toLowerCase().replace( /\s+/g, ' ' );

    if ( labels.has( label ) ) {
      return 'Give each answer option different text.';
    }

    labels.add( label );
  }

  return choice.options.some( ( option ) => option.id === choice.correctOptionId )
    ? '' : 'Choose the correct answer.';
}

export function createChoiceState( choice ) {
  if ( !choice ) {
    return null;
  }

  const optionIds = choice.options.map( ( option ) => option.id );

  if ( choice.shuffle ) {
    for ( let index = optionIds.length - 1; index > 0; index -= 1 ) {
      const other = Math.floor( Math.random() * ( index + 1 ) );
      [ optionIds[ index ], optionIds[ other ] ] = [ optionIds[ other ], optionIds[ index ] ];
    }
  }

  return { optionIds, selectedId: '', submittedId: '' };
}
