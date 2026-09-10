<script setup>
import { m } from 'motion-v';
import { onBeforeUnmount, onMounted, ref } from 'vue';

import ContentState from '../components/ContentState.vue';
import CssSnippetManager from '../components/CssSnippetManager.vue';
import PageHeader from '../components/PageHeader.vue';
import AppearanceSettings from '../components/settings/AppearanceSettings.vue';
import GeneralSettings from '../components/settings/GeneralSettings.vue';
import SchedulingSettings from '../components/settings/SchedulingSettings.vue';
import StudySettings from '../components/settings/StudySettings.vue';
import { conceptLibraryErrorMessage } from '../composables/useConceptLibrary';
import { useDevicePreferences } from '../composables/useDevicePreferences';
import { useSchedulingSettings } from '../composables/useSchedulingSettings';

const settingsSections = [
  { id: 'general', icon: 'i-lucide-settings-2', label: 'General' },
  { id: 'appearance', icon: 'i-lucide-palette', label: 'Appearance' },
  { id: 'snippets', icon: 'i-lucide-braces', label: 'Snippets' },
  { id: 'study', icon: 'i-lucide-graduation-cap', label: 'Study' },
  { id: 'scheduling', icon: 'i-lucide-calendar-sync', label: 'Scheduling' }
];

const panelTransition = {
  duration: 0.24,
  ease: [ 0.22, 1, 0.36, 1 ]
};

const { getDevicePreferences } = useDevicePreferences();
const { getSchedulingSettings } = useSchedulingSettings();

const devicePreferences = ref( null );
const schedulingSettings = ref( null );
const initialLoading = ref( true );
const loadError = ref( '' );
let loadRequestSequence = 0;

onMounted( loadSettings );

onBeforeUnmount( () => {
  loadRequestSequence += 1;
});

async function loadSettings() {
  const request = ++loadRequestSequence;

  initialLoading.value = true;
  loadError.value = '';

  try {
    const [ preferences, settings ] = await Promise.all([
      getDevicePreferences(),
      getSchedulingSettings()
    ]);

    if ( request !== loadRequestSequence ) {
      return;
    }

    devicePreferences.value = preferences;
    schedulingSettings.value = settings;
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

function scrollToSection( sectionId ) {
  document.getElementById( `settings-${ sectionId }` )?.scrollIntoView({
    behavior: 'smooth',
    block: 'start'
  });
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
        <GeneralSettings
          :initial-destination="devicePreferences.startupDestination"
          :panel-transition="panelTransition"
        />

        <AppearanceSettings
          :panel-transition="panelTransition"
        />

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

        <StudySettings
          :initial-preferences="devicePreferences"
          :panel-transition="panelTransition"
        />

        <SchedulingSettings
          :initial-settings="schedulingSettings"
          :panel-transition="panelTransition"
        />
      </div>
    </div>
  </div>
</template>
