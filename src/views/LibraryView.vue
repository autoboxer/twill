<script setup>
import ContentState from '../components/ContentState.vue';
import PageHeader from '../components/PageHeader.vue';
</script>

<template>
  <div class="page">
    <PageHeader title="Library" />

    <ContentState
      title="No concepts yet"
      description="Concepts will appear here after you create them."
    >
      <template #actions>
        <UButton
          to="/create"
          leading-icon="i-lucide-square-pen"
          size="lg"
        >
          Create concept
        </UButton>
      </template>
    </ContentState>
  </div>
</template>
