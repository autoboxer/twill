<script setup>
import StudyIdeaCheck from './StudyIdeaCheck.vue';

defineProps({
  items: {
    type: Array,
    required: true
  },
  kind: {
    type: String,
    required: true
  },
  checks: {
    type: Object,
    default: () => ({})
  },
  writtenResponse: {
    type: Boolean,
    default: false
  },
  disabled: {
    type: Boolean,
    default: false
  }
});

const emit = defineEmits([ 'compare' ]);
</script>

<template>
  <component
    :is="kind === 'checkpoint' ? 'ol' : 'ul'"
    class="study-idea-list"
  >
    <li
      v-for="( item, index ) in items"
      :key="index"
      class="study-idea-list__item"
    >
      <span class="study-idea-list__text">{{ item }}</span>
      <StudyIdeaCheck
        :label="`${ kind } ${ index + 1 }`"
        :model-value="checks[ `${ kind }:${ index }` ]"
        :written-response="writtenResponse"
        :disabled="disabled"
        @update:model-value="emit( 'compare', `${ kind }:${ index }`, $event )"
      />
    </li>
  </component>
</template>
