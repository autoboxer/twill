import { explainFocusDetails } from './focus';

const MAXIMUM_PROMPTS = 12;
const MAXIMUM_PROMPT_LENGTH = 280;

export function normalizeStructurePrompts( text ) {
  return text.split( /\r?\n/u ).map( line => line.trim().replace( /\s+/gu, ' ' ) ).filter( Boolean );
}

export function structurePromptsError( text ) {
  const prompts = normalizeStructurePrompts( text );

  if ( prompts.length > MAXIMUM_PROMPTS ) {
    return `Use ${ MAXIMUM_PROMPTS } structural prompts or fewer.`;
  }

  if ( prompts.some( prompt => Array.from( prompt ).length > MAXIMUM_PROMPT_LENGTH ) ) {
    return `Use ${ MAXIMUM_PROMPT_LENGTH } characters or fewer for each structural prompt.`;
  }

  if ( new Set( prompts.map( prompt => prompt.toLowerCase() ) ).size !== prompts.length ) {
    return 'Each structural prompt must be different.';
  }

  return '';
}

export function reconstructionPrompts( settings ) {
  return explainFocusDetails( settings?.focus ).reconstruction ? settings.structurePrompts ?? [] : [];
}
