<script setup>
import { computed, ref, watch } from 'vue';

import { cardQualityFormName } from '../card-quality/presentation';
import CardQualityDialog from './CardQualityDialog.vue';

const props = defineProps({
  card: { type: Object, required: true },
  disabled: { type: Boolean, default: false }
});

const open = ref( false );
const saved = ref( false );
const cards = computed( () => [{
  id: props.card.id,
  label: cardQualityFormName( props.card, props.card.content.prompt )
}]);

watch( () => props.card.id, () => {
  open.value = false;
  saved.value = false;
}, { flush: 'sync' });
</script>

<template>
  <CardQualityDialog
    v-model:open="open"
    :cards="cards"
    :concept-title="card.conceptTitle"
    @saved="saved = true"
  >
    <UButton
      :leading-icon="saved ? 'i-lucide-check' : 'i-lucide-flag'"
      color="neutral"
      variant="link"
      size="sm"
      class="study-card-issue"
      :disabled="disabled"
      :title="saved ? 'Issue saved. Report another card issue.' : undefined"
    >
      Card issue
    </UButton>
  </CardQualityDialog>

  <span role="status" class="sr-only">
    {{ saved ? 'Card issue saved.' : '' }}
  </span>
</template>
