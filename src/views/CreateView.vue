<script setup>
import { computed, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';

import ConceptForm from '../components/ConceptForm.vue';
import ContentState from '../components/ContentState.vue';
import OrganizationManager from '../components/OrganizationManager.vue';
import PageHeader from '../components/PageHeader.vue';
import { COMMAND_IDS } from '../commands/registry';
import { useCommandHandler } from '../composables/useCommands';
import {
  conceptLibraryErrorMessage,
  useConceptLibrary
} from '../composables/useConceptLibrary';
import { useTemplateLibrary } from '../composables/useTemplateLibrary';

const route = useRoute();
const router = useRouter();
const {
  clearError,
  createConcept,
  error,
  getLibrary,
  isPending,
  updateConcept
} = useConceptLibrary();
const {
  clearError: clearLoadError,
  getConcept: loadConcept,
  getLibrary: loadLibrary
} = useConceptLibrary();
const {
  clearError: clearTemplateLoadError,
  getTemplates
} = useTemplateLibrary();

const concept = ref( null );
const conceptForm = ref( null );
const initialLoading = ref( true );
const loadError = ref( '' );
const library = ref({
  archivedCount: 0,
  concepts: [],
  decks: [],
  tags: []
});
const organizationManagerOpen = ref( false );
const templates = ref([]);
let loadRequestSequence = 0;

const conceptId = computed( () => route.params.conceptId ?? '' );
const isEditing = computed( () => Boolean( conceptId.value ) );
const pageTitle = computed( () => isEditing.value ? 'Edit concept' : 'Create concept' );
const saveCommand = useCommandHandler( COMMAND_IDS.conceptSave, {
  enabled: computed( () => !initialLoading.value && !loadError.value && !isPending.value ),
  execute: () => conceptForm.value?.submit()
});

watch( conceptId, loadData, { immediate: true });

async function loadData() {
  const request = ++loadRequestSequence;
  const requestedConceptId = conceptId.value;

  clearError();
  clearLoadError();
  clearTemplateLoadError();
  initialLoading.value = true;
  loadError.value = '';

  try {
    const [ snapshot, existingConcept, templateCatalog ] = await Promise.all([
      loadLibrary( false ),
      requestedConceptId ? loadConcept( requestedConceptId ) : Promise.resolve( null ),
      getTemplates()
    ]);

    if ( request !== loadRequestSequence ) {
      return;
    }

    library.value = snapshot;
    concept.value = existingConcept;
    templates.value = templateCatalog.templates;
  } catch ( cause ) {
    if ( request === loadRequestSequence ) {
      loadError.value = conceptLibraryErrorMessage( cause );
    }
  } finally {
    if ( request === loadRequestSequence ) {
      initialLoading.value = false;
    }
  }
}

async function refreshOrganizations() {
  try {
    library.value = await getLibrary( false );
  } catch {
    // Error state is handled by the composable.
  }
}

async function saveConcept( input ) {
  clearError();

  try {
    const saved = isEditing.value
      ? await updateConcept({ id: conceptId.value, ...input })
      : await createConcept( input );

    await router.replace({
      name: 'concept-detail',
      params: { conceptId: saved.id }
    });
  } catch {
    // Error state is handled by the composable.
  }
}

function cancel() {
  if ( isEditing.value ) {
    router.push({
      name: 'concept-detail',
      params: { conceptId: conceptId.value }
    });
    return;
  }

  router.push({ name: 'library' });
}
</script>

<template>
  <div
    class="page editor-page"
    data-twill-page="concept-editor"
  >
    <PageHeader :title="pageTitle">
      <template #actions>
        <UButton
          leading-icon="i-lucide-arrow-left"
          color="neutral"
          variant="link"
          @click="cancel"
        >
          Back
        </UButton>
      </template>
    </PageHeader>

    <ContentState
      v-if="initialLoading"
      kind="loading"
      title="Loading concept editor"
    />

    <ContentState
      v-else-if="loadError"
      kind="error"
      :title="isEditing ? 'Concept could not be loaded' : 'Editor could not be loaded'"
      :description="loadError"
    >
      <template #actions>
        <UButton
          leading-icon="i-lucide-refresh-cw"
          @click="loadData"
        >
          Retry
        </UButton>
      </template>
    </ContentState>

    <ConceptForm
      v-else
      ref="conceptForm"
      :mode="isEditing ? 'edit' : 'create'"
      :concept="concept"
      :decks="library.decks"
      :tags="library.tags"
      :templates="templates"
      :error="error"
      :loading="isPending"
      :save-command="saveCommand"
      @cancel="cancel"
      @manage="organizationManagerOpen = true"
      @submit="saveConcept"
    />

    <OrganizationManager
      v-model:open="organizationManagerOpen"
      :decks="library.decks"
      :tags="library.tags"
      @changed="refreshOrganizations"
    />
  </div>
</template>
