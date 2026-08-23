<script setup>
import {
  AnimatePresence,
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
import { provideCommands } from '../composables/useCommands';

const routeTransition = {
  duration: 0.18,
  ease: [ 0.22, 1, 0.36, 1 ]
};

const animateRouteEntrance = ref( false );
const router = useRouter();
const commands = provideCommands( router );
const paletteCommand = commands.command( COMMAND_IDS.commandPaletteOpen );
const referenceCommand = commands.command( COMMAND_IDS.commandReferenceOpen );

onMounted( () => {
  animateRouteEntrance.value = true;
});
</script>

<template>
  <MotionConfig
    :transition="routeTransition"
    reduced-motion="user"
  >
    <a
      class="skip-link"
      href="#main-content"
    >
      Skip to content
    </a>

    <LazyMotion :features="domAnimation">
      <div class="twill-shell">
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

        <div class="app-viewport">
          <RouterView v-slot="{ Component, route }">
            <span
              class="sr-only"
              aria-live="polite"
            >
              {{ route.meta.title }}
            </span>

            <AnimatePresence
              mode="wait"
              :initial="false"
            >
              <m.main
                id="main-content"
                :key="route.name"
                class="route-content"
                tabindex="-1"
                :initial="animateRouteEntrance ? { opacity: 0, y: 6 } : false"
                :animate="{ opacity: 1, y: 0 }"
                :exit="{ opacity: 0, y: -4 }"
              >
                <component :is="Component" />
              </m.main>
            </AnimatePresence>
          </RouterView>
        </div>
      </div>

      <CommandCenter />
    </LazyMotion>
  </MotionConfig>
</template>
