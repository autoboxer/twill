<script setup>
import { computed } from 'vue';

const props = defineProps({
  items: {
    type: Array,
    required: true
  },
  pendingConceptId: {
    type: String,
    default: ''
  },
  starting: {
    type: Boolean,
    default: false
  }
});

const emit = defineEmits([ 'remove', 'start' ]);

const firstItemReady = computed( () => (
  props.items[ 0 ]?.targetStatus === 'current'
) );

function targetDetails( status ) {
  return {
    current: {
      icon: 'i-lucide-pencil',
      label: 'Ready to edit'
    },
    changed: {
      icon: 'i-lucide-file-warning',
      label: 'Changed since it was queued'
    },
    archived: {
      icon: 'i-lucide-archive',
      label: 'Archived since it was queued'
    },
    missing: {
      icon: 'i-lucide-file-x-2',
      label: 'Removed since it was queued'
    }
  }[ status ] ?? {
    icon: 'i-lucide-circle-alert',
    label: 'Unavailable'
  };
}
</script>

<template>
  <section
    class="deferred-edit-queue"
    data-twill-deferred-edit-queue
  >
    <header>
      <div>
        <h3>Queued edits</h3>
        <p>Concepts are handled in the order they were queued.</p>
      </div>

      <span>{{ items.length }}</span>
    </header>

    <ol>
      <li
        v-for="item in items"
        :key="item.conceptId"
        :data-twill-deferred-status="item.targetStatus"
      >
        <UIcon
          :name="targetDetails( item.targetStatus ).icon"
          aria-hidden="true"
        />

        <div>
          <strong>{{ item.conceptTitle }}</strong>
          <span>{{ targetDetails( item.targetStatus ).label }}</span>
        </div>

        <UButton
          color="neutral"
          variant="link"
          :disabled="starting || Boolean( pendingConceptId )"
          :loading="pendingConceptId === item.conceptId"
          :aria-label="`Remove ${ item.conceptTitle } from queued edits`"
          @click="emit( 'remove', item.conceptId )"
        >
          Remove
        </UButton>
      </li>
    </ol>

    <footer>
      <p v-if="!firstItemReady">
        Remove the first unavailable item to continue.
      </p>

      <UButton
        v-else
        leading-icon="i-lucide-list-start"
        :loading="starting"
        :disabled="Boolean( pendingConceptId )"
        @click="emit( 'start' )"
      >
        Start queued edits
      </UButton>
    </footer>
  </section>
</template>
