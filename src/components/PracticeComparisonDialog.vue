<script setup>
import { invoke } from '@tauri-apps/api/core';
import { computed, onBeforeUnmount, ref, watch } from 'vue';

import { conceptLibraryErrorMessage } from '../composables/useConceptLibrary';
import PracticeComparison from './PracticeComparison.vue';

const props = defineProps({
  conceptId: { type: String, required: true },
  link: { type: Object, default: null }
});
const emit = defineEmits([ 'close' ]);

const activity = ref( null );
const loading = ref( false );
const error = ref( '' );
let sequence = 0;

const open = computed({
  get: () => Boolean( props.link ),
  set: ( value ) => {
    if ( !value ) {
      emit( 'close' );
    }
  }
});

watch([ () => props.link, () => props.conceptId ], load, { flush: 'sync' });
onBeforeUnmount( () => {
  sequence += 1;
});

async function load() {
  const token = ++sequence;

  activity.value = null;
  error.value = '';
  loading.value = Boolean( props.link );

  if ( !props.link ) {
    return;
  }

  try {
    const result = await invoke( 'get_practice_comparison', {
      input: { conceptId: props.conceptId, linkId: props.link.id }
    });

    if ( token === sequence ) {
      activity.value = result;
    }
  } catch ( cause ) {
    if ( token === sequence ) {
      error.value = conceptLibraryErrorMessage( cause );
    }
  } finally {
    if ( token === sequence ) {
      loading.value = false;
    }
  }
}
</script>

<template>
  <UModal
    v-model:open="open"
    title="Compare cases"
    description="Compare the feature that changes which method fits."
    :ui="{ content: 'rounded-md sm:max-w-3xl', wrapper: 'pr-10' }"
  >
    <template #body>
      <p v-if="loading" role="status">Loading comparison</p>
      <div v-else-if="error" role="alert">
        <p>{{ error }}</p>
        <UButton color="neutral" variant="link" @click="load">Retry</UButton>
      </div>
      <PracticeComparison v-else-if="activity" :activity="activity" />
    </template>

    <template #footer>
      <div class="dialog-actions">
        <UButton color="neutral" variant="subtle" @click="open = false">Close</UButton>
      </div>
    </template>
  </UModal>
</template>
