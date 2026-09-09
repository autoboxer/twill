<script setup>
import { m } from 'motion-v';
import { computed, onBeforeUnmount, reactive, ref } from 'vue';

import { COMMAND_IDS } from '../../commands/registry';
import { useCommandHandler } from '../../composables/useCommands';
import { conceptLibraryErrorMessage } from '../../composables/useConceptLibrary';
import { useSchedulingSettings } from '../../composables/useSchedulingSettings';

const props = defineProps({
  initialSettings: {
    type: Object,
    required: true
  },
  panelTransition: {
    type: Object,
    required: true
  }
});

const DEFAULT_DESIRED_RETENTION_PERCENT = 90;
const DEFAULT_MAXIMUM_INTERVAL_DAYS = 36_500;
const MINIMUM_DESIRED_RETENTION_PERCENT = 80;
const MAXIMUM_DESIRED_RETENTION_PERCENT = 97;
const MINIMUM_INTERVAL_DAYS = 1;
const MAXIMUM_INTERVAL_DAYS = 36_500;

const { updateSchedulingSettings } = useSchedulingSettings();

const schedulingForm = reactive({
  desiredRetentionPercent: props.initialSettings.desiredRetention * 100,
  maximumIntervalDays: props.initialSettings.maximumIntervalDays
});

const savedSchedulingSettings = ref( props.initialSettings );
const schedulingSaveAttempted = ref( false );
const schedulingSaveError = ref( '' );
const schedulingSavePending = ref( false );
const schedulingSaveStatus = ref( '' );
let viewActive = true;

const desiredRetention = computed( () => {
  return Number( schedulingForm.desiredRetentionPercent ) / 100;
});

const desiredRetentionError = computed( () => {
  if ( !schedulingSaveAttempted.value ) {
    return '';
  }

  const value = Number( schedulingForm.desiredRetentionPercent );

  if (
    !Number.isFinite( value )
    || value < MINIMUM_DESIRED_RETENTION_PERCENT
    || value > MAXIMUM_DESIRED_RETENTION_PERCENT
  ) {
    return 'Enter a target from 80% to 97%.';
  }

  return '';
});

const maximumIntervalError = computed( () => {
  if ( !schedulingSaveAttempted.value ) {
    return '';
  }

  const value = Number( schedulingForm.maximumIntervalDays );

  if (
    !Number.isInteger( value )
    || value < MINIMUM_INTERVAL_DAYS
    || value > MAXIMUM_INTERVAL_DAYS
  ) {
    return 'Enter a whole number from 1 to 36,500 days.';
  }

  return '';
});

const schedulingFormValid = computed( () => {
  const retention = Number( schedulingForm.desiredRetentionPercent );
  const maximumInterval = Number( schedulingForm.maximumIntervalDays );

  return Number.isFinite( retention )
    && retention >= MINIMUM_DESIRED_RETENTION_PERCENT
    && retention <= MAXIMUM_DESIRED_RETENTION_PERCENT
    && Number.isInteger( maximumInterval )
    && maximumInterval >= MINIMUM_INTERVAL_DAYS
    && maximumInterval <= MAXIMUM_INTERVAL_DAYS;
});

const schedulingHasChanges = computed( () => {
  if ( !savedSchedulingSettings.value || !schedulingFormValid.value ) {
    return false;
  }

  return desiredRetention.value !== savedSchedulingSettings.value.desiredRetention
    || Number( schedulingForm.maximumIntervalDays )
      !== savedSchedulingSettings.value.maximumIntervalDays;
});

const schedulingDefaultsActive = computed( () => {
  return Number( schedulingForm.desiredRetentionPercent )
      === DEFAULT_DESIRED_RETENTION_PERCENT
    && Number( schedulingForm.maximumIntervalDays )
      === DEFAULT_MAXIMUM_INTERVAL_DAYS
    && savedSchedulingSettings.value?.desiredRetention
      === DEFAULT_DESIRED_RETENTION_PERCENT / 100
    && savedSchedulingSettings.value?.maximumIntervalDays
      === DEFAULT_MAXIMUM_INTERVAL_DAYS;
});

const maximumIntervalSummary = computed( () => {
  const days = Number( schedulingForm.maximumIntervalDays );

  if ( !Number.isFinite( days ) || days < 1 ) {
    return '';
  }

  if ( days >= 365 ) {
    const years = days / 365;

    return formatApproximateDuration(
      years,
      'year',
      years < 10 ? 1 : 0
    );
  }

  if ( days >= 30 ) {
    const months = days / 30;

    return formatApproximateDuration( months, 'month', 1 );
  }

  return `${ days } ${ days === 1 ? 'day' : 'days' }`;
});

const schedulingSaveCommand = useCommandHandler( COMMAND_IDS.schedulingSave, {
  enabled: computed( () => (
    !schedulingSavePending.value
    && !( schedulingFormValid.value && !schedulingHasChanges.value )
  ) ),
  execute: saveSchedulingSettings
});

onBeforeUnmount( () => {
  viewActive = false;
});

function applySchedulingSettings( settings ) {
  savedSchedulingSettings.value = settings;
  schedulingForm.desiredRetentionPercent = settings.desiredRetention * 100;
  schedulingForm.maximumIntervalDays = settings.maximumIntervalDays;
  schedulingSaveAttempted.value = false;
}

function clearSchedulingSaveFeedback() {
  schedulingSaveError.value = '';
  schedulingSaveStatus.value = '';
}

function formatApproximateDuration( value, unit, maximumFractionDigits ) {
  const duration = new Intl.NumberFormat( undefined, {
    style: 'unit',
    unit,
    unitDisplay: 'long',
    maximumFractionDigits
  }).format( value );

  return `About ${ duration }`;
}

async function saveSchedulingSettings() {
  schedulingSaveAttempted.value = true;
  clearSchedulingSaveFeedback();

  if ( !schedulingFormValid.value || schedulingSavePending.value ) {
    return;
  }

  await persistSchedulingSettings(
    desiredRetention.value,
    Number( schedulingForm.maximumIntervalDays ),
    'Scheduling settings saved.'
  );
}

async function restoreSchedulingDefaults() {
  if ( schedulingSavePending.value || schedulingDefaultsActive.value ) {
    return;
  }

  schedulingSaveAttempted.value = false;
  clearSchedulingSaveFeedback();

  await persistSchedulingSettings(
    DEFAULT_DESIRED_RETENTION_PERCENT / 100,
    DEFAULT_MAXIMUM_INTERVAL_DAYS,
    'Scheduling defaults restored.'
  );
}

async function persistSchedulingSettings(
  desiredRetentionValue,
  maximumIntervalDays,
  successMessage
) {
  schedulingSavePending.value = true;

  try {
    const settings = await updateSchedulingSettings(
      desiredRetentionValue,
      maximumIntervalDays
    );

    if ( !viewActive ) {
      return;
    }

    applySchedulingSettings( settings );
    schedulingSaveStatus.value = successMessage;
  } catch ( cause ) {
    if ( viewActive ) {
      schedulingSaveError.value = conceptLibraryErrorMessage( cause );
    }
  } finally {
    if ( viewActive ) {
      schedulingSavePending.value = false;
    }
  }
}
</script>

<template>
  <m.form
    id="settings-scheduling"
    class="settings-panel"
    data-twill-settings-section="scheduling"
    novalidate
    :initial="{ opacity: 0, y: 10 }"
    :animate="{ opacity: 1, y: 0 }"
    :transition="{ ...panelTransition, delay: 0.16 }"
    @submit.prevent="saveSchedulingSettings"
  >
    <header class="settings-panel__header">
      <span
        class="settings-panel__icon"
        aria-hidden="true"
      >
        <UIcon name="i-lucide-calendar-sync" />
      </span>

      <div>
        <p class="settings-panel__version">
          FSRS {{ savedSchedulingSettings.algorithmVersion }}
        </p>
        <h2>Scheduling</h2>
        <p>Adjust long-term retention and the longest review interval.</p>
      </div>
    </header>

    <UAlert
      description="Existing due dates stay unchanged. New values apply when each card is reviewed."
      icon="i-lucide-info"
      color="neutral"
      variant="subtle"
    />

    <div class="settings-fields">
      <UFormField
        label="Target retention"
        description="Higher values shorten intervals and increase the number of reviews."
        :error="desiredRetentionError"
        required
      >
        <div class="settings-input-row">
          <UInput
            v-model="schedulingForm.desiredRetentionPercent"
            type="number"
            :min="MINIMUM_DESIRED_RETENTION_PERCENT"
            :max="MAXIMUM_DESIRED_RETENTION_PERCENT"
            step="1"
            inputmode="decimal"
            class="settings-number-input"
            size="xl"
            @input="clearSchedulingSaveFeedback"
          />
          <span>%</span>
        </div>
      </UFormField>

      <UFormField
        label="Maximum interval"
        description="Shorter limits prevent distant due dates but can substantially increase reviews."
        :hint="maximumIntervalSummary"
        :error="maximumIntervalError"
        required
      >
        <div class="settings-input-row">
          <UInput
            v-model="schedulingForm.maximumIntervalDays"
            type="number"
            :min="MINIMUM_INTERVAL_DAYS"
            :max="MAXIMUM_INTERVAL_DAYS"
            step="1"
            inputmode="numeric"
            class="settings-number-input"
            size="xl"
            @input="clearSchedulingSaveFeedback"
          />
          <span>days</span>
        </div>
      </UFormField>
    </div>

    <UAlert
      v-if="schedulingSaveError"
      :description="schedulingSaveError"
      icon="i-lucide-circle-alert"
      color="error"
      variant="subtle"
    />

    <footer class="settings-actions">
      <p
        class="settings-save-status"
        aria-live="polite"
      >
        {{ schedulingSaveStatus }}
      </p>

      <UButton
        type="button"
        color="neutral"
        variant="link"
        :disabled="schedulingDefaultsActive || schedulingSavePending"
        @click="restoreSchedulingDefaults"
      >
        Restore defaults
      </UButton>

      <UButton
        type="submit"
        leading-icon="i-lucide-check"
        :disabled="schedulingFormValid && !schedulingHasChanges"
        :loading="schedulingSavePending"
        :aria-keyshortcuts="schedulingSaveCommand.ariaKeyshortcuts"
        :title="schedulingSaveCommand.tooltip"
        size="lg"
      >
        Save settings
      </UButton>
    </footer>
  </m.form>
</template>
