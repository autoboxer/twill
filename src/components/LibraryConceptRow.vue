<script setup>
import { computed } from 'vue';

import { retrievalFormLabel } from '../retrieval-forms/catalog';

const props = defineProps({
  concept: { type: Object, required: true },
  navigationQuery: { type: Object, required: true },
  now: { type: Number, required: true }
});

const emit = defineEmits([ 'open' ]);

const detailLocation = computed( () => ({
  name: 'concept-detail',
  params: { conceptId: props.concept.id },
  query: { ...props.navigationQuery, card: props.concept.matchingForm?.id }
}) );

const actions = computed( () => [
  {
    label: 'Open concept',
    icon: 'i-lucide-arrow-up-right',
    to: detailLocation.value,
    onSelect: () => emit( 'open' )
  },
  {
    label: 'Edit concept',
    icon: 'i-lucide-pencil',
    to: { ...detailLocation.value, name: 'concept-edit' },
    onSelect: () => emit( 'open' )
  }
]);

const due = computed( () => {
  if ( props.concept.archived ) {
    return { label: 'Archived', description: 'Archived concepts are excluded from study' };
  }

  if ( props.concept.nextDueAt == null ) {
    return { label: '—', description: 'No scheduled cards' };
  }

  const date = new Date( props.concept.nextDueAt );
  const today = new Date( props.now );
  const tomorrow = new Date( props.now );

  tomorrow.setDate( tomorrow.getDate() + 1 );

  const isDue = props.concept.nextDueAt <= props.now;
  const label = isDue ? 'Due'
    : date.toDateString() === today.toDateString() ? 'Today'
      : date.toDateString() === tomorrow.toDateString() ? 'Tomorrow'
        : new Intl.DateTimeFormat( undefined, {
          day: 'numeric',
          month: 'short',
          ...( date.getFullYear() !== today.getFullYear() ? { year: 'numeric' } : {})
        }).format( date );

  return {
    label,
    isDue,
    description: isDue ? 'One or more cards are due' : `Next card due ${ date.toLocaleString() }`
  };
});
</script>

<template>
  <article
    class="concept-card"
    data-twill-concept-card
  >
    <RouterLink
      :id="`library-concept-${concept.id}`"
      :to="detailLocation"
      class="concept-card__link"
      @click="emit( 'open' )"
    >
      <div class="concept-card__content">
        <h3>{{ concept.title }}</h3>

        <span
          v-if="concept.matchingForm"
          class="concept-card__match"
        >
          {{ retrievalFormLabel( concept.matchingForm ) }}
        </span>
      </div>

      <span class="concept-card__count">
        {{ concept.cardCount }}
        <span class="sr-only">{{ concept.cardCount === 1 ? 'card' : 'cards' }}</span>
      </span>

      <span
        class="concept-card__due"
        :class="{ 'concept-card__due--ready': due.isDue }"
        :title="due.description"
      >
        {{ due.label }}
      </span>
    </RouterLink>

    <UDropdownMenu
      :items="actions"
      :content="{ align: 'end' }"
    >
      <UButton
        :aria-label="`Actions for ${concept.title}`"
        icon="i-lucide-ellipsis"
        color="neutral"
        variant="ghost"
        size="sm"
      />
    </UDropdownMenu>
  </article>
</template>
