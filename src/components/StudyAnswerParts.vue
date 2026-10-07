<script setup>
import { computed } from 'vue';

import { answerPartRevealGroups, answerPartSections } from '../answer-parts/documents';
import { richDocumentHasContent } from '../rich-content/schema';
import RichContentRenderer from './RichContentRenderer.vue';
import StudyIdeaCheck from './StudyIdeaCheck.vue';

const props = defineProps({
  document: {
    type: Object,
    required: true
  },
  fullAnswerRevealed: {
    type: Boolean,
    required: true
  },
  revealed: {
    type: Array,
    default: () => []
  },
  disabled: {
    type: Boolean,
    default: false
  },
  comparisonEnabled: {
    type: Boolean,
    default: false
  },
  writtenResponse: {
    type: Boolean,
    default: false
  },
  checks: {
    type: Object,
    default: () => ({})
  }
});

const emit = defineEmits([ 'toggle', 'compare' ]);
const groups = computed( () => answerPartRevealGroups( props.document ) );
const sections = computed( () => answerPartSections( props.document ) );

function revealGroup( part ) {
  return groups.value.find( ( group ) => group.parts.some( ( item ) => item.id === part.id ) );
}

function showsGroupControl( part ) {
  const members = revealGroup( part ).parts;

  return members.length > 1 && members[ 0 ].id === part.id;
}

function groupAction( part ) {
  return groupRevealed( part ) ? 'Hide' : 'Reveal';
}

function groupControlLabel( part ) {
  const group = revealGroup( part );

  if ( group.default ) {
    return `${ groupAction( part ) } all parts`;
  }

  if ( group.parts.length > 3 ) {
    return `${ groupAction( part ) } ${ group.parts.length } parts`;
  }

  const numbers = group.parts.map( ( item ) => item.label.replace( 'Part ', '' ) );
  const last = numbers.pop();

  return `${ groupAction( part ) } parts ${ numbers.join( ', ' ) } and ${ last }`;
}

function groupAccessibleLabel( part ) {
  const group = revealGroup( part );

  if ( group.default || group.parts.length <= 3 ) {
    return groupControlLabel( part );
  }

  const labels = group.parts.map( ( item ) => item.label.toLowerCase() ).join( ', ' );

  return `${ groupControlLabel( part ) }: ${ labels }`;
}

function groupRevealed( part ) {
  return revealGroup( part ).parts.every( ( item ) => props.revealed.includes( item.id ) );
}

function startsGroup( index ) {
  const current = sections.value[ index ];

  for ( let previousIndex = index - 1; previousIndex >= 0; previousIndex-- ) {
    const previous = sections.value[ previousIndex ];

    if ( previous.id ) {
      return revealGroup( previous ).id !== revealGroup( current ).id;
    }
  }

  return false;
}
</script>

<template>
  <div
    class="study-answer-parts"
    data-twill-answer-parts
  >
    <template v-for="( section, index ) in sections" :key="section.key">
      <section
        v-if="section.id"
        class="study-answer-part"
        :class="{ 'study-answer-part--group-start': startsGroup( index ) }"
        :data-answer-part-id="section.id"
        :data-answer-group-id="revealGroup( section ).id"
      >
        <div class="study-answer-part__heading">
          <span>{{ section.label }}</span>
          <span v-if="groups.length > 1" class="study-answer-part__group-label">
            Group {{ groups.indexOf( revealGroup( section ) ) + 1 }}
          </span>
          <template v-if="!fullAnswerRevealed">
            <UButton
              type="button"
              color="neutral"
              variant="link"
              size="sm"
              :aria-expanded="revealed.includes( section.id )"
              :aria-controls="`answer-part-${ section.id }`"
              :aria-label="`${ revealed.includes( section.id ) ? 'Hide' : 'Reveal' } ${ section.label.toLowerCase() }`"
              :disabled="disabled"
              @click="emit( 'toggle', [ section.id ] )"
            >
              {{ revealed.includes( section.id ) ? 'Hide' : 'Reveal' }}
            </UButton>
            <UButton
              v-if="showsGroupControl( section )"
              type="button"
              color="neutral"
              variant="link"
              size="sm"
              :aria-label="groupAccessibleLabel( section )"
              :aria-expanded="groupRevealed( section )"
              :disabled="disabled"
              @click="emit( 'toggle', revealGroup( section ).parts.map( ( item ) => item.id ) )"
            >
              {{ groupControlLabel( section ) }}
            </UButton>
          </template>
          <StudyIdeaCheck
            v-else-if="comparisonEnabled"
            :label="section.label.toLowerCase()"
            :model-value="checks[ `part:${ section.id }` ]"
            :written-response="writtenResponse"
            :disabled="disabled"
            @update:model-value="emit( 'compare', `part:${ section.id }`, $event )"
          />
        </div>
        <div
          v-if="fullAnswerRevealed || revealed.includes( section.id )"
          :id="`answer-part-${ section.id }`"
        >
          <RichContentRenderer
            :document="section.document"
            :label="section.label"
          />
        </div>
      </section>
      <RichContentRenderer
        v-else-if="fullAnswerRevealed && richDocumentHasContent( section.document )"
        :document="section.document"
        label="Answer"
      />
    </template>
  </div>
</template>
