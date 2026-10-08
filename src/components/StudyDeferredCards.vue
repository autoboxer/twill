<script setup>
import { ref } from 'vue';

import { retrievalFormLabel } from '../retrieval-forms/catalog';

defineProps({
  items: { type: Array, required: true }
});

const open = ref( false );
</script>

<template>
  <UPopover v-model:open="open" :ui="{ content: 'z-50' }">
    <UButton
      color="neutral"
      variant="link"
      size="sm"
      data-twill-deferred-related
    >
      {{ items.length }} deferred
    </UButton>

    <template #content>
      <section class="study-deferred-cards" aria-label="Deferred cards">
        <h3>Deferred cards</h3>
        <p>Related answers may have cued these cards. They are still due. Start a new session to review them.</p>

        <ul>
          <li v-for="item in items" :key="item.card.id">
            <RouterLink :to="{ name: 'concept-detail', params: { conceptId: item.card.conceptId } }" @click="open = false">
              {{ item.card.conceptTitle }}
            </RouterLink>
            <span>{{ retrievalFormLabel( item.card ) }} · {{ item.reason === 'shared' ? 'Shared content' : 'Linked case' }}</span>
          </li>
        </ul>
      </section>
    </template>
  </UPopover>
</template>

<style scoped>
.study-deferred-cards {
  width: min(22rem, calc(100vw - 2rem));
  padding: 1rem;
}

.study-deferred-cards h3 {
  font-weight: 600;
}

.study-deferred-cards p {
  margin-top: 0.5rem;
  color: var(--ui-text-muted);
  font-size: 0.875rem;
}

.study-deferred-cards ul {
  max-height: min(18rem, 45vh);
  margin-top: 0.75rem;
  overflow-y: auto;
}

.study-deferred-cards li {
  display: grid;
  gap: 0.125rem;
  padding-block: 0.5rem;
  border-top: 1px solid var(--ui-border);
  overflow-wrap: anywhere;
}

.study-deferred-cards a {
  color: var(--ui-text-highlighted);
}

.study-deferred-cards a:hover {
  text-decoration: underline;
}

.study-deferred-cards span {
  color: var(--ui-text-muted);
  font-size: 0.75rem;
}
</style>
