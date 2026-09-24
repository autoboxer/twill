<script setup>
import { m } from 'motion-v';
import { onBeforeUnmount, ref } from 'vue';

import { conceptLibraryErrorMessage } from '../../composables/useConceptLibrary';
import { useDevicePreferences } from '../../composables/useDevicePreferences';
import { useActionNotifications } from '../../composables/useActionNotifications';

const props = defineProps({
  initialPreferences: {
    type: Object,
    required: true
  },
  panelTransition: {
    type: Object,
    required: true
  }
});

const DEFAULT_GRADING_MODE = 'simple';

const DEFAULT_MIXED_PRACTICE_ENABLED = false;

const DEFAULT_PRETESTING_ENABLED = false;

const gradingModeItems = [
  { label: 'Simple', value: 'simple' },
  { label: 'Advanced', value: 'advanced' }
];

const {
  setGradingMode,
  setMixedPracticeEnabled,
  setPretestingEnabled
} = useDevicePreferences();
const { notifySuccess } = useActionNotifications();

const gradingMode = ref( props.initialPreferences.gradingMode );
const savedGradingMode = ref( props.initialPreferences.gradingMode );
const gradingModeError = ref( '' );
const gradingModePending = ref( false );

const mixedPracticeEnabled = ref( Boolean( props.initialPreferences.mixedPracticeEnabled ) );
const savedMixedPracticeEnabled = ref( mixedPracticeEnabled.value );
const mixedPracticeError = ref( '' );
const mixedPracticePending = ref( false );

const pretestingEnabled = ref( Boolean( props.initialPreferences.pretestingEnabled ) );
const savedPretestingEnabled = ref( pretestingEnabled.value );
const pretestingError = ref( '' );
const pretestingPending = ref( false );

let viewActive = true;

onBeforeUnmount( () => {
  viewActive = false;
});

async function updateGradingMode(
  nextMode,
  successMessage = 'Grading mode saved.'
) {
  if (
    gradingModePending.value
    || !gradingModeItems.some( ( item ) => item.value === nextMode )
    || nextMode === savedGradingMode.value
  ) {
    return;
  }

  const previousMode = savedGradingMode.value;

  gradingMode.value = nextMode;
  gradingModeError.value = '';
  gradingModePending.value = true;

  try {
    const preferences = await setGradingMode( nextMode );

    if ( !viewActive ) {
      return;
    }

    gradingMode.value = preferences.gradingMode;
    savedGradingMode.value = preferences.gradingMode;
    notifySuccess( successMessage );
  } catch ( cause ) {
    if ( viewActive ) {
      gradingMode.value = previousMode;
      gradingModeError.value = conceptLibraryErrorMessage( cause );
    }
  } finally {
    if ( viewActive ) {
      gradingModePending.value = false;
    }
  }
}

async function updatePretesting(
  enabled,
  successMessage = 'Pretesting preference saved.'
) {
  if (
    pretestingPending.value
    || typeof enabled !== 'boolean'
    || enabled === savedPretestingEnabled.value
  ) {
    return;
  }

  const previousValue = savedPretestingEnabled.value;

  pretestingEnabled.value = enabled;
  pretestingError.value = '';
  pretestingPending.value = true;

  try {
    const preferences = await setPretestingEnabled( enabled );

    if ( !viewActive ) {
      return;
    }

    pretestingEnabled.value = preferences.pretestingEnabled;
    savedPretestingEnabled.value = preferences.pretestingEnabled;
    notifySuccess( successMessage );
  } catch ( cause ) {
    if ( viewActive ) {
      pretestingEnabled.value = previousValue;
      pretestingError.value = conceptLibraryErrorMessage( cause );
    }
  } finally {
    if ( viewActive ) {
      pretestingPending.value = false;
    }
  }
}

async function updateMixedPractice(
  enabled,
  successMessage = 'Mixed practice preference saved.'
) {
  if (
    mixedPracticePending.value
    || typeof enabled !== 'boolean'
    || enabled === savedMixedPracticeEnabled.value
  ) {
    return;
  }

  const previousValue = savedMixedPracticeEnabled.value;

  mixedPracticeEnabled.value = enabled;
  mixedPracticeError.value = '';
  mixedPracticePending.value = true;

  try {
    const preferences = await setMixedPracticeEnabled( enabled );

    if ( !viewActive ) {
      return;
    }

    mixedPracticeEnabled.value = preferences.mixedPracticeEnabled;
    savedMixedPracticeEnabled.value = preferences.mixedPracticeEnabled;
    notifySuccess( successMessage );
  } catch ( cause ) {
    if ( viewActive ) {
      mixedPracticeEnabled.value = previousValue;
      mixedPracticeError.value = conceptLibraryErrorMessage( cause );
    }
  } finally {
    if ( viewActive ) {
      mixedPracticePending.value = false;
    }
  }
}

async function restoreStudyDefaults() {
  if (
    gradingModePending.value
    || mixedPracticePending.value
    || pretestingPending.value
  ) {
    return;
  }

  if ( savedMixedPracticeEnabled.value !== DEFAULT_MIXED_PRACTICE_ENABLED ) {
    await updateMixedPractice( DEFAULT_MIXED_PRACTICE_ENABLED, '' );
  }

  if ( savedPretestingEnabled.value !== DEFAULT_PRETESTING_ENABLED ) {
    await updatePretesting( DEFAULT_PRETESTING_ENABLED, '' );
  }

  if ( savedGradingMode.value !== DEFAULT_GRADING_MODE ) {
    await updateGradingMode( DEFAULT_GRADING_MODE, '' );
  }

  if (
    viewActive
    && !gradingModeError.value
    && !mixedPracticeError.value
    && !pretestingError.value
  ) {
    notifySuccess( 'Study defaults restored.' );
  }
}
</script>

<template>
  <m.section
    id="settings-study"
    class="settings-panel"
    data-twill-settings-section="study"
    :initial="{ opacity: 0, y: 10 }"
    :animate="{ opacity: 1, y: 0 }"
    :transition="{ ...panelTransition, delay: 0.12 }"
  >
    <header class="settings-panel__header">
      <span
        class="settings-panel__icon"
        aria-hidden="true"
      >
        <UIcon name="i-lucide-graduation-cap" />
      </span>

      <div>
        <h2>Study</h2>
      </div>
    </header>

    <div class="settings-preference-row">
      <div>
        <label for="settings-pretesting">Optional pretesting</label>
        <p id="settings-pretesting-description">
          Before the first review of a new concept, attempt one prompt and
          then see its answer. Attempts stay separate from review grading
          and FSRS scheduling.
        </p>
      </div>

      <USwitch
        id="settings-pretesting"
        :model-value="pretestingEnabled"
        :loading="pretestingPending"
        :disabled="pretestingPending"
        aria-describedby="settings-pretesting-description"
        @update:model-value="updatePretesting"
      />
    </div>

    <UAlert
      v-if="pretestingError"
      :description="pretestingError"
      icon="i-lucide-circle-alert"
      color="error"
      variant="subtle"
    />

    <div class="settings-preference-row">
      <div>
        <label for="settings-mixed-practice">Mixed practice</label>
        <p id="settings-mixed-practice-description">
          Reorder small groups of due cards to separate forms of one
          concept and vary retrieval forms. Shared tags can place related
          concepts together for contrast. Scheduling does not change.
        </p>
      </div>

      <USwitch
        id="settings-mixed-practice"
        :model-value="mixedPracticeEnabled"
        :loading="mixedPracticePending"
        :disabled="mixedPracticePending"
        aria-describedby="settings-mixed-practice-description"
        @update:model-value="updateMixedPractice"
      />
    </div>

    <UAlert
      v-if="mixedPracticeError"
      :description="mixedPracticeError"
      icon="i-lucide-circle-alert"
      color="error"
      variant="subtle"
    />

    <div class="settings-preference-row">
      <div>
        <label for="settings-grading-mode">Grading mode</label>
        <p>Simple uses Forgot and Remembered. Advanced adds Hard and Easy.</p>
      </div>

      <USelect
        id="settings-grading-mode"
        :model-value="gradingMode"
        :items="gradingModeItems"
        :loading="gradingModePending"
        :disabled="gradingModePending"
        value-key="value"
        leading-icon="i-lucide-list-checks"
        class="settings-select"
        @update:model-value="updateGradingMode"
      />
    </div>

    <UAlert
      v-if="gradingModeError"
      :description="gradingModeError"
      icon="i-lucide-circle-alert"
      color="error"
      variant="subtle"
    />

    <footer class="settings-section-actions">
      <UButton
        type="button"
        color="neutral"
        variant="link"
        :disabled="(savedGradingMode === DEFAULT_GRADING_MODE
          && savedMixedPracticeEnabled === DEFAULT_MIXED_PRACTICE_ENABLED
          && savedPretestingEnabled === DEFAULT_PRETESTING_ENABLED)
          || gradingModePending
          || mixedPracticePending
          || pretestingPending"
        @click="restoreStudyDefaults"
      >
        Restore default
      </UButton>
    </footer>
  </m.section>
</template>
