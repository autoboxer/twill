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
import { provideCommands } from '../composables/useCommands';

const routeTransition = {
  duration: 0.18,
  ease: [ 0.22, 1, 0.36, 1 ]
};

const animateRouteEntrance = ref( false );
const router = useRouter();

provideCommands( router );

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
    </LazyMotion>
  </MotionConfig>
</template>
