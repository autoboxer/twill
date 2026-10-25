import { createUuid } from '../lib/identifiers';

export const MAXIMUM_WORKED_STEPS = 12;
export const MAXIMUM_STEP_PROMPT_LENGTH = 280;
export const MAXIMUM_STEP_SOLUTION_LENGTH = 2000;
export const MAXIMUM_PRINCIPLE_PROMPT_LENGTH = 500;

export const diagnosisFields = [
  {
    id: 'attempt', label: 'Example with an error', maximum: MAXIMUM_STEP_SOLUTION_LENGTH,
    requiredMessage: 'Write an example containing an error.'
  },
  {
    id: 'location', label: 'Error location', maximum: MAXIMUM_STEP_PROMPT_LENGTH,
    requiredMessage: 'Identify the error.'
  },
  {
    id: 'explanation', label: 'Why it is wrong', maximum: MAXIMUM_STEP_SOLUTION_LENGTH,
    requiredMessage: 'Explain why the step is wrong.'
  },
  {
    id: 'correction', label: 'Corrected work', maximum: MAXIMUM_STEP_SOLUTION_LENGTH,
    requiredMessage: 'Provide the corrected work.'
  }
];

export function createErrorDiagnosis() {
  return Object.fromEntries( diagnosisFields.map( ({ id }) => [ id, '' ]) );
}

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

  const cloned = {
    steps: example.steps.map( ( step ) => ({
      id: typeof step?.id === 'string' ? step.id : createUuid(),
      prompt: textValue( step?.prompt ),
      solution: textValue( step?.solution ),
      principlePrompt: textValue( step?.principlePrompt ),
      showInCompletion: step?.showInCompletion === true
    }) )
  };

  if ( example.errorDiagnosis ) {
    cloned.errorDiagnosis = Object.fromEntries( diagnosisFields.map( ({ id }) => (
      [ id, textValue( example.errorDiagnosis[ id ]) ]
    ) ) );
  }

  return cloned;
}

export function normalizeWorkedExample( example ) {
  const cloned = cloneWorkedExample( example );

  if ( !cloned ) {
    return null;
  }

  const normalized = {
    steps: cloned.steps.map( ( step ) => ({
      ...step,
      prompt: step.prompt.trim(),
      solution: step.solution.trim(),
      principlePrompt: step.principlePrompt.trim()
    }) )
  };

  if ( cloned.errorDiagnosis ) {
    normalized.errorDiagnosis = Object.fromEntries( diagnosisFields.map( ({ id }) => (
      [ id, cloned.errorDiagnosis[ id ].trim() ]
    ) ) );
  }

  return normalized;
}

export function errorDiagnosisErrors( diagnosis ) {
  return Object.fromEntries( diagnosisFields.map( ({ id, requiredMessage, maximum }) => (
    [ id, fieldError( diagnosis[ id ], requiredMessage, maximum ) ]
  ) ) );
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
    .concat( example.errorDiagnosis ? Object.values( errorDiagnosisErrors( example.errorDiagnosis ) ) : [])
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
