import { getMarkRange } from '@tiptap/core';
import { closeHistory } from '@tiptap/pm/history';
import { TextSelection } from '@tiptap/pm/state';

const words = new Intl.Segmenter( undefined, { granularity: 'word' });

export function clozeEditingTarget( editor ) {
  const { doc, selection, schema } = editor.state;
  const markType = schema.marks.cloze;

  if ( !( selection instanceof TextSelection ) || !markType ) {
    return null;
  }

  let { from, to } = selection;

  if ( selection.empty ) {
    const range = getMarkRange( selection.$from, markType );

    if ( range ) {
      from = range.from;
      to = range.to;
    } else {
      const { parent, parentOffset } = selection.$from;

      if ( !parent.isTextblock || !parent.type.allowsMarkType( markType ) ) {
        return null;
      }

      const text = parent.textBetween( 0, parent.content.size, '', '\uFFFC' );
      const segments = Array.from( words.segment( text ) );
      const word = segments.find( segment => segment.isWordLike
        && segment.index <= parentOffset && parentOffset < segment.index + segment.segment.length )
        ?? segments.find( segment => segment.isWordLike
          && segment.index + segment.segment.length === parentOffset );

      if ( !word ) {
        return null;
      }

      from = selection.$from.start() + word.index;
      to = from + word.segment.length;
    }
  }

  const passages = [];
  const omissions = new Map();
  let unsupported = false;

  doc.nodesBetween( from, to, ( node, position, parent ) => {
    if ( !node.isText ) {
      unsupported ||= node.isAtom && node.type.name !== 'hardBreak';
      return;
    }

    const start = Math.max( from, position );
    const end = Math.min( to, position + node.nodeSize );
    const text = node.text.slice( start - position, end - position );

    if ( !text.trim() ) {
      return;
    }

    if ( !parent.type.allowsMarkType( markType )
      || node.marks.some( mark => mark.type !== markType && mark.type.excludes( markType ) ) ) {
      unsupported = true;
      return;
    }

    passages.push({ from: start, to: end });

    const mark = markType.isInSet( node.marks );

    if ( mark ) {
      const range = getMarkRange( doc.resolve( start ), markType, mark.attrs );

      if ( range ) {
        omissions.set( range.from, { ...range, groupId: mark.attrs.groupId });
      }
    }
  });

  if ( unsupported || !passages.length ) {
    return null;
  }

  return { passages, omissions: Array.from( omissions.values() ) };
}

export function changeClozeOmissions( editor, target, groupId = null ) {
  if ( !editor.isEditable || !target ) {
    return;
  }

  const markType = editor.schema.marks.cloze;
  const transaction = closeHistory( editor.state.tr );

  for ( const range of target.omissions ) {
    transaction.removeMark( range.from, range.to, markType );
  }

  if ( groupId ) {
    for ( const range of [ ...target.passages, ...target.omissions ]) {
      transaction.addMark( range.from, range.to, markType.create({ groupId }) );
    }
  }

  transaction.removeStoredMark( markType );
  editor.view.dispatch( transaction );
  editor.view.dispatch( closeHistory( editor.state.tr ) );
  editor.commands.focus();
}
