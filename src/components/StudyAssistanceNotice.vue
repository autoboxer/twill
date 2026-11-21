<script setup>
defineProps({
  assisted: { type: Boolean, required: true },
  revealed: { type: Boolean, required: true },
  response: { type: String, required: true },
  originalResponse: { type: String, required: true },
  grading: { type: Boolean, default: true }
});
</script>

<template>
  <div
    v-if="assisted"
    class="study-assistance-notice"
    data-twill-assisted
    role="status"
  >
    <strong>Help used</strong>
    <span>{{ grading ? 'Grade your answer before help.' : 'Compare your response before help.' }}</span>
    <div
      v-if="revealed && response && response !== originalResponse"
      class="study-assisted-response"
    >
      <span>Your response after help</span>
      <p>{{ response }}</p>
    </div>
  </div>
</template>
