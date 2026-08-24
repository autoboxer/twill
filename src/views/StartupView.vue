<script setup>
import { onBeforeUnmount, onMounted } from 'vue';
import { useRouter } from 'vue-router';

import ContentState from '../components/ContentState.vue';
import { useDevicePreferences } from '../composables/useDevicePreferences';

const router = useRouter();
const { getDevicePreferences } = useDevicePreferences();
let viewActive = true;

onMounted( openStartupDestination );

onBeforeUnmount( () => {
  viewActive = false;
});

async function openStartupDestination() {
  let destination = 'study';

  try {
    const preferences = await getDevicePreferences();

    if ([ 'study', 'library' ].includes( preferences.startupDestination ) ) {
      destination = preferences.startupDestination;
    }
  } catch {
    // Study remains available when preferences cannot be read
  }

  if ( !viewActive || router.currentRoute.value.name !== 'startup' ) {
    return;
  }

  await router.replace({ name: destination });
}
</script>

<template>
  <ContentState
    class="startup-state"
    data-twill-page="startup"
    kind="loading"
    title="Opening Twill"
  />
</template>
