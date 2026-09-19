<script setup>
import { m } from 'motion-v';
import { computed, onBeforeUnmount, ref } from 'vue';

import {
  DEFAULT_APPEARANCE,
  motionOptions,
  readingFontOptions,
  readingTextSizeOptions,
  themeOptions
} from '../../appearance/options';
import { useAppearance } from '../../composables/useAppearance';
import { conceptLibraryErrorMessage } from '../../composables/useConceptLibrary';

defineProps({
  panelTransition: {
    type: Object,
    required: true
  }
});

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

const { appearance, setAppearance } = useAppearance();

const appearanceError = ref( '' );
const appearancePendingCount = ref( 0 );
const appearanceStatus = ref( '' );

let appearanceRequestSequence = 0;
let viewActive = true;

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

onBeforeUnmount( () => {
  viewActive = false;
  appearanceRequestSequence += 1;
});

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
</script>

<template>
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
      </div>
    </header>

    <fieldset class="appearance-fieldset">
      <legend>Theme</legend>

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
</template>
