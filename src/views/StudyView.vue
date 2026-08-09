<script setup>
import ContentState from '../components/ContentState.vue';
import PageHeader from '../components/PageHeader.vue';
</script>

<template>
  <div class="page">
    <PageHeader title="Study" />

    <ContentState
      title="No cards are due"
      description="Create a concept or open the library."
    >
      <template #actions>
        <UButton
          to="/create"
          leading-icon="i-lucide-square-pen"
          size="lg"
        >
          Create concept
        </UButton>

        <UButton
          to="/library"
          leading-icon="i-lucide-library"
          color="neutral"
          variant="subtle"
          size="lg"
        >
          Open library
        </UButton>
      </template>
    </ContentState>
  </div>
</template>
