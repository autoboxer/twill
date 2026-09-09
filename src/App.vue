<script setup>
import AppShell from './components/AppShell.vue';
import NativeActionDialog from './components/NativeActionDialog.vue';
import { useNativeLifecycle } from './composables/useNativeLifecycle';

const { pending } = useNativeLifecycle();
</script>

<template>
  <UApp>
    <div
      class="contents"
      :inert="pending"
    >
      <AppShell />
    </div>

    <NativeActionDialog />
  </UApp>
</template>
