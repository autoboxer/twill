<script setup>
defineProps({
  library: { type: Object, required: true },
  loading: { type: Boolean, default: false }
});

defineEmits([ 'change' ]);
</script>

<template>
  <nav
    v-if="library.totalCount > library.pageSize"
    class="library-pagination"
    aria-label="Library pages"
  >
    <UButton
      leading-icon="i-lucide-chevron-left"
      color="neutral"
      variant="subtle"
      :disabled="loading || library.page <= 1"
      @click="$emit( 'change', library.page - 1 )"
    >
      Previous
    </UButton>

    <span>Page {{ library.page }} of {{ Math.ceil( library.totalCount / library.pageSize ) }}</span>

    <UButton
      trailing-icon="i-lucide-chevron-right"
      color="neutral"
      variant="subtle"
      :disabled="loading || library.page * library.pageSize >= library.totalCount"
      @click="$emit( 'change', library.page + 1 )"
    >
      Next
    </UButton>
  </nav>
</template>
