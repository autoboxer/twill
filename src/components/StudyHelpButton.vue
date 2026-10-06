<script setup>
import { useStudyHelp } from '../composables/useStudyHelp';

defineProps({
  topic: {
    type: String,
    required: true
  },
  label: {
    type: String,
    default: 'Help'
  }
});

const help = useStudyHelp();
</script>

<template>
  <UTooltip :text="label">
    <UButton
      type="button"
      icon="i-lucide-circle-help"
      color="neutral"
      variant="ghost"
      size="sm"
      :aria-label="label"
      aria-haspopup="dialog"
      :data-twill-help="topic"
      @click="help.show( topic, $event.currentTarget )"
    />
  </UTooltip>
</template>
