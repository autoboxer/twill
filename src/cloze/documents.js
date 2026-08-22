const CLOZE_MARK = 'cloze';

export const MAXIMUM_CLOZE_GROUPS = 100;

export function collectClozeGroups( document ) {
  const groups = [];
  const groupsById = new Map();

  collectPassages( document );

  for ( const group of groups ) {
    group.passages = group.passages
      .map( ( passage ) => passage.trim() )
      .filter( Boolean );
  }

  return groups;

  function collectPassages( node ) {
    let activeGroupId = '';

    for ( const child of ( node?.content ?? []) ) {
      if ( child.type !== 'text' ) {
        activeGroupId = '';
        collectPassages( child );
        continue;
      }

      const mark = child.marks?.find( ( candidate ) => candidate.type === CLOZE_MARK );
      const groupId = mark?.attrs?.groupId ?? '';

      if ( !groupId ) {
        activeGroupId = '';
        continue;
      }

      let group = groupsById.get( groupId );

      if ( !group ) {
        group = {
          id: groupId,
          passages: []
        };

        groupsById.set( groupId, group );
        groups.push( group );
      }

      if ( activeGroupId === groupId ) {
        group.passages[ group.passages.length - 1 ] += child.text;
      } else {
        group.passages.push( child.text );
      }

      activeGroupId = groupId;
    }
  }
}

export function createClozeGroupId() {
  if ( typeof globalThis.crypto?.randomUUID === 'function' ) {
    return globalThis.crypto.randomUUID();
  }

  const bytes = globalThis.crypto.getRandomValues( new Uint8Array( 16 ) );

  bytes[ 6 ] = ( bytes[ 6 ] & 0x0f ) | 0x40;
  bytes[ 8 ] = ( bytes[ 8 ] & 0x3f ) | 0x80;

  const hexadecimal = Array.from(
    bytes,
    ( value ) => value.toString( 16 ).padStart( 2, '0' )
  ).join( '' );

  return [
    hexadecimal.slice( 0, 8 ),
    hexadecimal.slice( 8, 12 ),
    hexadecimal.slice( 12, 16 ),
    hexadecimal.slice( 16, 20 ),
    hexadecimal.slice( 20 )
  ].join( '-' );
}

export function createClozePrompt( document, activeGroupId, revealed = false ) {
  return transformDocument( document, ( node ) => {
    const clozeMark = node.marks?.find( ( mark ) => mark.type === CLOZE_MARK );
    const marks = node.marks?.filter( ( mark ) => mark.type !== CLOZE_MARK );

    if ( clozeMark?.attrs?.groupId === activeGroupId ) {
      if ( !revealed ) {
        return { type: 'clozeBlank' };
      }

      return {
        ...node,
        marks: [
          ...( marks ?? []),
          clozeMark
        ]
      };
    }

    return withMarks( node, marks );
  });
}

export function removeAllClozeMarks( document ) {
  return transformDocument( document, ( node ) => {
    const marks = node.marks?.filter( ( mark ) => mark.type !== CLOZE_MARK );

    return withMarks( node, marks );
  });
}

function transformDocument( document, transformText ) {
  function transformNode( node ) {
    if ( node.type === 'text' ) {
      return transformText( node );
    }

    if ( !Array.isArray( node.content ) ) {
      return { ...node };
    }

    const content = [];
    let blankPending = false;

    for ( const child of node.content ) {
      const transformed = transformNode( child );

      if ( transformed.type === 'clozeBlank' ) {
        if ( !blankPending ) {
          content.push({
            type: 'clozeBlank'
          });
        }

        blankPending = true;
        continue;
      }

      blankPending = false;
      content.push( transformed );
    }

    return {
      ...node,
      content
    };
  }

  return transformNode( document );
}

function withMarks( node, marks ) {
  const transformed = { ...node };

  if ( marks?.length ) {
    transformed.marks = marks;
  } else {
    delete transformed.marks;
  }

  return transformed;
}
