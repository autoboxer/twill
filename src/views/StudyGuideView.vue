<script setup>
import { computed, nextTick, ref, watch } from 'vue';
import { useRoute } from 'vue-router';

import PageHeader from '../components/PageHeader.vue';
import StudyGuideContent from '../components/StudyGuideContent.vue';
import { useStartupReady } from '../composables/useStartupReady';
import { guideGroups, guideTopic, searchGuide } from '../guide/topics';

const route = useRoute();
const query = ref( '' );
const contentsOpen = ref( false );
const reader = ref( null );
const searchInput = ref( null );
const topic = computed( () => guideTopic( route.params.topicId || 'start' ) );
const matchingTopics = computed( () => searchGuide( query.value ) );
const searching = computed( () => Boolean( query.value.trim() ) );
const relatedTopics = computed( () => topic.value?.related.map( guideTopic ) ?? []);

watch( () => route.params.topicId, async () => {
  contentsOpen.value = false;

  await nextTick();

  reader.value?.focus();
});

function clearSearch() {
  query.value = '';
  searchInput.value?.inputRef?.focus();
}

async function selectTopic() {
  contentsOpen.value = false;

  await nextTick();

  reader.value?.focus();
}

useStartupReady( false );
</script>

<template>
  <div class="page study-guide-page" data-twill-page="guide">
    <PageHeader title="Using Twill">
      <template #actions>
        <UButton
          :aria-expanded="contentsOpen"
          aria-controls="guide-contents"
          leading-icon="i-lucide-list"
          color="neutral"
          variant="subtle"
          class="study-guide-contents-toggle"
          @click="contentsOpen = !contentsOpen"
        >
          Topics
        </UButton>
      </template>
    </PageHeader>

    <div class="study-guide-layout">
      <aside
        id="guide-contents"
        class="study-guide-contents"
        :class="{ 'study-guide-contents--open': contentsOpen }"
        aria-label="Guide contents"
      >
        <label class="sr-only" for="guide-search">Search guide</label>

        <UInput
          id="guide-search"
          ref="searchInput"
          v-model="query"
          type="search"
          placeholder="Search guide"
          leading-icon="i-lucide-search"
          class="study-guide-search"
        >
          <template v-if="query" #trailing>
            <UButton
              icon="i-lucide-x"
              aria-label="Clear guide search"
              size="xs"
              color="neutral"
              variant="ghost"
              @click="clearSearch"
            />
          </template>
        </UInput>

        <nav v-if="searching" aria-label="Guide search results">
          <p class="study-guide-results" role="status">
            Found {{ matchingTopics.length }} {{ matchingTopics.length === 1 ? 'topic' : 'topics' }}.
          </p>

          <RouterLink
            v-for="result in matchingTopics"
            :key="result.id"
            :to="{ name: 'guide', params: { topicId: result.id } }"
            :aria-current="result.id === topic?.id ? 'page' : undefined"
            class="study-guide-topic-link"
            @click="selectTopic"
          >
            {{ result.title }}
          </RouterLink>

          <p v-if="!matchingTopics.length" class="study-guide-results">
            Search for a card type or a word such as grading, draft, or feedback.
          </p>
        </nav>

        <nav v-else aria-label="Guide topics">
          <div v-for="group in guideGroups" :key="group.title" class="study-guide-group">
            <h2>{{ group.title }}</h2>

            <RouterLink
              v-for="topicId in group.topics"
              :key="topicId"
              :to="{ name: 'guide', params: { topicId } }"
              :aria-current="topicId === topic?.id ? 'page' : undefined"
              class="study-guide-topic-link"
              @click="selectTopic"
            >
              {{ guideTopic( topicId ).title }}
            </RouterLink>
          </div>
        </nav>
      </aside>

      <div class="study-guide-reader">
        <StudyGuideContent v-if="topic" ref="reader" :topic="topic" />

        <div v-else class="study-guide-content">
          <h2>Topic not found</h2>
          <p>Choose a topic from the guide contents.</p>
          <RouterLink :to="{ name: 'guide' }" class="study-guide-topic-link">
            Start studying
          </RouterLink>
        </div>

        <nav v-if="relatedTopics.length" class="study-guide-related" aria-label="Related guide topics">
          <h2>Related topics</h2>

          <RouterLink
            v-for="related in relatedTopics"
            :key="related.id"
            :to="{ name: 'guide', params: { topicId: related.id } }"
            class="study-guide-topic-link"
          >
            {{ related.title }}
          </RouterLink>
        </nav>
      </div>
    </div>
  </div>
</template>
