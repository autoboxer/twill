<script setup>
import { computed, nextTick, ref, watch } from 'vue';

import StudyGuideContent from './StudyGuideContent.vue';
import { useStudyHelp } from '../composables/useStudyHelp';
import { guideTopic, guideTopics } from '../guide/topics';

const help = useStudyHelp();
const { isOpen, topicId } = help;
const reader = ref( null );
const body = ref( null );
const topic = computed( () => guideTopic( topicId.value ) ?? guideTopic( 'start' ) );
const relatedTopics = computed( () => topic.value.related.map( guideTopic ) );
const topicItems = guideTopics.map( ( item ) => ({ label: item.title, value: item.id }) );
let focusAfterSelection = false;

watch( topicId, () => {
  if ( !focusAfterSelection ) {
    focusTopic();
  }
}, { flush: 'post' });

async function focusTopic() {
  await nextTick();
  body.value?.closest( '[data-slot="body"]' )?.scrollTo({ top: 0 });
  reader.value?.focus();
}

function focusOnOpen( event ) {
  event.preventDefault();
  focusTopic();
}

function selectTopic( nextTopic ) {
  focusAfterSelection = true;
  topicId.value = nextTopic;
}

function finishTopicSelection( event ) {
  if ( !focusAfterSelection ) {
    return;
  }

  event.preventDefault();
  focusAfterSelection = false;
  focusTopic();
}
</script>

<template>
  <UModal
    v-model:open="isOpen"
    title="Using Twill"
    class="study-help-dialog"
    :content="{
      onOpenAutoFocus: focusOnOpen,
      onCloseAutoFocus: help.restoreFocus
    }"
    :ui="{ content: 'study-help-dialog__content' }"
  >
    <template #body>
      <div ref="body" data-twill-study-help>
        <USelect
          :model-value="topicId"
          :items="topicItems"
          :content="{ onCloseAutoFocus: finishTopicSelection }"
          value-key="value"
          aria-label="Help topic"
          class="study-help-topics"
          @update:model-value="selectTopic"
        />

        <StudyGuideContent
          ref="reader"
          :topic="topic"
          heading-prefix="help-topic"
        />

        <nav v-if="relatedTopics.length" class="study-guide-related" aria-label="Related help topics">
          <h2>Related topics</h2>

          <button
            v-for="related in relatedTopics"
            :key="related.id"
            type="button"
            class="study-guide-topic-link"
            @click="topicId = related.id"
          >
            {{ related.title }}
          </button>
        </nav>
      </div>
    </template>
  </UModal>
</template>
