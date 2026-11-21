<script setup>
import { nextTick, ref, useId } from 'vue';

const props = defineProps({
  followUp: { type: Object, required: true },
  state: { type: Object, required: true },
  disabled: { type: Boolean, default: false }
});

const emit = defineEmits([ 'toggle', 'response', 'toggle-reference' ]);
const root = ref( null );
const reference = ref( null );
const id = useId();

async function toggle() {
  emit( 'toggle' );
  await nextTick();

  if ( props.state.followUpOpen ) {
    root.value?.querySelector( 'textarea' )?.focus();
  }
}

async function toggleReference() {
  emit( 'toggle-reference' );
  await nextTick();

  if ( props.state.followUpRevealed ) {
    reference.value?.focus();
  }
}
</script>

<template>
  <section ref="root" class="example-follow-up" data-twill-example-follow-up>
    <UButton
      type="button"
      color="neutral"
      variant="link"
      size="sm"
      :disabled="disabled"
      :aria-expanded="state.followUpOpen"
      :aria-controls="id"
      @click="toggle"
    >
      {{ state.followUpOpen ? 'Hide follow-up' : 'Follow-up' }}
    </UButton>

    <div v-if="state.followUpOpen" :id="id" class="example-follow-up__content">
      <p class="example-follow-up__text">{{ followUp.prompt }}</p>
      <UFormField label="Your follow-up response">
        <UTextarea
          :model-value="state.followUpResponse"
          placeholder="Write a response, or answer mentally"
          :rows="3"
          :maxrows="8"
          autoresize
          class="w-full"
          :disabled="disabled"
          :readonly="state.followUpRevealed"
          @update:model-value="emit( 'response', $event )"
        />
      </UFormField>
      <UButton
        type="button"
        color="neutral"
        variant="subtle"
        size="sm"
        :disabled="disabled"
        :aria-expanded="state.followUpRevealed"
        :aria-controls="`${ id }-reference`"
        @click="toggleReference"
      >
        {{ state.followUpRevealed ? 'Hide follow-up reference' : 'Compare follow-up' }}
      </UButton>
      <div
        v-if="state.followUpRevealed"
        :id="`${ id }-reference`"
        ref="reference"
        role="region"
        aria-label="Follow-up reference"
        tabindex="-1"
      >
        <strong>Follow-up reference</strong>
        <p class="example-follow-up__text">{{ followUp.reference }}</p>
      </div>
      <p class="study-comparison-guidance">
        Grade the original attempt, not this follow-up. Your writing is kept only for this session.
      </p>
    </div>
  </section>
</template>

<style scoped>
.example-follow-up__content {
  display: grid;
  gap: 0.75rem;
  margin-top: 0.5rem;
  padding-left: 0.75rem;
  border-left: 2px solid var(--ui-border-accented);
  min-width: 0;
}

.example-follow-up__content > button {
  justify-self: start;
}

.example-follow-up__text {
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}
</style>
