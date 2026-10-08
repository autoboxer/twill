<script setup>
import { nextTick } from 'vue';

import { retrievalFormIcon, retrievalFormLabel } from '../retrieval-forms/catalog';
import { collectClozeGroups } from '../cloze/documents';
import { collectImageOcclusionGroups } from '../image-occlusion/documents';

const props = defineProps({
  open: { type: Boolean, default: false },
  links: { type: Array, required: true },
  selectedCase: { type: Object, default: null },
  loading: { type: Boolean, default: false },
  error: { type: String, default: '' }
});

const emit = defineEmits([ 'close', 'select', 'start', 'back', 'closed' ]);

async function restoreStudyFocus( event ) {
  event.preventDefault();
  await nextTick();

  if ( !props.open ) {
    emit( 'closed' );
  }
}

function cardLabel( card ) {
  const groups = card.cloze ? collectClozeGroups( card.content.prompt )
    : card.imageOcclusion ? collectImageOcclusionGroups( card.content.prompt ) : [];
  const groupId = card.cloze?.groupId ?? card.imageOcclusion?.groupId;
  const index = groups.findIndex( ( group ) => group.id === groupId );

  return `${ retrievalFormLabel( card ) }${ index >= 0 ? ` ${ index + 1 }` : '' }`;
}
</script>

<template>
  <UModal
    :open="open"
    title="Linked practice"
    description="Choose a case and card for extra practice. Review dates stay unchanged."
    :content="{ onCloseAutoFocus: restoreStudyFocus }"
    :ui="{ overlay: 'z-70', content: 'z-71 rounded-md' }"
    @update:open="!$event && emit('close')"
  >
    <template #body>
      <div class="linked-practice-picker" data-twill-linked-picker>
        <UAlert
          v-if="error"
          :description="error"
          color="error"
          variant="subtle"
          role="alert"
        />

        <p v-if="loading" role="status">Loading cases</p>

        <template v-if="selectedCase">
          <h3>{{ selectedCase.concept.title }}</h3>
          <p>{{ selectedCase.link.objective }}</p>

          <ul class="linked-practice-picker__list">
            <li v-for="card in selectedCase.cards" :key="card.id">
              <UButton
                :leading-icon="retrievalFormIcon( card )"
                :disabled="loading"
                :data-twill-practice-target="card.id"
                color="neutral"
                variant="subtle"
                class="linked-practice-picker__choice"
                @click="emit('start', card.id)"
              >
                {{ cardLabel( card ) }}
              </UButton>
            </li>
          </ul>

          <p v-if="!selectedCase.cards.length && !loading">This case has no available cards.</p>
        </template>

        <template v-else-if="!loading">
          <ul v-if="links.length" class="linked-practice-picker__list">
            <li v-for="link in links" :key="link.id">
              <UButton
                :disabled="link.archived"
                :data-twill-practice-link="link.id"
                color="neutral"
                variant="subtle"
                class="linked-practice-picker__choice"
                @click="emit('select', link)"
              >
                <span>
                  <strong>{{ link.title }}</strong>
                  <span>{{ link.objective }}</span>
                  <span v-if="link.archived">Archived</span>
                </span>
              </UButton>
            </li>
          </ul>

          <p v-else-if="!error">No linked cases. Add a link from this concept in Library.</p>
        </template>
      </div>
    </template>

    <template #footer>
      <div class="dialog-actions">
        <UButton
          v-if="selectedCase || error"
          :disabled="loading"
          color="neutral"
          variant="link"
          @click="emit('back')"
        >
          {{ selectedCase ? 'Choose another case' : 'Retry' }}
        </UButton>
        <UButton color="neutral" variant="subtle" @click="emit('close')">Cancel</UButton>
      </div>
    </template>
  </UModal>
</template>

<style scoped>
.linked-practice-picker,
.linked-practice-picker__list {
  display: grid;
  gap: 0.75rem;
  min-width: 0;
}

.linked-practice-picker h3 {
  font-weight: 600;
}

.linked-practice-picker p,
.linked-practice-picker__choice {
  overflow-wrap: anywhere;
}

.linked-practice-picker__choice {
  width: 100%;
  justify-content: flex-start;
  text-align: start;
}

.linked-practice-picker__choice span {
  display: grid;
  gap: 0.25rem;
  min-width: 0;
}
</style>
