export const MAXIMUM_ANSWER_PARTS = 100;

export function supportsAnswerParts( card ) {
  return Boolean( card && !card.template && collectAnswerParts( card.content.answer ).length );
}

export function collectAnswerParts( document ) {
  return ( document?.content ?? [])
    .filter( ( node ) => node.type === 'answerPart' )
    .map( ( node, index ) => ({
      id: node.attrs.id,
      groupId: node.attrs.groupId,
      label: `Part ${ index + 1 }`,
      document: { type: 'doc', content: node.content }
    }) );
}

export function answerPartRevealGroups( document ) {
  const parts = collectAnswerParts( document );
  const groups = new Map();

  for ( const part of parts ) {
    if ( !groups.has( part.groupId ) ) {
      groups.set( part.groupId, {
        id: part.groupId,
        default: false,
        parts: []
      });
    }

    groups.get( part.groupId ).parts.push( part );
  }

  if ( parts.length && groups.size === parts.length ) {
    return [{ id: parts[ 0 ].groupId, default: true, parts }];
  }

  return [ ...groups.values() ];
}

export function answerPartSections( document ) {
  const parts = collectAnswerParts( document );
  const sections = [];

  for ( const node of document.content ) {
    if ( node.type === 'answerPart' ) {
      const part = parts.find( ( value ) => value.id === node.attrs.id );

      sections.push({ ...part, key: part.id });
    } else {
      const previous = sections.at( -1 );

      if ( previous && !previous.id ) {
        previous.document.content.push( node );
      } else {
        sections.push({
          key: `context-${ sections.length }`,
          document: { type: 'doc', content: [ node ] }
        });
      }
    }
  }

  return sections;
}
