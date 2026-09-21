<script setup>
import { nextTick, onMounted, ref, watch } from 'vue';

import {
  cardQualityFormName,
  cardQualityItemKey,
  cardQualityLabel
} from '../card-quality/presentation';
import CardQualityPreview from '../components/CardQualityPreview.vue';
import ContentState from '../components/ContentState.vue';
import PageHeader from '../components/PageHeader.vue';
import { useStartupReady } from '../composables/useStartupReady';
import { useCardQualityQueue } from '../composables/useCardQualityQueue';

const {
  actionError,
  actionsDisabled,
  finishItem,
  groups,
  loadError,
  loading,
  notice,
  pendingKey,
  refresh
} = useCardQualityQueue();
const inspectedCardId = ref( '' );
const errorContainer = ref( null );

watch([ loadError, actionError ], async () => {
  await nextTick();
  errorContainer.value?.focus();
});

watch( groups, ( currentGroups ) => {
  if ( !currentGroups.some( ( group ) => group.card.cardId === inspectedCardId.value ) ) {
    inspectedCardId.value = '';
  }
});

onMounted( refresh );

function formattedDate( timestamp ) {
  return new Intl.DateTimeFormat( undefined, {
    dateStyle: 'medium',
    timeStyle: 'short'
  }).format( new Date( timestamp ) );
}

useStartupReady( loading );
</script>

<template>
  <div class="page quality-page" data-twill-page="card-quality">
    <PageHeader title="Card quality">
      <template #leading>
        <UButton
          :to="{ name: 'library' }"
          leading-icon="i-lucide-arrow-left"
          color="neutral"
          variant="link"
        >
          Library
        </UButton>
      </template>

      <template #actions>
        <UButton
          leading-icon="i-lucide-refresh-cw"
          color="neutral"
          variant="subtle"
          :loading="loading"
          :disabled="Boolean( pendingKey )"
          @click="refresh"
        >
          Refresh
        </UButton>
      </template>
    </PageHeader>

    <p class="quality-page__introduction">
      Review reported issues and possible trouble patterns. Resolve issues you have addressed,
      or dismiss suggestions that are not useful. Neither action changes the card or its schedule.
    </p>

    <p v-if="notice" class="quality-page__notice" role="status">{{ notice }}</p>

    <div
      v-if="loadError || actionError"
      ref="errorContainer"
      tabindex="-1"
    >
      <UAlert
        :title="loadError ? 'Queue could not be refreshed' : 'Action could not be completed'"
        :description="loadError || actionError"
        icon="i-lucide-circle-alert"
        color="error"
        variant="subtle"
      />
    </div>

    <ContentState
      v-if="loading && !groups.length"
      kind="loading"
      title="Loading card quality"
    />

    <ContentState
      v-else-if="!groups.length && !loadError"
      title="No cards to improve"
      description="Use Card issue during study to report a concern. Review-history suggestions appear here when a repeated difficulty pattern is present."
    />

    <div v-else-if="groups.length" class="quality-queue" :aria-busy="loading">
      <p class="quality-queue__count">
        {{ groups.length }} {{ groups.length === 1 ? 'card' : 'cards' }} to review
      </p>

      <section
        v-for="group in groups"
        :key="group.card.cardId"
        class="quality-card"
        :data-card-id="group.card.cardId"
        :aria-labelledby="`quality-title-${ group.card.cardId }`"
      >
        <header class="quality-card__header">
          <div class="quality-card__title">
            <h2 :id="`quality-title-${ group.card.cardId }`">{{ group.card.conceptTitle }}</h2>
            <p>
              {{ cardQualityFormName( group.card ) }}
              · {{ group.items.length }} {{ group.items.length === 1 ? 'item' : 'items' }}
            </p>
          </div>

          <div class="quality-card__actions">
            <UButton
              :aria-expanded="inspectedCardId === group.card.cardId"
              :aria-controls="`quality-preview-${ group.card.cardId }`"
              color="neutral"
              variant="subtle"
              @click="inspectedCardId = inspectedCardId === group.card.cardId ? '' : group.card.cardId"
            >
              {{ inspectedCardId === group.card.cardId ? 'Hide card' : 'Inspect card' }}
            </UButton>

            <UButton
              :to="{ name: 'concept-edit', params: { conceptId: group.card.conceptId } }"
              leading-icon="i-lucide-pencil"
              color="neutral"
              variant="link"
              :disabled="Boolean( pendingKey )"
            >
              Edit concept
            </UButton>
          </div>
        </header>

        <div :id="`quality-preview-${ group.card.cardId }`">
          <CardQualityPreview
            v-if="inspectedCardId === group.card.cardId"
            :item="group.card"
          />
        </div>

        <ul class="quality-card__items">
          <li
            v-for="item in group.items"
            :key="cardQualityItemKey( item )"
            class="quality-item"
            :data-quality-item="cardQualityItemKey( item )"
          >
            <div class="quality-item__heading">
              <h3>{{ cardQualityLabel( item ) }}</h3>
              <span>{{ item.source === 'manual' ? 'Reported by you' : 'Review-history suggestion' }}</span>
            </div>

            <p v-if="item.note" class="quality-item__note">{{ item.note }}</p>

            <div v-if="item.evidence" class="quality-item__evidence">
              <p>
                {{ item.evidence.againCount }} of the latest {{ item.evidence.reviewsConsidered }}
                reviews were Forgot / Again. Undone reviews are excluded.
              </p>
              <p>
                This may reflect difficult material, not a problem with the card.
                Dismissing waits for three new Forgot / Again outcomes before suggesting it again.
              </p>
            </div>

            <footer class="quality-item__footer">
              <time :datetime="new Date( item.noticedAt ).toISOString()">
                {{ formattedDate( item.noticedAt ) }}
              </time>

              <div class="quality-card__actions">
                <UButton
                  v-if="item.source === 'manual'"
                  leading-icon="i-lucide-check"
                  variant="subtle"
                  :disabled="actionsDisabled"
                  @click="finishItem( item, 'resolved' )"
                >
                  Resolve
                </UButton>

                <UButton
                  color="neutral"
                  variant="link"
                  :disabled="actionsDisabled"
                  @click="finishItem( item, 'dismissed' )"
                >
                  Dismiss
                </UButton>

                <span v-if="pendingKey === cardQualityItemKey( item )" role="status">
                  Saving…
                </span>
              </div>
            </footer>
          </li>
        </ul>
      </section>
    </div>
  </div>
</template>

<style scoped>
.quality-page {
  display: grid;
  align-content: start;
  gap: 1.5rem;
}

.quality-page__introduction {
  max-width: 65ch;
  color: var(--ui-text-muted);
  line-height: 1.65;
}

.quality-page__notice {
  color: var(--ui-text-success);
}

.quality-queue {
  display: grid;
  gap: 1rem;
}

.quality-queue__count,
.quality-item__heading > span,
.quality-item__footer,
.quality-card__title p {
  color: var(--ui-text-muted);
  font-size: 0.8rem;
}

.quality-card {
  min-width: 0;
  border: 1px solid var(--ui-border);
  border-radius: 0.375rem;
  background: var(--ui-bg-muted);
  overflow-wrap: anywhere;
}

.quality-card__header,
.quality-item__footer,
.quality-card__actions,
.quality-item__heading {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.75rem;
}

.quality-card__header,
.quality-item__footer {
  justify-content: space-between;
}

.quality-card__header,
.quality-item {
  padding: 1.25rem;
}

.quality-card__title {
  flex: 1 1 15rem;
  min-width: 0;
}

.quality-card__title h2 {
  font-size: 1rem;
  font-weight: 600;
}

.quality-card__title p {
  margin-top: 0.3rem;
}

.quality-item {
  display: grid;
  gap: 0.75rem;
  border-top: 1px solid var(--ui-border);
}

.quality-item__heading h3 {
  font-size: 0.9rem;
  font-weight: 600;
}

.quality-item__note {
  white-space: pre-wrap;
}

.quality-item__evidence {
  display: grid;
  gap: 0.5rem;
  font-size: 0.9rem;
  line-height: 1.65;
}

.quality-item__evidence p + p {
  color: var(--ui-text-muted);
}
</style>
