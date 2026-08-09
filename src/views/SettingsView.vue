<script setup>
import ContentState from '../components/ContentState.vue';
import PageHeader from '../components/PageHeader.vue';
</script>

<template>
  <div class="page">
    <PageHeader title="Settings" />

    <ContentState
      title="No settings yet"
      description="Theme follows your device."
    />
  </div>
</template>
