<script setup>
import {
  createWorkedStep,
  MAXIMUM_WORKED_STEPS,
  MAXIMUM_STEP_PROMPT_LENGTH,
  MAXIMUM_STEP_SOLUTION_LENGTH,
  MAXIMUM_PRINCIPLE_PROMPT_LENGTH,
  workedStepErrors
} from '../problem/workedExamples';

const props = defineProps({
  modelValue: { type: Object, default: null },
  disabled: { type: Boolean, default: false },
  submitted: { type: Boolean, default: false }
});
const emit = defineEmits([ 'update:modelValue' ]);

function updateSteps( steps ) {
  emit( 'update:modelValue', { steps });
}

function updateStep( id, field, value ) {
  updateSteps( props.modelValue.steps.map( ( step ) => (
    step.id === id ? { ...step, [ field ]: value } : step
  ) ) );
}

function moveStep( index, offset ) {
  const steps = [ ...props.modelValue.steps ];
  const destination = index + offset;

  if ( destination < 0 || destination >= steps.length ) {
    return;
  }

  const [ step ] = steps.splice( index, 1 );

  steps.splice( destination, 0, step );
  updateSteps( steps );
}

function errorFor( step, field ) {
  return props.submitted ? workedStepErrors( step )[ field ] : false;
}

function lengthHint( value, maximum ) {
  return `${ Array.from( value.trim() ).length } / ${ maximum }`;
}
</script>

<template>
  <section class="worked-example-editor" data-twill-worked-example-editor>
    <div class="worked-example-editor__heading">
      <h3>Worked example</h3>
      <UButton
        v-if="modelValue"
        type="button"
        color="neutral"
        variant="link"
        size="sm"
        :disabled="disabled"
        @click="emit( 'update:modelValue', null )"
      >
        Remove example
      </UButton>
    </div>

    <UButton
      v-if="!modelValue"
      type="button"
      color="neutral"
      variant="subtle"
      size="sm"
      leading-icon="i-lucide-plus"
      :disabled="disabled"
      @click="updateSteps([ createWorkedStep() ])"
    >
      Add worked example
    </UButton>

    <template v-else>
      <p class="worked-example-editor__guidance">
        Write a checked solution for each step. Choose which solutions remain visible in completion practice.
      </p>

      <ol class="worked-example-editor__steps">
        <li v-for="( step, index ) in modelValue.steps" :key="step.id">
          <div class="worked-example-editor__heading">
            <h4>Step {{ index + 1 }}</h4>
            <div class="worked-example-editor__actions">
              <UButton
                type="button"
                icon="i-lucide-chevron-up"
                :aria-label="`Move worked step ${ index + 1 } up`"
                color="neutral"
                variant="ghost"
                size="xs"
                square
                :disabled="disabled || index === 0"
                @click="moveStep( index, -1 )"
              />
              <UButton
                type="button"
                icon="i-lucide-chevron-down"
                :aria-label="`Move worked step ${ index + 1 } down`"
                color="neutral"
                variant="ghost"
                size="xs"
                square
                :disabled="disabled || index === modelValue.steps.length - 1"
                @click="moveStep( index, 1 )"
              />
              <UButton
                type="button"
                icon="i-lucide-x"
                :aria-label="`Remove worked step ${ index + 1 }`"
                color="neutral"
                variant="ghost"
                size="xs"
                square
                :disabled="disabled || modelValue.steps.length === 1"
                @click="updateSteps( modelValue.steps.filter( ( item ) => item.id !== step.id ) )"
              />
            </div>
          </div>

          <UFormField
            :label="`Step ${ index + 1 } prompt`"
            :hint="lengthHint( step.prompt, MAXIMUM_STEP_PROMPT_LENGTH )"
            :error="errorFor( step, 'prompt' )"
            required
          >
            <UTextarea
              :model-value="step.prompt"
              :aria-label="`Step ${ index + 1 } prompt`"
              :rows="1"
              autoresize
              :maxrows="4"
              :disabled="disabled"
              class="worked-example-editor__input"
              @update:model-value="updateStep( step.id, 'prompt', $event )"
            />
          </UFormField>

          <UFormField
            :label="`Step ${ index + 1 } solution`"
            :hint="lengthHint( step.solution, MAXIMUM_STEP_SOLUTION_LENGTH )"
            :error="errorFor( step, 'solution' )"
            required
          >
            <UTextarea
              :model-value="step.solution"
              :aria-label="`Step ${ index + 1 } solution`"
              :rows="2"
              autoresize
              :maxrows="8"
              :disabled="disabled"
              class="worked-example-editor__input"
              @update:model-value="updateStep( step.id, 'solution', $event )"
            />
          </UFormField>

          <UFormField
            label="Principle question (optional)"
            :hint="lengthHint( step.principlePrompt, MAXIMUM_PRINCIPLE_PROMPT_LENGTH )"
            :error="errorFor( step, 'principlePrompt' )"
          >
            <UTextarea
              :model-value="step.principlePrompt"
              :aria-label="`Step ${ index + 1 } principle question`"
              :rows="1"
              autoresize
              :maxrows="4"
              :disabled="disabled"
              class="worked-example-editor__input"
              @update:model-value="updateStep( step.id, 'principlePrompt', $event )"
            />
          </UFormField>

          <UCheckbox
            :model-value="step.showInCompletion"
            :label="`Show step ${ index + 1 } solution in completion practice`"
            :disabled="disabled"
            @update:model-value="updateStep( step.id, 'showInCompletion', $event === true )"
          />
        </li>
      </ol>

      <UButton
        type="button"
        color="neutral"
        variant="subtle"
        size="sm"
        leading-icon="i-lucide-plus"
        :disabled="disabled || modelValue.steps.length >= MAXIMUM_WORKED_STEPS"
        @click="updateSteps([ ...modelValue.steps, createWorkedStep() ])"
      >
        Add worked step
      </UButton>
    </template>
  </section>
</template>

<style scoped>
.worked-example-editor,
.worked-example-editor__steps,
.worked-example-editor__steps > li {
  display: grid;
  min-width: 0;
  gap: 0.75rem;
}

.worked-example-editor {
  padding-top: 1rem;
  border-top: 1px solid var(--ui-border);
}

.worked-example-editor__heading,
.worked-example-editor__actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.4rem;
}

.worked-example-editor h3,
.worked-example-editor h4,
.worked-example-editor__guidance {
  margin: 0;
}

.worked-example-editor__guidance {
  color: var(--ui-text-muted);
  font-size: 0.82rem;
}

.worked-example-editor__steps {
  padding: 0;
  margin: 0;
  list-style: none;
}

.worked-example-editor__steps > li + li {
  padding-top: 0.75rem;
  border-top: 1px solid var(--ui-border);
}

.worked-example-editor__input {
  width: 100%;
}

.worked-example-editor > button {
  justify-self: start;
}
</style>
