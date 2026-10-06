<script setup>
import { ref } from 'vue';

defineProps({
  topic: {
    type: Object,
    required: true
  }
});

const heading = ref( null );

defineExpose({
  focus: () => heading.value?.focus({ preventScroll: true })
});
</script>

<template>
  <article
    class="study-guide-content"
    :aria-labelledby="`guide-topic-${ topic.id }`"
    data-twill-guide-topic
    :data-twill-topic="topic.id"
  >
    <h2
      :id="`guide-topic-${ topic.id }`"
      ref="heading"
      tabindex="-1"
    >
      {{ topic.title }}
    </h2>

    <p class="study-guide-content__summary">{{ topic.summary }}</p>

    <section
      v-for="section in topic.sections"
      :key="section.title"
    >
      <h3>{{ section.title }}</h3>

      <p v-for="paragraph in section.paragraphs" :key="paragraph">
        {{ paragraph }}
      </p>

      <ol v-if="section.steps">
        <li v-for="step in section.steps" :key="step">{{ step }}</li>
      </ol>

      <ul v-if="section.bullets">
        <li v-for="bullet in section.bullets" :key="bullet">{{ bullet }}</li>
      </ul>

      <p v-if="section.example" class="study-guide-content__example">
        <strong>Example:</strong> {{ section.example }}
      </p>

      <p v-if="section.avoid">
        <strong>Avoid:</strong> {{ section.avoid }}
      </p>
    </section>
  </article>
</template>
