import { createUuid } from '../lib/identifiers';

export const MAXIMUM_IMAGE_LABEL_TARGETS = 100;
export const MAXIMUM_LABEL_LENGTH = 200;
export const MAXIMUM_LABEL_ALTERNATIVES = 20;
export const MAXIMUM_TARGET_DESCRIPTION_LENGTH = 500;

export function imageLabelTargets( node ) {
  return Array.isArray( node?.attrs?.labelTargets ) ? node.attrs.labelTargets : [];
}

export function collectImageLabelTargets( document ) {
  const targets = [];

  function visit( node ) {
    for ( const target of imageLabelTargets( node ) ) {
      targets.push({ ...target, image: node.attrs });
    }

    for ( const child of node?.content ?? []) {
      visit( child );
    }
  }

  visit( document );
  return targets;
}

export function createImageLabelTarget() {
  return { id: createUuid(), x: 0.5, y: 0.5, label: '', alternatives: [], description: '' };
}

export function removeAllImageLabelTargets( document ) {
  function visit( node ) {
    return {
      ...node,
      ...( node.type === 'mediaImage' ? { attrs: { ...node.attrs, labelTargets: [] } } : {}),
      ...( node.content ? { content: node.content.map( visit ) } : {})
    };
  }

  return visit( document );
}

export function cloneImageLabelTargets( targets ) {
  return targets.map( target => ({ ...target, alternatives: [ ...target.alternatives ] }) );
}

export function normalizeImageLabelTargets( targets ) {
  return targets.map( target => ({
    ...target,
    label: target.label.trim(),
    alternatives: target.alternatives.map( answer => answer.trim() ).filter( Boolean ),
    description: target.description.trim()
  }) );
}

export function imageLabelErrors( target ) {
  const label = target.label.trim();
  const alternatives = target.alternatives.map( answer => answer.trim() ).filter( Boolean );
  const answers = [ label, ...alternatives ].map( answer => answer.replace( /\s+/gu, ' ' ).toLowerCase() );

  return {
    label: !label ? 'Enter the expected label.'
      : Array.from( label ).length > MAXIMUM_LABEL_LENGTH ? 'Use 200 characters or fewer.' : '',
    alternatives: alternatives.length > MAXIMUM_LABEL_ALTERNATIVES
      ? 'Use 20 alternatives or fewer.'
      : alternatives.some( answer => Array.from( answer ).length > MAXIMUM_LABEL_LENGTH )
        ? 'Use 200 characters or fewer for each alternative.'
        : new Set( answers ).size !== answers.length ? 'Each accepted answer must be different.' : '',
    description: Array.from( target.description.trim() ).length > MAXIMUM_TARGET_DESCRIPTION_LENGTH
      ? 'Use 500 characters or fewer.' : ''
  };
}

export function imageTargetDescription( target, number ) {
  return `Target ${ number }. ${ target.description || `Position ${ Math.round( target.x * 100 ) }% from the left, ${ Math.round( target.y * 100 ) }% from the top.` }`;
}
