<script setup>
import { computed, nextTick, ref, useId } from 'vue';
import StudyIdeaList from './StudyIdeaList.vue';
import { explainFocusDetails } from '../explain/focus';

const props = defineProps({
  modelValue: {
    type: String,
    required: true
  },

  revealed: {
    type: Boolean,
    required: true
  },

  checks: {
    type: Object,
    default: () => ({})
  },

  disabled: {
    type: Boolean,
    default: false
  },

  review: {
    type: Object,
    default: null
  },

  settings: {
    type: Object,
    required: true
  }
});

const emit = defineEmits([ 'update:modelValue', 'compare', 'toggle-review', 'correction' ]);

const result = ref( null );
const root = ref( null );
const reviewId = useId();

const hasResponse = computed( () => Boolean( props.modelValue.trim() ) );
const focusDetails = computed( () => explainFocusDetails( props.settings.focus ) );
const gaps = computed( () => props.settings.keyPoints
  .map( ( text, index ) => ({ text, index }) )
  .filter( ({ index }) => [ 'missing', 'partial' ].includes( props.checks[ `idea:${ index }` ]) )
);

async function toggleReview() {
  emit( 'toggle-review' );
  await nextTick();

  if ( props.review?.open ) {
    root.value?.querySelector( '.reconstruction-review textarea' )?.focus();
  }
}

defineExpose({ focus });

function focus() {
  if ( props.revealed ) {
    result.value?.focus();
    return;
  }

  root.value?.querySelector( 'textarea' )?.focus();
}
</script>

<template>
  <section
    ref="root"
    class="explain-response"
    :data-twill-reconstruction="focusDetails.reconstruction || undefined"
  >
    <header class="explain-response__heading">
      <span aria-hidden="true">
        <UIcon name="i-lucide-message-square-text" />
      </span>

      <div>
        <strong>{{ focusDetails.instruction }}</strong>
      </div>
    </header>

    <UFormField
      v-if="!revealed"
      :label="focusDetails.inputLabel"
    >
      <UTextarea
        :model-value="modelValue"
        :placeholder="focusDetails.placeholder"
        :rows="5"
        autoresize
        :maxrows="10"
        class="explain-response__input"
        :disabled="disabled"
        @update:model-value="emit( 'update:modelValue', $event )"
      />
    </UFormField>

    <div
      v-else
      ref="result"
      class="explain-comparison"
      role="group"
      :aria-label="focusDetails.comparison"
      tabindex="-1"
    >
      <div class="explain-comparison__heading">
        <strong>{{ focusDetails.comparison }}</strong>
      </div>

      <div
        class="explain-comparison__content"
        :class="{ 'explain-comparison__content--written': hasResponse }"
      >
        <div v-if="hasResponse">
          <span>{{ focusDetails.response }}</span>

          <p class="explain-comparison__attempt">
            {{ modelValue }}
          </p>
        </div>

        <div>
          <span>{{ focusDetails.criteria }}</span>

          <StudyIdeaList
            :items="settings.keyPoints"
            kind="idea"
            :ordered="settings.focus === 'sequence'"
            :item-label="focusDetails.reconstruction ? focusDetails.criterion.toLowerCase() : ''"
            :checks="checks"
            :written-response="hasResponse"
            :disabled="disabled"
            @compare="( id, value ) => emit( 'compare', id, value )"
          />
        </div>
      </div>

      <template v-if="focusDetails.reconstruction && review">
        <div class="reconstruction-review__actions">
          <UButton
            color="neutral"
            variant="link"
            size="sm"
            :aria-expanded="review.open"
            :aria-controls="reviewId"
            :disabled="disabled"
            @click="toggleReview"
          >
            {{ review.open ? 'Hide correction' : review.correction.trim() ? 'Show correction' : 'Add correction' }}
          </UButton>
        </div>

        <div
          v-if="review.open"
          :id="reviewId"
          class="reconstruction-review"
        >
          <div v-if="gaps.length" class="reconstruction-review__gaps">
            <strong>Revisit</strong>
            <component :is="settings.focus === 'sequence' ? 'ol' : 'ul'">
              <li
                v-for="gap in gaps"
                :key="gap.index"
                :value="settings.focus === 'sequence' ? gap.index + 1 : undefined"
              >
                {{ gap.text }}
              </li>
            </component>
          </div>

          <UFormField label="Your correction">
            <UTextarea
              :model-value="review.correction"
              placeholder="Write what you would change"
              :rows="3"
              :maxrows="10"
              autoresize
              class="explain-response__input"
              :disabled="disabled"
              :aria-describedby="`${ reviewId }-guidance`"
              @update:model-value="emit( 'correction', $event )"
            />
          </UFormField>

          <p :id="`${ reviewId }-guidance`" class="study-comparison-guidance">
            Check and grade your original attempt, not this correction.
          </p>
        </div>
      </template>
    </div>
  </section>
</template>
