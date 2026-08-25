<script setup>
import { m } from 'motion-v';
import { computed, ref } from 'vue';

import { richDocumentHasContent } from '../rich-content/schema';
import RichContentRenderer from './RichContentRenderer.vue';

const props = defineProps({
  feedback: {
    type: Object,
    required: true
  }
});

const root = ref( null );

const documents = computed( () => [
  {
    document: props.feedback.explanation,
    id: 'explanation',
    label: 'Explanation and context'
  },
  {
    document: props.feedback.commonMistakes,
    id: 'common-mistakes',
    label: 'Common mistakes'
  }
].filter( ( item ) => richDocumentHasContent( item.document ) ) );

defineExpose({ focus });

function focus() {
  const element = root.value?.$el ?? root.value;

  element?.focus();
}
</script>

<template>
  <m.section
    ref="root"
    class="study-answer-feedback"
    data-twill-answer-feedback
    tabindex="-1"
    aria-labelledby="study-answer-feedback-heading"
    :initial="{ opacity: 0, y: 8 }"
    :animate="{ opacity: 1, y: 0 }"
    :transition="{ duration: 0.2, ease: [ 0.22, 1, 0.36, 1 ] }"
  >
    <h3 id="study-answer-feedback-heading">Answer feedback</h3>

    <div class="study-answer-feedback__documents">
      <div
        v-for="item in documents"
        :key="item.id"
        class="study-answer-feedback__document"
      >
        <h4>{{ item.label }}</h4>

        <RichContentRenderer
          :document="item.document"
          :label="item.label"
        />
      </div>
    </div>
  </m.section>
</template>
