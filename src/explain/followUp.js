import { explainFocusDetails } from './focus';

export const FOLLOW_UP_PROMPT_LIMIT = 500;
export const FOLLOW_UP_REFERENCE_LIMIT = 2000;

export function cloneExampleFollowUp( value ) {
  if ( !value || typeof value !== 'object' || Array.isArray( value ) ) {
    return null;
  }

  return {
    prompt: typeof value.prompt === 'string' ? value.prompt : '',
    reference: typeof value.reference === 'string' ? value.reference : ''
  };
}

export function normalizeExampleFollowUp( value ) {
  const followUp = cloneExampleFollowUp( value );

  return followUp ? { prompt: followUp.prompt.trim(), reference: followUp.reference.trim() } : null;
}

export function exampleFollowUpError( value ) {
  const followUp = normalizeExampleFollowUp( value );

  if ( !followUp ) {
    return '';
  }

  for ( const [ text, label, maximum ] of [
    [ followUp.prompt, 'question', FOLLOW_UP_PROMPT_LIMIT ],
    [ followUp.reference, 'reference', FOLLOW_UP_REFERENCE_LIMIT ]
  ]) {
    if ( !text ) {
      return `Enter a follow-up ${ label }.`;
    }

    if ( Array.from( text ).length > maximum ) {
      return `Use ${ maximum } characters or fewer for the follow-up ${ label }.`;
    }
  }

  return '';
}

export function exampleFollowUp( settings ) {
  if ( !explainFocusDetails( settings?.focus ).generation || exampleFollowUpError( settings.followUp ) ) {
    return null;
  }

  return normalizeExampleFollowUp( settings.followUp );
}
