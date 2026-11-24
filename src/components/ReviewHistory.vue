<script setup>
import { computed, ref, watch } from 'vue';

import { conceptLibraryErrorMessage, useConceptLibrary } from '../composables/useConceptLibrary';

const props = defineProps({
  cardId: { type: String, required: true },
  conceptVersion: { type: String, required: true },
  improvable: { type: Boolean, default: true }
});
const { getReviewHistory } = useConceptLibrary();
const open = ref( false );
const history = ref( null );
const loading = ref( false );
const error = ref( '' );
const retry = ref( 0 );
const emit = defineEmits([ 'improve' ]);
const dateFormat = new Intl.DateTimeFormat( undefined, { dateStyle: 'medium', timeStyle: 'short' });
const ratings = {
  again: 'Forgot',
  hard: 'Remembered with difficulty',
  good: 'Remembered',
  easy: 'Remembered easily'
};
const evidenceLabels = { first: 'First review of this version', recent: 'Recent practice', delayed: 'Delayed review' };
const otherReviews = computed( () => {
  const evidence = history.value?.evidence;

  return evidence ? [
    [ evidence.first, 'first' ],
    [ evidence.recent, 'recent' ],
    [ evidence.helped, 'with help' ],
    [ evidence.unknown, 'with incomplete context' ]
  ].filter( ([ count ]) => count > 0 ).map( ([ count, label ]) => `${ count } ${ label }` ).join( ', ' ) : '';
});

watch([ open, () => props.cardId, () => props.conceptVersion, retry ], async ( values, previous, onCleanup ) => {
  let current = true;

  onCleanup( () => {
    current = false;
  });
  history.value = null;
  error.value = '';
  loading.value = open.value;

  if ( !open.value ) {
    return;
  }

  try {
    const result = await getReviewHistory( props.cardId );

    if ( current ) {
      history.value = result;
    }
  } catch ( cause ) {
    if ( current ) {
      error.value = conceptLibraryErrorMessage( cause );
    }
  } finally {
    if ( current ) {
      loading.value = false;
    }
  }
});

function assistanceLabel( assisted ) {
  if ( assisted === null ) {
    return 'Help use not recorded';
  }

  return assisted ? 'Help used' : 'No help opened';
}
</script>

<template>
  <details class="review-history" @toggle="open = $event.target.open">
    <summary>Recent reviews</summary>

    <div v-if="open" :aria-busy="loading">
      <p v-if="loading" role="status">Loading reviews…</p>

      <div v-else-if="error" role="alert">
        <p>{{ error }}</p>
        <UButton
          color="neutral"
          variant="subtle"
          size="sm"
          @click="retry += 1"
        >
          Try again
        </UButton>
      </div>

      <template v-else-if="history">
        <p v-if="!history.reviews.length">No scheduled reviews yet.</p>
        <template v-else>
          <section class="review-history__evidence" aria-label="Learning evidence">
            <p v-if="history.evidence.delayedTotal">
              Remembered in {{ history.evidence.delayedRemembered }} of {{ history.evidence.delayedTotal }}
              delayed {{ history.evidence.delayedTotal === 1 ? 'review' : 'reviews' }}.
            </p>
            <p v-else>No delayed reviews recorded for this version in the past 30 days.</p>
            <p>This summary covers the past 30 days and the current card version.</p>
            <p v-if="otherReviews">Other reviews include {{ otherReviews }}.</p>
            <details class="review-history__method">
              <summary>How this is counted</summary>
              <p>
                Delayed reviews follow an earlier review and at least 24 hours without
                recorded exposure to this concept or a linked case. Helped and undone reviews are excluded.
                These counts describe recorded practice, not permanent mastery.
              </p>
            </details>
            <UButton
              v-if="improvable && history.evidence.delayedTotal > history.evidence.delayedRemembered"
              color="neutral"
              variant="link"
              size="sm"
              @click="emit( 'improve' )"
            >
              Needs improvement
            </UButton>
          </section>
          <p v-if="history.hasMore">Showing the 10 most recent grades, including any you undid.</p>
          <ul class="review-history__list" aria-label="Recent reviews">
            <li v-for="review in history.reviews" :key="review.id">
              <div class="review-history__heading">
                <span class="review-history__rating">{{ ratings[ review.rating ] }}</span>
                <time :datetime="new Date( review.reviewedAt ).toISOString()">
                  {{ dateFormat.format( review.reviewedAt ) }}
                </time>
              </div>
              <p>{{ assistanceLabel( review.assisted ) }}</p>
              <p v-if="review.uncertaintyMarked">Marked Not sure before reveal</p>
              <p v-if="evidenceLabels[ review.evidenceKind ]">{{ evidenceLabels[ review.evidenceKind ] }}</p>
              <p v-if="review.currentContent === false">Earlier version of this card</p>
              <p v-else-if="review.currentContent === null">Card version not recorded</p>
              <p v-if="review.reversedAt !== null" class="review-history__undone">
                Undone. This grade does not count toward your schedule or review total.
              </p>
            </li>
          </ul>
        </template>
      </template>
    </div>
  </details>
</template>

<style scoped>
.review-history {
  margin-top: 0.5rem;
  font-size: 0.82rem;
  overflow-wrap: anywhere;
}

.review-history summary {
  width: fit-content;
  cursor: pointer;
}

.review-history summary:focus-visible {
  outline: 2px solid var(--ui-primary);
  outline-offset: 3px;
}

.review-history > div {
  margin-top: 0.5rem;
}

.review-history__list {
  padding: 0;
  margin: 0;
  list-style: none;
}

.review-history__evidence {
  margin-bottom: 0.75rem;
}

.review-history__evidence p + p {
  margin-top: 0.35rem;
  color: var(--ui-text-muted);
}

.review-history__list > li {
  padding: 0.5rem 0;
  border-top: 1px solid var(--ui-border);
}

.review-history__heading {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  justify-content: space-between;
  gap: 0.2rem 0.75rem;
}

.review-history__rating {
  color: var(--ui-text-highlighted);
  font-weight: 600;
}

.review-history time,
.review-history__list p {
  color: var(--ui-text-muted);
}

.review-history__undone {
  margin-top: 0.25rem;
}
</style>
