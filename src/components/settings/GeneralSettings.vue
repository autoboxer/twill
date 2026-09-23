<script setup>
import { m } from 'motion-v';
import { onBeforeUnmount, ref } from 'vue';

import { conceptLibraryErrorMessage } from '../../composables/useConceptLibrary';
import { useDevicePreferences } from '../../composables/useDevicePreferences';
import { useActionNotifications } from '../../composables/useActionNotifications';

const props = defineProps({
  initialDestination: {
    type: String,
    required: true
  },
  panelTransition: {
    type: Object,
    required: true
  }
});

const DEFAULT_STARTUP_DESTINATION = 'study';

const startupDestinationItems = [
  { label: 'Study', value: 'study' },
  { label: 'Library', value: 'library' }
];

const { setStartupDestination } = useDevicePreferences();
const { notifySuccess } = useActionNotifications();

const startupDestination = ref( props.initialDestination );
const savedStartupDestination = ref( props.initialDestination );

const startupDestinationError = ref( '' );
const startupDestinationPending = ref( false );

let viewActive = true;

onBeforeUnmount( () => {
  viewActive = false;
});

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
  startupDestinationPending.value = true;

  try {
    const preferences = await setStartupDestination( nextDestination );

    if ( !viewActive ) {
      return;
    }

    startupDestination.value = preferences.startupDestination;
    savedStartupDestination.value = preferences.startupDestination;
    notifySuccess( successMessage );
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
</script>

<template>
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
</template>
