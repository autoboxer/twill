<script setup>
import { computed } from 'vue';

const stateIcons = {
  empty: 'i-lucide-inbox',
  error: 'i-lucide-circle-alert',
  loading: 'i-lucide-loader-circle'
};

const props = defineProps({
  kind: {
    type: String,
    default: 'empty',
    validator: ( value ) => [ 'empty', 'error', 'loading' ].includes( value )
  },
  title: {
    type: String,
    required: true
  },
  description: {
    type: String,
    default: ''
  }
});

const icon = computed( () => stateIcons[ props.kind ]);
const isLoading = computed( () => props.kind === 'loading' );
const role = computed( () => {
  if ( props.kind === 'error' ) {
    return 'alert';
  }

  if ( props.kind === 'loading' ) {
    return 'status';
  }

  return undefined;
});

const stateUi = {
  root: 'content-state',
  header: 'content-state__header',
  title: 'content-state__title',
  description: 'content-state__description',
  body: 'content-state__body',
  actions: 'content-state__actions'
};
</script>

<template>
  <section
    :role="role"
    :aria-live="kind === 'error' ? 'assertive' : undefined"
  >
    <UEmpty
      :title="title"
      :description="description"
      :loading="isLoading"
      :ui="stateUi"
      variant="outline"
    >
      <template #leading>
        <span
          class="content-state__icon"
          :class="{ 'content-state__icon--loading': isLoading }"
          aria-hidden="true"
        >
          <UIcon :name="icon" />
        </span>
      </template>

      <template
        v-if="$slots.actions"
        #actions
      >
        <slot name="actions" />
      </template>
    </UEmpty>
  </section>
</template>
