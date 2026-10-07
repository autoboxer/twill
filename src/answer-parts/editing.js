import { Fragment } from '@tiptap/pm/model';
import { closeHistory } from '@tiptap/pm/history';
import { TextSelection } from '@tiptap/pm/state';

import { createUuid } from '../lib/identifiers';
import { MAXIMUM_ANSWER_PARTS } from './documents';

export function answerPartEditingTarget( editor ) {
  const { doc, selection } = editor.state;
  const { from, to } = selection;
  const blocks = [];
  let partCount = 0;

  doc.forEach( ( node, position ) => {
    if ( node.type.name === 'answerPart' ) {
      partCount += 1;
    }

    if ( selection.empty
      ? position <= from && from < position + node.nodeSize
      : position < to && position + node.nodeSize > from ) {
      blocks.push({ node, position });
    }
  });

  if ( !blocks.length ) {
    return null;
  }

  if ( blocks.length === 1 && blocks[ 0 ].node.type.name === 'answerPart' ) {
    return { ...blocks[ 0 ], active: true };
  }

  if ( blocks.some( ( block ) => block.node.type.name === 'answerPart' )
    || partCount >= MAXIMUM_ANSWER_PARTS ) {
    return null;
  }

  return {
    active: false,
    position: blocks[ 0 ].position,
    end: blocks.at( -1 ).position + blocks.at( -1 ).node.nodeSize,
    content: Fragment.fromArray( blocks.map( ( block ) => block.node ) )
  };
}

export function toggleAnswerPart( editor ) {
  const target = answerPartEditingTarget( editor );

  if ( !target ) {
    return;
  }

  const transaction = closeHistory( editor.state.tr );
  const position = target.position;

  if ( target.active ) {
    transaction.replaceWith( position, position + target.node.nodeSize, target.node.content );
  } else {
    transaction.replaceWith( position, target.end, editor.schema.nodes.answerPart.create({
      id: createUuid(),
      groupId: createUuid()
    }, target.content ) );
  }

  transaction.setSelection( TextSelection.near( transaction.doc.resolve( position + 1 ) ) );
  editor.view.dispatch( transaction.scrollIntoView() );
  editor.view.dispatch( closeHistory( editor.state.tr ) );
  editor.view.focus();
}

export function setAnswerPartGroup( editor, id, groupId ) {
  editor.state.doc.forEach( ( node, position ) => {
    if ( node.type.name === 'answerPart' && node.attrs.id === id ) {
      editor.view.dispatch( closeHistory( editor.state.tr ).setNodeMarkup( position, undefined, {
        ...node.attrs,
        groupId
      }) );
      editor.view.dispatch( closeHistory( editor.state.tr ) );
    }
  });
}
