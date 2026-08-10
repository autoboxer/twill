<script setup>
import { m } from 'motion-v';
import { computed, onBeforeUnmount, onMounted, reactive, ref } from 'vue';

import ContentState from '../components/ContentState.vue';
import PageHeader from '../components/PageHeader.vue';
import { conceptLibraryErrorMessage } from '../composables/useConceptLibrary';
import { useSchedulingSettings } from '../composables/useSchedulingSettings';

const DEFAULT_DESIRED_RETENTION_PERCENT = 90;
const DEFAULT_MAXIMUM_INTERVAL_DAYS = 36_500;
const MINIMUM_DESIRED_RETENTION_PERCENT = 80;
const MAXIMUM_DESIRED_RETENTION_PERCENT = 97;
const MINIMUM_INTERVAL_DAYS = 1;
const MAXIMUM_INTERVAL_DAYS = 36_500;

const {
  getSchedulingSettings,
  updateSchedulingSettings
} = useSchedulingSettings();

const form = reactive({
  desiredRetentionPercent: DEFAULT_DESIRED_RETENTION_PERCENT,
  maximumIntervalDays: DEFAULT_MAXIMUM_INTERVAL_DAYS
});

const initialLoading = ref( true );
const loadError = ref( '' );
const saveAttempted = ref( false );
const saveError = ref( '' );
const savePending = ref( false );
const saveStatus = ref( '' );
const savedSettings = ref( null );
let loadRequestSequence = 0;
let viewActive = true;

const panelTransition = {
  duration: 0.24,
  ease: [ 0.22, 1, 0.36, 1 ]
};

const desiredRetention = computed( () => {
  return Number( form.desiredRetentionPercent ) / 100;
});

const desiredRetentionError = computed( () => {
  if ( !saveAttempted.value ) {
    return '';
  }

  const value = Number( form.desiredRetentionPercent );

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
  if ( !saveAttempted.value ) {
    return '';
  }

  const value = Number( form.maximumIntervalDays );

  if (
    !Number.isInteger( value )
    || value < MINIMUM_INTERVAL_DAYS
    || value > MAXIMUM_INTERVAL_DAYS
  ) {
    return 'Enter a whole number from 1 to 36,500 days.';
  }

  return '';
});

const formValid = computed( () => {
  const retention = Number( form.desiredRetentionPercent );
  const maximumInterval = Number( form.maximumIntervalDays );

  return Number.isFinite( retention )
    && retention >= MINIMUM_DESIRED_RETENTION_PERCENT
    && retention <= MAXIMUM_DESIRED_RETENTION_PERCENT
    && Number.isInteger( maximumInterval )
    && maximumInterval >= MINIMUM_INTERVAL_DAYS
    && maximumInterval <= MAXIMUM_INTERVAL_DAYS;
});

const hasChanges = computed( () => {
  if ( !savedSettings.value || !formValid.value ) {
    return false;
  }

  return desiredRetention.value !== savedSettings.value.desiredRetention
    || Number( form.maximumIntervalDays ) !== savedSettings.value.maximumIntervalDays;
});

const defaultsActive = computed( () => {
  return Number( form.desiredRetentionPercent )
      === DEFAULT_DESIRED_RETENTION_PERCENT
    && Number( form.maximumIntervalDays ) === DEFAULT_MAXIMUM_INTERVAL_DAYS
    && savedSettings.value?.desiredRetention
      === DEFAULT_DESIRED_RETENTION_PERCENT / 100
    && savedSettings.value?.maximumIntervalDays
      === DEFAULT_MAXIMUM_INTERVAL_DAYS;
});

const maximumIntervalSummary = computed( () => {
  const days = Number( form.maximumIntervalDays );

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

onMounted( loadSettings );

onBeforeUnmount( () => {
  viewActive = false;
  loadRequestSequence += 1;
});

async function loadSettings() {
  const request = ++loadRequestSequence;

  initialLoading.value = true;
  loadError.value = '';
  saveError.value = '';
  saveStatus.value = '';

  try {
    const settings = await getSchedulingSettings();

    if ( request !== loadRequestSequence ) {
      return;
    }

    applySettings( settings );
  } catch ( cause ) {
    if ( request === loadRequestSequence ) {
      loadError.value = conceptLibraryErrorMessage( cause );
    }
  } finally {
    if ( request === loadRequestSequence ) {
      initialLoading.value = false;
    }
  }
}

function applySettings( settings ) {
  savedSettings.value = settings;
  form.desiredRetentionPercent = settings.desiredRetention * 100;
  form.maximumIntervalDays = settings.maximumIntervalDays;
  saveAttempted.value = false;
}

function clearSaveFeedback() {
  saveError.value = '';
  saveStatus.value = '';
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

async function saveSettings() {
  saveAttempted.value = true;
  saveError.value = '';
  saveStatus.value = '';

  if ( !formValid.value || savePending.value ) {
    return;
  }

  await persistSettings(
    desiredRetention.value,
    Number( form.maximumIntervalDays ),
    'Scheduling settings saved.'
  );
}

async function restoreDefaults() {
  if ( savePending.value || defaultsActive.value ) {
    return;
  }

  saveAttempted.value = false;
  saveError.value = '';
  saveStatus.value = '';

  await persistSettings(
    DEFAULT_DESIRED_RETENTION_PERCENT / 100,
    DEFAULT_MAXIMUM_INTERVAL_DAYS,
    'Scheduling defaults restored.'
  );
}

async function persistSettings(
  desiredRetentionValue,
  maximumIntervalDays,
  successMessage
) {
  savePending.value = true;

  try {
    const settings = await updateSchedulingSettings(
      desiredRetentionValue,
      maximumIntervalDays
    );

    if ( !viewActive ) {
      return;
    }

    applySettings( settings );
    saveStatus.value = successMessage;
  } catch ( cause ) {
    if ( viewActive ) {
      saveError.value = conceptLibraryErrorMessage( cause );
    }
  } finally {
    if ( viewActive ) {
      savePending.value = false;
    }
  }
}
</script>

<template>
  <div class="page settings-page">
    <PageHeader title="Settings" />

    <ContentState
      v-if="initialLoading"
      kind="loading"
      title="Loading settings"
    />

    <ContentState
      v-else-if="loadError"
      kind="error"
      title="Settings could not be loaded"
      :description="loadError"
    >
      <template #actions>
        <UButton
          leading-icon="i-lucide-refresh-cw"
          @click="loadSettings"
        >
          Retry
        </UButton>
      </template>
    </ContentState>

    <div
      v-else
      class="settings-layout"
    >
      <m.form
        class="settings-panel"
        novalidate
        :initial="{ opacity: 0, y: 10 }"
        :animate="{ opacity: 1, y: 0 }"
        :transition="panelTransition"
        @submit.prevent="saveSettings"
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
              FSRS {{ savedSettings.algorithmVersion }}
            </p>
            <h2>Scheduling</h2>
            <p>Adjust long-term retention and the longest review interval.</p>
          </div>
        </header>

        <UAlert
          description="Existing due dates stay unchanged. New values apply when each card is reviewed."
          icon="i-lucide-info"
          color="neutral"
          variant="soft"
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
                v-model="form.desiredRetentionPercent"
                type="number"
                :min="MINIMUM_DESIRED_RETENTION_PERCENT"
                :max="MAXIMUM_DESIRED_RETENTION_PERCENT"
                step="1"
                inputmode="decimal"
                class="settings-number-input"
                size="xl"
                @input="clearSaveFeedback"
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
                v-model="form.maximumIntervalDays"
                type="number"
                :min="MINIMUM_INTERVAL_DAYS"
                :max="MAXIMUM_INTERVAL_DAYS"
                step="1"
                inputmode="numeric"
                class="settings-number-input"
                size="xl"
                @input="clearSaveFeedback"
              />
              <span>days</span>
            </div>
          </UFormField>
        </div>

        <UAlert
          v-if="saveError"
          :description="saveError"
          icon="i-lucide-circle-alert"
          color="error"
          variant="soft"
        />

        <footer class="settings-actions">
          <p
            class="settings-save-status"
            aria-live="polite"
          >
            {{ saveStatus }}
          </p>

          <UButton
            type="button"
            color="neutral"
            variant="ghost"
            :disabled="defaultsActive || savePending"
            @click="restoreDefaults"
          >
            Restore defaults
          </UButton>

          <UButton
            type="submit"
            leading-icon="i-lucide-check"
            :disabled="formValid && !hasChanges"
            :loading="savePending"
            size="lg"
          >
            Save settings
          </UButton>
        </footer>
      </m.form>
    </div>
  </div>
</template>
