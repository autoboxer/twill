<script setup>
defineProps({
  title: {
    type: String,
    required: true
  }
});
</script>

<template>
  <header class="page-header">
    <div
      v-if="$slots.leading"
      class="page-header__leading"
    >
      <slot name="leading" />
    </div>

    <h1>{{ title }}</h1>

    <div
      v-if="$slots.actions"
      class="page-header__actions"
    >
      <slot name="actions" />
    </div>
  </header>
</template>
