<script setup>
import AppShell from './components/AppShell.vue';
import NativeActionDialog from './components/NativeActionDialog.vue';
import { useNativeLifecycle } from './composables/useNativeLifecycle';
import { useNotificationViewport } from './composables/useNotificationViewport';

const { pending } = useNativeLifecycle();
const { position } = useNotificationViewport();
</script>

<template>
  <UApp :toaster="{ position, duration: 4000, max: 2, progress: false }">
    <div
      class="contents"
      :inert="pending"
    >
      <AppShell />
    </div>

    <NativeActionDialog />
  </UApp>
</template>
