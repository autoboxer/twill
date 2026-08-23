import { createUuid } from '../lib/identifiers';

export const MAXIMUM_IMAGE_OCCLUSION_GROUPS = 100;
export const MAXIMUM_IMAGE_OCCLUSION_REGIONS = 500;

export function collectImageOcclusionGroups( document ) {
  const groups = [];
  const groupsById = new Map();

  visitNodes( document, ( node ) => {
    if ( node.type !== 'mediaImage' ) {
      return;
    }

    const image = {
      alt: node.attrs?.alt ?? '',
      mediaId: node.attrs?.mediaId ?? '',
      title: node.attrs?.title ?? ''
    };

    for ( const region of imageOcclusionRegions( node ) ) {
      let group = groupsById.get( region.groupId );

      if ( !group ) {
        group = {
          id: region.groupId,
          image,
          regions: []
        };

        groupsById.set( region.groupId, group );
        groups.push( group );
      }

      group.regions.push({ ...region });
    }
  });

  return groups;
}

export function createImageOcclusionId() {
  return createUuid();
}

export function imageOcclusionRegions( node ) {
  return Array.isArray( node?.attrs?.occlusionRegions )
    ? node.attrs.occlusionRegions
    : [];
}

export function imageOcclusionGroupIds( regions ) {
  return [ ...new Set( regions.map( ( region ) => region.groupId ).filter( Boolean ) ) ];
}

export function removeAllImageOcclusionRegions( document ) {
  return transformNodes( document, ( node ) => {
    if ( node.type !== 'mediaImage' ) {
      return node;
    }

    return {
      ...node,
      attrs: {
        ...node.attrs,
        occlusionRegions: []
      }
    };
  });
}

function transformNodes( node, transform ) {
  const transformed = transform({ ...node });

  if ( !Array.isArray( transformed.content ) ) {
    return transformed;
  }

  return {
    ...transformed,
    content: transformed.content.map( ( child ) => transformNodes( child, transform ) )
  };
}

function visitNodes( node, visitor ) {
  visitor( node );

  for ( const child of ( node?.content ?? []) ) {
    visitNodes( child, visitor );
  }
}
