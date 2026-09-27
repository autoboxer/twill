<script setup>
import { computed, onBeforeUnmount, ref, watch } from 'vue';

import { cardQualityFormName } from '../card-quality/presentation';
import { useActionNotifications } from '../composables/useActionNotifications';
import {
  conceptLibraryErrorMessage,
  useConceptLibrary
} from '../composables/useConceptLibrary';
import CardQualityDialog from './CardQualityDialog.vue';

const props = defineProps({
  conceptId: { type: String, default: '' }
});

const open = defineModel( 'open', { type: Boolean, default: false });
const { getConcept } = useConceptLibrary();
const { notifySuccess } = useActionNotifications();
const concept = ref( null );
const loading = ref( false );
const loadError = ref( '' );
let requestSequence = 0;

const cards = computed( () => concept.value?.cards.map( ( card ) => ({
  id: card.id,
  label: card.template
    ? `${ cardQualityFormName( card, concept.value.content.prompt ) } (template)`
    : cardQualityFormName( card, concept.value.content.prompt )
}) ) ?? []);

watch([ open, () => props.conceptId ], loadCards, { immediate: true, flush: 'sync' });

onBeforeUnmount( () => {
  requestSequence += 1;
});

async function loadCards() {
  const request = ++requestSequence;

  loading.value = false;

  if ( !open.value || !props.conceptId ) {
    return;
  }

  concept.value = null;
  loadError.value = '';
  loading.value = true;

  try {
    const loadedConcept = await getConcept( props.conceptId );

    if ( request !== requestSequence ) {
      return;
    }

    if ( loadedConcept.archived ) {
      loadError.value = 'This concept is archived. Restore it before reporting an issue.';
      return;
    }

    if ( !loadedConcept.cards.length ) {
      loadError.value = 'This concept has no cards. Edit it to add a card before reporting an issue.';
      return;
    }

    concept.value = loadedConcept;
  } catch ( cause ) {
    if ( request === requestSequence ) {
      loadError.value = conceptLibraryErrorMessage( cause );
    }
  } finally {
    if ( request === requestSequence ) {
      loading.value = false;
    }
  }
}
</script>

<template>
  <CardQualityDialog
    v-model:open="open"
    :cards="cards"
    :concept-title="concept?.title ?? ''"
    :loading="loading"
    :load-error="loadError"
    @retry="loadCards"
    @saved="notifySuccess( 'Card issue saved' )"
  >
    <template v-if="$slots.default" #default>
      <slot />
    </template>
  </CardQualityDialog>
</template>
