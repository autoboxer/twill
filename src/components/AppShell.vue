<script setup>
import {
  domAnimation,
  LazyMotion,
  m,
  MotionConfig
} from 'motion-v';
import { onMounted, ref } from 'vue';
import { useRouter } from 'vue-router';

import AppNavigation from './AppNavigation.vue';
import CommandCenter from './CommandCenter.vue';
import { COMMAND_IDS } from '../commands/registry';
import { useAppearance } from '../composables/useAppearance';
import { provideCommands } from '../composables/useCommands';

const routeTransition = {
  duration: 0.18,
  ease: [ 0.22, 1, 0.36, 1 ]
};

const animateRouteEntrance = ref( false );
const mainContent = ref( null );
const router = useRouter();
const commands = provideCommands( router );
const { motionConfigPreference, resolvedMotion } = useAppearance();
const paletteCommand = commands.command( COMMAND_IDS.commandPaletteOpen );
const referenceCommand = commands.command( COMMAND_IDS.commandReferenceOpen );

onMounted( () => {
  animateRouteEntrance.value = true;
});

function skipToContent() {
  mainContent.value?.focus({ preventScroll: true });
  mainContent.value?.scrollTo({ top: 0, behavior: 'instant' });
}
</script>

<template>
  <MotionConfig
    :transition="routeTransition"
    :reduced-motion="motionConfigPreference"
  >
    <a
      class="skip-link"
      href="#main-content"
      @click.prevent="skipToContent"
    >
      Skip to content
    </a>

    <LazyMotion :features="domAnimation">
      <div
        class="twill-shell"
        data-twill-app
      >
        <header class="mobile-header">
          <RouterLink
            class="mobile-wordmark"
            to="/study"
            aria-label="Twill study"
          >
            Twill
          </RouterLink>

          <div class="mobile-command-actions">
            <UButton
              icon="i-lucide-search"
              :aria-label="paletteCommand.label"
              :aria-keyshortcuts="paletteCommand.ariaKeyshortcuts"
              :title="paletteCommand.tooltip"
              color="neutral"
              variant="subtle"
              @click="commands.execute( COMMAND_IDS.commandPaletteOpen )"
            />

            <UButton
              icon="i-lucide-keyboard"
              :aria-label="referenceCommand.label"
              :aria-keyshortcuts="referenceCommand.ariaKeyshortcuts"
              :title="referenceCommand.tooltip"
              color="neutral"
              variant="subtle"
              @click="commands.execute( COMMAND_IDS.commandReferenceOpen )"
            />
          </div>
        </header>

        <AppNavigation />

        <main
          id="main-content"
          ref="mainContent"
          class="app-viewport"
          tabindex="-1"
        >
          <RouterView v-slot="{ Component, route }">
            <span
              class="sr-only"
              aria-live="polite"
            >
              {{ route.meta.title }}
            </span>

            <m.div
              :key="route.name"
              class="route-content"
              :initial="animateRouteEntrance && resolvedMotion === 'full'
                ? { opacity: 0.9 }
                : false"
              :animate="{ opacity: 1 }"
              :transition="{ duration: 0.12 }"
            >
              <component :is="Component" />
            </m.div>
          </RouterView>
        </main>
      </div>

      <CommandCenter />
    </LazyMotion>
  </MotionConfig>
</template>
