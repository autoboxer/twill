import ui from '@nuxt/ui/vue-plugin';
import { createApp } from 'vue';

import '@fontsource-variable/ibm-plex-sans/wght.css';
import '@fontsource-variable/ibm-plex-sans/wght-italic.css';
import '@fontsource-variable/inter/wght.css';
import '@fontsource-variable/inter/wght-italic.css';
import '@fontsource-variable/jetbrains-mono/wght.css';
import '@fontsource-variable/jetbrains-mono/wght-italic.css';
import '@fontsource-variable/source-serif-4/wght.css';
import '@fontsource-variable/source-serif-4/wght-italic.css';

import App from './App.vue';
import { initializeAppearance } from './composables/useAppearance';
import { initializeCssSnippets } from './composables/useCssSnippets';
import { initializeNativeLifecycle } from './composables/useNativeLifecycle';
import router from './router';
import { isStartupPending } from './startup';
import './styles/main.css';

export async function startApplication() {
  const app = createApp( App );

  app.use( router );
  app.use( ui );

  await Promise.all([
    initializeAppearance(),
    initializeNativeLifecycle(),
    initializeCssSnippets(),
    router.isReady()
  ]);

  if ( isStartupPending() ) {
    app.mount( '#app' );
  }
}
