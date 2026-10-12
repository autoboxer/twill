import { createUuid } from '../lib/identifiers';

export const MAXIMUM_WORKED_STEPS = 12;
export const MAXIMUM_STEP_PROMPT_LENGTH = 280;
export const MAXIMUM_STEP_SOLUTION_LENGTH = 2000;
export const MAXIMUM_PRINCIPLE_PROMPT_LENGTH = 500;

export function createWorkedStep() {
  return {
    id: createUuid(),
    prompt: '',
    solution: '',
    principlePrompt: '',
    showInCompletion: false
  };
}

export function cloneWorkedExample( example ) {
  if ( !example || !Array.isArray( example.steps ) ) {
    return null;
  }

  return {
    steps: example.steps.map( ( step ) => ({
      id: typeof step?.id === 'string' ? step.id : createUuid(),
      prompt: textValue( step?.prompt ),
      solution: textValue( step?.solution ),
      principlePrompt: textValue( step?.principlePrompt ),
      showInCompletion: step?.showInCompletion === true
    }) )
  };
}

export function normalizeWorkedExample( example ) {
  const cloned = cloneWorkedExample( example );

  if ( !cloned ) {
    return null;
  }

  return {
    steps: cloned.steps.map( ( step ) => ({
      ...step,
      prompt: step.prompt.trim(),
      solution: step.solution.trim(),
      principlePrompt: step.principlePrompt.trim()
    }) )
  };
}

export function workedStepErrors( step ) {
  return {
    prompt: fieldError( step.prompt, 'Enter the step to complete.', MAXIMUM_STEP_PROMPT_LENGTH ),
    solution: fieldError( step.solution, 'Enter the checked solution.', MAXIMUM_STEP_SOLUTION_LENGTH ),
    principlePrompt: fieldError( step.principlePrompt, '', MAXIMUM_PRINCIPLE_PROMPT_LENGTH )
  };
}

export function workedExampleError( example ) {
  if ( !example ) {
    return '';
  }

  if ( !example.steps.length || example.steps.length > MAXIMUM_WORKED_STEPS ) {
    return `Use between 1 and ${ MAXIMUM_WORKED_STEPS } worked steps.`;
  }

  return example.steps.flatMap( ( step ) => Object.values( workedStepErrors( step ) ) )
    .find( Boolean ) ?? '';
}

function textValue( value ) {
  return typeof value === 'string' ? value : '';
}

function fieldError( value, emptyMessage, maximum ) {
  const text = value.trim();

  if ( !text && emptyMessage ) {
    return emptyMessage;
  }

  return Array.from( text ).length > maximum
    ? `Use ${ maximum } characters or fewer.`
    : '';
}
