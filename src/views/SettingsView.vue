<script setup>
import { m } from 'motion-v';
import { computed, onBeforeUnmount, onMounted, reactive, ref } from 'vue';

import ContentState from '../components/ContentState.vue';
import CssSnippetManager from '../components/CssSnippetManager.vue';
import PageHeader from '../components/PageHeader.vue';
import {
  DEFAULT_APPEARANCE,
  motionOptions,
  readingFontOptions,
  readingTextSizeOptions,
  themeOptions
} from '../appearance/options';
import { COMMAND_IDS } from '../commands/registry';
import { useAppearance } from '../composables/useAppearance';
import { useCommandHandler } from '../composables/useCommands';
import { conceptLibraryErrorMessage } from '../composables/useConceptLibrary';
import { useDevicePreferences } from '../composables/useDevicePreferences';
import { useSchedulingSettings } from '../composables/useSchedulingSettings';

const DEFAULT_DESIRED_RETENTION_PERCENT = 90;
const DEFAULT_GRADING_MODE = 'simple';
const DEFAULT_MAXIMUM_INTERVAL_DAYS = 36_500;
const DEFAULT_PRETESTING_ENABLED = false;
const DEFAULT_STARTUP_DESTINATION = 'study';
const MINIMUM_DESIRED_RETENTION_PERCENT = 80;
const MAXIMUM_DESIRED_RETENTION_PERCENT = 97;
const MINIMUM_INTERVAL_DAYS = 1;
const MAXIMUM_INTERVAL_DAYS = 36_500;

const gradingModeItems = [
  { label: 'Simple', value: 'simple' },
  { label: 'Advanced', value: 'advanced' }
];

const settingsSections = [
  { id: 'general', icon: 'i-lucide-settings-2', label: 'General' },
  { id: 'appearance', icon: 'i-lucide-palette', label: 'Appearance' },
  { id: 'snippets', icon: 'i-lucide-braces', label: 'Snippets' },
  { id: 'study', icon: 'i-lucide-graduation-cap', label: 'Study' },
  { id: 'scheduling', icon: 'i-lucide-calendar-sync', label: 'Scheduling' }
];

const startupDestinationItems = [
  { label: 'Study', value: 'study' },
  { label: 'Library', value: 'library' }
];

const themeGroups = [
  {
    id: 'dark',
    label: 'Dark',
    themes: themeOptions.filter( ( theme ) => theme.mode === 'dark' )
  },
  {
    id: 'light',
    label: 'Light',
    themes: themeOptions.filter( ( theme ) => theme.mode === 'light' )
  }
];

const {
  getDevicePreferences,
  setGradingMode,
  setPretestingEnabled,
  setStartupDestination
} = useDevicePreferences();
const {
  appearance,
  setAppearance
} = useAppearance();
const {
  getSchedulingSettings,
  updateSchedulingSettings
} = useSchedulingSettings();

const schedulingForm = reactive({
  desiredRetentionPercent: DEFAULT_DESIRED_RETENTION_PERCENT,
  maximumIntervalDays: DEFAULT_MAXIMUM_INTERVAL_DAYS
});

const gradingMode = ref( DEFAULT_GRADING_MODE );
const gradingModeError = ref( '' );
const gradingModePending = ref( false );
const gradingModeStatus = ref( '' );
const initialLoading = ref( true );
const loadError = ref( '' );
const pretestingEnabled = ref( DEFAULT_PRETESTING_ENABLED );
const pretestingError = ref( '' );
const pretestingPending = ref( false );
const pretestingStatus = ref( '' );
const appearanceError = ref( '' );
const appearancePendingCount = ref( 0 );
const appearanceStatus = ref( '' );
const savedGradingMode = ref( DEFAULT_GRADING_MODE );
const savedPretestingEnabled = ref( DEFAULT_PRETESTING_ENABLED );
const savedSchedulingSettings = ref( null );
const savedStartupDestination = ref( DEFAULT_STARTUP_DESTINATION );
const schedulingSaveAttempted = ref( false );
const schedulingSaveError = ref( '' );
const schedulingSavePending = ref( false );
const schedulingSaveStatus = ref( '' );
const startupDestination = ref( DEFAULT_STARTUP_DESTINATION );
const startupDestinationError = ref( '' );
const startupDestinationPending = ref( false );
const startupDestinationStatus = ref( '' );
let loadRequestSequence = 0;
let appearanceRequestSequence = 0;
let viewActive = true;

const panelTransition = {
  duration: 0.24,
  ease: [ 0.22, 1, 0.36, 1 ]
};

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

const appearanceDefaultsActive = computed( () => {
  return Object.entries( DEFAULT_APPEARANCE ).every( ([ key, value ]) => (
    appearance.value[ key ] === value
  ) );
});

const appearanceSaveStatus = computed( () => {
  return appearancePendingCount.value > 0
    ? 'Saving appearance…'
    : appearanceStatus.value;
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
    !initialLoading.value
    && !loadError.value
    && !schedulingSavePending.value
    && !( schedulingFormValid.value && !schedulingHasChanges.value )
  ) ),
  execute: saveSchedulingSettings
});

onMounted( loadSettings );

onBeforeUnmount( () => {
  viewActive = false;
  loadRequestSequence += 1;
  appearanceRequestSequence += 1;
});

async function loadSettings() {
  const request = ++loadRequestSequence;

  initialLoading.value = true;
  loadError.value = '';
  clearPreferenceFeedback();
  clearSchedulingSaveFeedback();

  try {
    const [ preferences, schedulingSettings ] = await Promise.all([
      getDevicePreferences(),
      getSchedulingSettings()
    ]);

    if ( request !== loadRequestSequence ) {
      return;
    }

    applyDevicePreferences( preferences );
    applySchedulingSettings( schedulingSettings );
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

function applyDevicePreferences( preferences ) {
  gradingMode.value = preferences.gradingMode;
  savedGradingMode.value = preferences.gradingMode;
  pretestingEnabled.value = Boolean( preferences.pretestingEnabled );
  savedPretestingEnabled.value = Boolean( preferences.pretestingEnabled );
  startupDestination.value = preferences.startupDestination;
  savedStartupDestination.value = preferences.startupDestination;
}

function applySchedulingSettings( settings ) {
  savedSchedulingSettings.value = settings;
  schedulingForm.desiredRetentionPercent = settings.desiredRetention * 100;
  schedulingForm.maximumIntervalDays = settings.maximumIntervalDays;
  schedulingSaveAttempted.value = false;
}

function clearPreferenceFeedback() {
  appearanceError.value = '';
  appearanceStatus.value = '';
  gradingModeError.value = '';
  gradingModeStatus.value = '';
  pretestingError.value = '';
  pretestingStatus.value = '';
  startupDestinationError.value = '';
  startupDestinationStatus.value = '';
}

function appearanceOptionAllowed( field, value ) {
  const optionsByField = {
    theme: themeOptions,
    readingFont: readingFontOptions,
    readingTextSize: readingTextSizeOptions,
    motionPreference: motionOptions
  };

  return optionsByField[ field ]?.some( ( option ) => option.value === value );
}

function updateAppearanceField( field, value ) {
  if (
    !appearanceOptionAllowed( field, value )
    || appearance.value[ field ] === value
  ) {
    return;
  }

  return persistAppearance({
    ...appearance.value,
    [ field ]: value
  });
}

function restoreAppearanceDefaults() {
  if ( appearanceDefaultsActive.value ) {
    return;
  }

  return persistAppearance(
    DEFAULT_APPEARANCE,
    'Appearance defaults restored.'
  );
}

async function persistAppearance(
  nextAppearance,
  successMessage = 'Appearance saved.'
) {
  const request = ++appearanceRequestSequence;

  appearanceError.value = '';
  appearanceStatus.value = '';
  appearancePendingCount.value += 1;

  try {
    await setAppearance( nextAppearance );

    if ( viewActive && request === appearanceRequestSequence ) {
      appearanceStatus.value = successMessage;
    }
  } catch ( cause ) {
    if ( viewActive && request === appearanceRequestSequence ) {
      appearanceError.value = conceptLibraryErrorMessage( cause );
    }
  } finally {
    appearancePendingCount.value = Math.max(
      0,
      appearancePendingCount.value - 1
    );
  }
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

function scrollToSection( sectionId ) {
  document.getElementById( `settings-${ sectionId }` )?.scrollIntoView({
    behavior: 'smooth',
    block: 'start'
  });
}

async function updateStartupDestination(
  nextDestination,
  successMessage = 'Startup destination saved.'
) {
  if (
    startupDestinationPending.value
    || !startupDestinationItems.some( ( item ) => item.value === nextDestination )
    || nextDestination === savedStartupDestination.value
  ) {
    return;
  }

  const previousDestination = savedStartupDestination.value;

  startupDestination.value = nextDestination;
  startupDestinationError.value = '';
  startupDestinationStatus.value = '';
  startupDestinationPending.value = true;

  try {
    const preferences = await setStartupDestination( nextDestination );

    if ( !viewActive ) {
      return;
    }

    startupDestination.value = preferences.startupDestination;
    savedStartupDestination.value = preferences.startupDestination;
    startupDestinationStatus.value = successMessage;
  } catch ( cause ) {
    if ( viewActive ) {
      startupDestination.value = previousDestination;
      startupDestinationError.value = conceptLibraryErrorMessage( cause );
    }
  } finally {
    if ( viewActive ) {
      startupDestinationPending.value = false;
    }
  }
}

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
  gradingModeStatus.value = '';
  pretestingStatus.value = '';
  gradingModePending.value = true;

  try {
    const preferences = await setGradingMode( nextMode );

    if ( !viewActive ) {
      return;
    }

    gradingMode.value = preferences.gradingMode;
    savedGradingMode.value = preferences.gradingMode;
    gradingModeStatus.value = successMessage;
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
  gradingModeStatus.value = '';
  pretestingError.value = '';
  pretestingStatus.value = '';
  pretestingPending.value = true;

  try {
    const preferences = await setPretestingEnabled( enabled );

    if ( !viewActive ) {
      return;
    }

    pretestingEnabled.value = preferences.pretestingEnabled;
    savedPretestingEnabled.value = preferences.pretestingEnabled;
    pretestingStatus.value = successMessage;
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

async function restoreStudyDefaults() {
  if ( gradingModePending.value || pretestingPending.value ) {
    return;
  }

  if ( savedPretestingEnabled.value !== DEFAULT_PRETESTING_ENABLED ) {
    await updatePretesting( DEFAULT_PRETESTING_ENABLED, '' );
  }

  if ( savedGradingMode.value !== DEFAULT_GRADING_MODE ) {
    await updateGradingMode( DEFAULT_GRADING_MODE, '' );
  }

  if ( !pretestingError.value && !gradingModeError.value ) {
    gradingModeStatus.value = 'Study defaults restored.';
  }
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
  <div
    class="page settings-page"
    data-twill-page="settings"
  >
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
      <nav
        class="settings-navigation"
        aria-label="Settings sections"
      >
        <UButton
          v-for="section in settingsSections"
          :key="section.id"
          type="button"
          color="neutral"
          variant="ghost"
          :leading-icon="section.icon"
          :aria-controls="`settings-${ section.id }`"
          @click="scrollToSection( section.id )"
        >
          {{ section.label }}
        </UButton>
      </nav>

      <div class="settings-sections">
        <m.section
          id="settings-general"
          class="settings-panel"
          data-twill-settings-section="general"
          :initial="{ opacity: 0, y: 10 }"
          :animate="{ opacity: 1, y: 0 }"
          :transition="panelTransition"
        >
          <header class="settings-panel__header">
            <span
              class="settings-panel__icon"
              aria-hidden="true"
            >
              <UIcon name="i-lucide-settings-2" />
            </span>

            <div>
              <h2>General</h2>
              <p>Choose what opens when Twill starts.</p>
            </div>
          </header>

          <div class="settings-preference-row">
            <div>
              <label for="startup-destination">Startup destination</label>
              <p>Deep links still open their requested page.</p>
            </div>

            <USelect
              id="startup-destination"
              :model-value="startupDestination"
              :items="startupDestinationItems"
              :loading="startupDestinationPending"
              :disabled="startupDestinationPending"
              value-key="value"
              leading-icon="i-lucide-house"
              class="settings-select"
              @update:model-value="updateStartupDestination"
            />
          </div>

          <UAlert
            v-if="startupDestinationError"
            :description="startupDestinationError"
            icon="i-lucide-circle-alert"
            color="error"
            variant="subtle"
          />

          <footer class="settings-section-actions">
            <p
              class="settings-save-status"
              aria-live="polite"
            >
              {{ startupDestinationStatus }}
            </p>

            <UButton
              type="button"
              color="neutral"
              variant="link"
              :disabled="savedStartupDestination === DEFAULT_STARTUP_DESTINATION
                || startupDestinationPending"
              @click="updateStartupDestination(
                DEFAULT_STARTUP_DESTINATION,
                'General defaults restored.'
              )"
            >
              Restore default
            </UButton>
          </footer>
        </m.section>

        <m.section
          id="settings-appearance"
          class="settings-panel"
          data-twill-settings-section="appearance"
          :initial="{ opacity: 0, y: 10 }"
          :animate="{ opacity: 1, y: 0 }"
          :transition="{ ...panelTransition, delay: 0.04 }"
        >
          <header class="settings-panel__header">
            <span
              class="settings-panel__icon"
              aria-hidden="true"
            >
              <UIcon name="i-lucide-palette" />
            </span>

            <div>
              <h2>Appearance</h2>
              <p>Control color, reading type, and motion.</p>
            </div>
          </header>

          <fieldset class="appearance-fieldset">
            <legend>Theme</legend>
            <p>Choose a complete dark or light color theme.</p>

            <section
              v-for="group in themeGroups"
              :key="group.id"
              class="appearance-theme-group"
              :aria-labelledby="`appearance-${ group.id }-themes`"
            >
              <h3 :id="`appearance-${ group.id }-themes`">
                {{ group.label }}
              </h3>

              <div class="appearance-theme-grid">
                <label
                  v-for="theme in group.themes"
                  :key="theme.value"
                  class="appearance-theme-option"
                  :class="{
                    'appearance-theme-option--selected': appearance.theme
                      === theme.value
                  }"
                >
                  <input
                    type="radio"
                    name="appearance-theme"
                    :value="theme.value"
                    :checked="appearance.theme === theme.value"
                    :aria-label="`${ theme.label }. ${ group.label } theme. ${ theme.description }`"
                    @change="updateAppearanceField('theme', theme.value)"
                  >

                  <span
                    class="appearance-theme-preview"
                    :style="{
                      backgroundColor: theme.preview.background,
                      color: theme.preview.text
                    }"
                    aria-hidden="true"
                  >
                    <span
                      class="appearance-theme-preview__rail"
                      :style="{ backgroundColor: theme.preview.surface }"
                    >
                      <i :style="{ backgroundColor: theme.preview.accent }" />
                      <i :style="{ backgroundColor: theme.preview.text }" />
                      <i :style="{ backgroundColor: theme.preview.text }" />
                    </span>

                    <span class="appearance-theme-preview__canvas">
                      <i
                        class="appearance-theme-preview__title"
                        :style="{ backgroundColor: theme.preview.text }"
                      />
                      <i
                        class="appearance-theme-preview__line appearance-theme-preview__line--long"
                        :style="{ backgroundColor: theme.preview.text }"
                      />
                      <i
                        class="appearance-theme-preview__line"
                        :style="{ backgroundColor: theme.preview.text }"
                      />
                      <i
                        class="appearance-theme-preview__accent"
                        :style="{ backgroundColor: theme.preview.accent }"
                      />
                    </span>
                  </span>

                  <span class="appearance-theme-copy">
                    <strong>{{ theme.label }}</strong>
                    <span>{{ theme.description }}</span>
                  </span>

                  <span
                    class="appearance-theme-check"
                    :class="{
                      'appearance-theme-check--visible': appearance.theme
                        === theme.value
                    }"
                    aria-hidden="true"
                  >
                    <UIcon name="i-lucide-check" />
                  </span>
                </label>
              </div>
            </section>
          </fieldset>

          <div class="settings-preference-row">
            <div>
              <label for="appearance-reading-font">Reading font</label>
              <p>Used for authored content, previews, and study cards.</p>
            </div>

            <USelect
              id="appearance-reading-font"
              :model-value="appearance.readingFont"
              :items="readingFontOptions"
              value-key="value"
              leading-icon="i-lucide-type"
              class="settings-select"
              @update:model-value="updateAppearanceField('readingFont', $event)"
            />
          </div>

          <div class="appearance-reading-sample">
            <span>Reading preview</span>
            <p>The cell membrane controls what enters and leaves the cell.</p>
          </div>

          <div class="settings-preference-row">
            <div>
              <label for="appearance-reading-size">Study text size</label>
              <p>Changes prompts and answers without scaling the interface.</p>
            </div>

            <USelect
              id="appearance-reading-size"
              :model-value="appearance.readingTextSize"
              :items="readingTextSizeOptions"
              value-key="value"
              leading-icon="i-lucide-case-sensitive"
              class="settings-select"
              @update:model-value="updateAppearanceField('readingTextSize', $event)"
            />
          </div>

          <div class="settings-preference-row">
            <div>
              <label for="appearance-motion">Motion</label>
              <p>Reduced motion limits transitions and animated feedback.</p>
            </div>

            <USelect
              id="appearance-motion"
              :model-value="appearance.motionPreference"
              :items="motionOptions"
              value-key="value"
              leading-icon="i-lucide-gauge"
              class="settings-select"
              @update:model-value="updateAppearanceField('motionPreference', $event)"
            />
          </div>

          <UAlert
            v-if="appearanceError"
            :description="appearanceError"
            icon="i-lucide-circle-alert"
            color="error"
            variant="subtle"
          />

          <footer class="settings-section-actions">
            <p
              class="settings-save-status"
              aria-live="polite"
            >
              {{ appearanceSaveStatus }}
            </p>

            <UButton
              type="button"
              color="neutral"
              variant="link"
              :disabled="appearanceDefaultsActive"
              @click="restoreAppearanceDefaults"
            >
              Restore defaults
            </UButton>
          </footer>
        </m.section>

        <m.section
          id="settings-snippets"
          class="settings-panel"
          data-twill-settings-section="snippets"
          :initial="{ opacity: 0, y: 10 }"
          :animate="{ opacity: 1, y: 0 }"
          :transition="{ ...panelTransition, delay: 0.08 }"
        >
          <header class="settings-panel__header">
            <span
              class="settings-panel__icon"
              aria-hidden="true"
            >
              <UIcon name="i-lucide-braces" />
            </span>

            <div>
              <h2>CSS snippets</h2>
              <p>Apply validated CSS after Twill's built-in styles.</p>
            </div>
          </header>

          <CssSnippetManager />
        </m.section>

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
              <p>Set the controls used for review sessions.</p>
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
            <p
              class="settings-save-status"
              aria-live="polite"
            >
              {{ pretestingStatus || gradingModeStatus }}
            </p>

            <UButton
              type="button"
              color="neutral"
              variant="link"
              :disabled="(savedGradingMode === DEFAULT_GRADING_MODE
                && savedPretestingEnabled === DEFAULT_PRETESTING_ENABLED)
                || gradingModePending
                || pretestingPending"
              @click="restoreStudyDefaults"
            >
              Restore default
            </UButton>
          </footer>
        </m.section>

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
      </div>
    </div>
  </div>
</template>
