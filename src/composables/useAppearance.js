import { isTauri } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { computed, readonly, ref } from 'vue';

import {
  DEFAULT_APPEARANCE,
  normalizeAppearance,
  themeOptions
} from '../appearance/options';
import { loadFrameFontCss } from '../appearance/frameFonts';
import { useDevicePreferences } from './useDevicePreferences';

const reducedMotionQuery = window.matchMedia( '(prefers-reduced-motion: reduce)' );
const preferences = ref( normalizeAppearance( DEFAULT_APPEARANCE ) );
const persistedPreferences = ref( normalizeAppearance( DEFAULT_APPEARANCE ) );
const ready = ref( false );
const loadError = ref( '' );
const resolvedColorMode = ref( resolveColorMode() );
const resolvedMotion = ref( resolveMotion() );
const frameAppearance = ref({});
const { getDevicePreferences, setAppearancePreferences } = useDevicePreferences();

let initialized = false;
let initializationPromise = null;
let persistenceQueue = Promise.resolve();
let preferenceRevision = 0;
let frameAppearanceRevision = 0;

export const motionConfigPreference = computed( () => {
  if ( preferences.value.motionPreference === 'full' ) {
    return 'never';
  }

  if ( preferences.value.motionPreference === 'reduced' ) {
    return 'always';
  }

  return 'user';
});

export function initializeAppearance() {
  if ( initialized ) {
    return initializationPromise;
  }

  initialized = true;
  applyAppearance( preferences.value );
  reducedMotionQuery.addEventListener( 'change', handleSystemAppearanceChange );

  const startingRevision = preferenceRevision;

  initializationPromise = getDevicePreferences()
    .then( ( devicePreferences ) => {
      const appearance = normalizeAppearance( devicePreferences.appearance );

      persistedPreferences.value = appearance;

      if ( startingRevision === preferenceRevision ) {
        applyAppearance( appearance );
      }
    })
    .catch( () => {
      loadError.value = 'Appearance preferences could not be loaded.';
    })
    .finally( () => {
      ready.value = true;
    });

  return initializationPromise;
}

export function useAppearance() {
  return {
    appearance: readonly( preferences ),
    frameAppearance: readonly( frameAppearance ),
    ready: readonly( ready ),
    loadError: readonly( loadError ),
    motionConfigPreference,
    resolvedColorMode: readonly( resolvedColorMode ),
    resolvedMotion: readonly( resolvedMotion ),
    setAppearance
  };
}

export function setAppearance( value ) {
  const requested = normalizeAppearance( value );
  const requestRevision = ++preferenceRevision;

  applyAppearance( requested );

  const request = persistenceQueue
    .catch( () => undefined )
    .then( () => setAppearancePreferences( requested ) );

  persistenceQueue = request;

  return request
    .then( ( devicePreferences ) => {
      const stored = normalizeAppearance( devicePreferences.appearance );

      persistedPreferences.value = stored;

      if ( requestRevision === preferenceRevision ) {
        applyAppearance( stored );
      }

      return stored;
    })
    .catch( ( cause ) => {
      if ( requestRevision === preferenceRevision ) {
        applyAppearance( persistedPreferences.value );
      }

      throw cause;
    });
}

function applyAppearance( value ) {
  const next = normalizeAppearance( value );
  const root = document.documentElement;

  preferences.value = next;
  resolvedColorMode.value = resolveColorMode( next );
  resolvedMotion.value = resolveMotion( next );

  root.dataset.theme = next.theme;
  root.dataset.colorMode = resolvedColorMode.value;
  root.dataset.readingFont = next.readingFont;
  root.dataset.readingTextSize = next.readingTextSize;
  root.dataset.motion = resolvedMotion.value;
  root.classList.toggle( 'dark', resolvedColorMode.value === 'dark' );

  updateThemeColor();
  syncNativeTheme( resolvedColorMode.value );
}

function handleSystemAppearanceChange() {
  applyAppearance( preferences.value );
}

function resolveColorMode( value = preferences.value ) {
  const theme = themeOptions.find( ( option ) => option.value === value.theme );

  return theme?.mode ?? 'dark';
}

function resolveMotion( value = preferences.value ) {
  if ( value.motionPreference === 'system' ) {
    return reducedMotionQuery.matches ? 'reduced' : 'full';
  }

  return value.motionPreference;
}

function updateThemeColor() {
  requestAnimationFrame( () => {
    const updateRevision = ++frameAppearanceRevision;
    const rootStyles = getComputedStyle( document.documentElement );
    const background = rootStyles.getPropertyValue( '--ui-bg' ).trim();
    const themeColor = document.querySelector( 'meta[name="theme-color"]' );
    const readingFontId = preferences.value.readingFont;
    const retainedFontCss = frameAppearance.value.readingFontId === readingFontId
      ? frameAppearance.value.fontCss ?? ''
      : '';

    themeColor?.setAttribute( 'content', background );
    const nextFrameAppearance = {
      colorScheme: resolvedColorMode.value,
      background,
      surface: rootStyles.getPropertyValue( '--ui-bg-elevated' ).trim(),
      text: rootStyles.getPropertyValue( '--ui-text' ).trim(),
      highlightedText: rootStyles.getPropertyValue( '--ui-text-highlighted' ).trim(),
      mutedText: rootStyles.getPropertyValue( '--ui-text-muted' ).trim(),
      border: rootStyles.getPropertyValue( '--ui-border' ).trim(),
      primary: rootStyles.getPropertyValue( '--ui-primary' ).trim(),
      fontCss: retainedFontCss,
      readingFont: rootStyles.getPropertyValue( '--font-reading' ).trim(),
      readingFontId,
      readingTextSize: rootStyles
        .getPropertyValue( '--reading-text-size' )
        .trim(),
      codeBackground: rootStyles
        .getPropertyValue( '--twill-code-background' )
        .trim(),
      codeText: rootStyles.getPropertyValue( '--twill-code-text' ).trim(),
      syntaxComment: rootStyles
        .getPropertyValue( '--twill-syntax-comment' )
        .trim(),
      syntaxKeyword: rootStyles
        .getPropertyValue( '--twill-syntax-keyword' )
        .trim(),
      syntaxString: rootStyles
        .getPropertyValue( '--twill-syntax-string' )
        .trim(),
      syntaxNumber: rootStyles
        .getPropertyValue( '--twill-syntax-number' )
        .trim(),
      syntaxTitle: rootStyles
        .getPropertyValue( '--twill-syntax-title' )
        .trim()
    };

    frameAppearance.value = nextFrameAppearance;

    void loadFrameFontCss( readingFontId )
      .then( ( fontCss ) => {
        if ( updateRevision === frameAppearanceRevision ) {
          frameAppearance.value = { ...nextFrameAppearance, fontCss };
        }
      })
      .catch( () => undefined );
  });
}

function syncNativeTheme( colorMode ) {
  if ( !isTauri() ) {
    return;
  }

  void getCurrentWindow().setTheme( colorMode ).catch( () => undefined );
}
