<script setup>
import ProblemErrorDiagnosis from './ProblemErrorDiagnosis.vue';

defineProps({
  example: { type: Object, required: true },
  state: { type: Object, required: true },
  revealed: { type: Boolean, default: false },
  disabled: { type: Boolean, default: false }
});

const emit = defineEmits([ 'mode', 'toggle', 'response', 'principle', 'diagnosis', 'correction' ]);
const modes = [
  { id: 'workpad', label: 'Workpad' },
  { id: 'completion', label: 'Complete steps' },
  { id: 'example', label: 'Worked example' }
];
</script>

<template>
  <section class="study-problem-steps" data-twill-guided-problem aria-label="Problem practice">
    <div class="study-problem-steps__modes" role="group" aria-label="Problem support">
      <UButton
        v-for="mode in modes"
        :key="mode.id"
        type="button"
        color="neutral"
        :variant="state.mode === mode.id ? 'subtle' : 'ghost'"
        size="sm"
        :aria-pressed="state.mode === mode.id"
        :disabled="disabled"
        @click="emit( 'mode', mode.id )"
      >
        {{ mode.label }}
      </UButton>
      <UButton
        v-if="example.errorDiagnosis"
        type="button"
        color="neutral"
        :variant="state.mode === 'diagnosis' ? 'subtle' : 'ghost'"
        size="sm"
        :aria-pressed="state.mode === 'diagnosis'"
        :disabled="disabled"
        @click="emit( 'mode', 'diagnosis' )"
      >
        Find the error
      </UButton>
    </div>

    <ProblemErrorDiagnosis
      v-if="state.mode === 'diagnosis' && example.errorDiagnosis"
      :diagnosis="example.errorDiagnosis"
      :responses="state.diagnosis"
      :revealed="state.correctionShown"
      :checked="revealed"
      :disabled="disabled"
      @response="( field, value ) => emit( 'diagnosis', field, value )"
      @toggle="emit( 'correction' )"
    />
    <ol v-else-if="state.mode !== 'workpad'" class="study-problem-steps__list">
      <li v-for="( step, index ) in example.steps" :key="step.id" :data-twill-step="step.id">
        <h3>{{ index + 1 }}. {{ step.prompt }}</h3>

        <p
          v-if="state.mode === 'example' || revealed || step.showInCompletion || state.revealed.includes( step.id )"
          :id="`problem-step-solution-${ step.id }`"
          class="study-problem-steps__solution"
        >
          {{ step.solution }}
        </p>

        <template v-if="state.mode === 'completion'">
          <UFormField v-if="!step.showInCompletion" :label="`Your step ${ index + 1 }`">
            <UTextarea
              :model-value="state.responses[ step.id ] ?? ''"
              :aria-label="`Your step ${ index + 1 }`"
              :rows="2"
              autoresize
              :maxrows="8"
              :readonly="revealed"
              :disabled="disabled"
              class="study-problem-steps__input"
              @update:model-value="emit( 'response', step.id, $event )"
            />
          </UFormField>

          <UButton
            v-if="!step.showInCompletion && !revealed"
            type="button"
            color="neutral"
            variant="link"
            size="sm"
            :aria-label="`${ state.revealed.includes( step.id ) ? 'Hide' : 'Show' } step ${ index + 1 } solution`"
            :aria-expanded="state.revealed.includes( step.id )"
            :aria-controls="`problem-step-solution-${ step.id }`"
            :disabled="disabled"
            @click="emit( 'toggle', step.id )"
          >
            {{ state.revealed.includes( step.id ) ? 'Hide solution' : 'Show solution' }}
          </UButton>

          <UFormField v-if="step.principlePrompt" :label="step.principlePrompt" hint="Optional">
            <UTextarea
              :model-value="state.principles[ step.id ] ?? ''"
              :rows="1"
              autoresize
              :maxrows="5"
              :readonly="revealed"
              :disabled="disabled"
              class="study-problem-steps__input"
              @update:model-value="emit( 'principle', step.id, $event )"
            />
          </UFormField>
        </template>
        <p v-else-if="step.principlePrompt" class="study-problem-steps__principle">{{ step.principlePrompt }}</p>
      </li>
    </ol>
  </section>
</template>

<style scoped>
.study-problem-steps,
.study-problem-steps__list,
.study-problem-steps__list > li {
  display: grid;
  min-width: 0;
  gap: 0.65rem;
}

.study-problem-steps__modes {
  display: flex;
  flex-wrap: wrap;
  gap: 0.2rem;
}

.study-problem-steps__list {
  padding: 0;
  margin: 0;
  list-style: none;
}

.study-problem-steps__list > li {
  padding-left: 0.75rem;
  border-left: 2px solid var(--ui-border);
}

.study-problem-steps h3,
.study-problem-steps p {
  margin: 0;
  font-size: 0.9rem;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.study-problem-steps h3 {
  font-weight: 600;
}

.study-problem-steps__principle {
  color: var(--ui-text-muted);
}

.study-problem-steps__input {
  width: 100%;
}

.study-problem-steps__list > li > button {
  justify-self: start;
}
</style>
