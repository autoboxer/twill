<script setup>
import { ref, watch } from 'vue';

import { conceptLibraryErrorMessage, useConceptLibrary } from '../composables/useConceptLibrary';

const props = defineProps({ revision: { type: Number, default: 0 } });
const { getStudyWorkload } = useConceptLibrary();
const open = ref( false );
const workload = ref( null );
const error = ref( '' );
const loading = ref( false );
const retry = ref( 0 );

watch([ open, () => props.revision, retry ], async ( values, previous, onCleanup ) => {
  let current = true;

  onCleanup( () => {
    current = false;
  });
  workload.value = null;
  error.value = '';
  loading.value = open.value;

  if ( !open.value ) {
    return;
  }

  try {
    const result = await getStudyWorkload();

    if ( current ) {
      workload.value = result;
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
</script>

<template>
  <details class="study-workload" @toggle="open = $event.target.open">
    <summary>Library workload</summary>
    <div v-if="open" :aria-busy="loading">
      <p v-if="loading" role="status">Loading workload…</p>
      <div v-else-if="error" role="alert">
        <p>{{ error }}</p>
        <UButton
          color="neutral"
          variant="link"
          size="sm"
          @click="retry += 1"
        >
          Try again
        </UButton>
      </div>
      <template v-else-if="workload">
        <p>
          {{ workload.due }} {{ workload.due === 1 ? 'review' : 'reviews' }} due,
          including {{ workload.overdue }} overdue by at least 24 hours.
        </p>
        <p>
          {{ workload.new }} {{ workload.new === 1 ? 'card' : 'cards' }} not yet reviewed.
          {{ workload.introduced }} introduced in the past 7 days.
        </p>
        <p>Counts cover your whole active library, not just this session.</p>
      </template>
    </div>
  </details>
</template>

<style scoped>
.study-workload {
  margin-bottom: 0.5rem;
  font-size: 0.82rem;
  color: var(--ui-text-muted);
  overflow-wrap: anywhere;
}

.study-workload summary {
  width: fit-content;
  cursor: pointer;
}

.study-workload summary:focus-visible {
  outline: 2px solid var(--ui-primary);
  outline-offset: 3px;
}

.study-workload > div {
  padding-top: 0.35rem;
}
</style>
