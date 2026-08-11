<script setup>
import { AnimatePresence, m } from 'motion-v';
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';

import ConfirmDialog from '../components/ConfirmDialog.vue';
import ContentState from '../components/ContentState.vue';
import PageHeader from '../components/PageHeader.vue';
import { conceptLibraryErrorMessage } from '../composables/useConceptLibrary';
import { useTemplateLibrary } from '../composables/useTemplateLibrary';

const {
  clearError,
  deleteTemplate,
  error,
  getTemplates,
  isPending
} = useTemplateLibrary();

const catalog = ref({ templates: [] });
const deleteTarget = ref( null );
const initialLoading = ref( true );
const loadError = ref( '' );
let loadRequestSequence = 0;

const deleteDialogOpen = computed({
  get: () => Boolean( deleteTarget.value ),
  set: ( value ) => {
    if ( !value ) {
      deleteTarget.value = null;
    }
  }
});

onMounted( loadTemplates );

onBeforeUnmount( () => {
  loadRequestSequence += 1;
});

async function loadTemplates() {
  const request = ++loadRequestSequence;

  clearError();
  initialLoading.value = true;
  loadError.value = '';

  try {
    const templates = await getTemplates();

    if ( request !== loadRequestSequence ) {
      return;
    }

    catalog.value = templates;
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

async function confirmDelete() {
  const target = deleteTarget.value;

  if ( !target ) {
    return;
  }

  clearError();

  try {
    await deleteTemplate( target.id );

    catalog.value.templates = catalog.value.templates.filter(
      ( template ) => template.id !== target.id
    );
    deleteTarget.value = null;
  } catch {
    // Error state is handled by the composable.
  }
}

function modeLabel( mode ) {
  return mode === 'custom' ? 'HTML & CSS' : 'Visual';
}

function formattedDate( timestamp ) {
  return new Intl.DateTimeFormat( undefined, {
    day: 'numeric',
    month: 'short',
    year: 'numeric'
  }).format( new Date( timestamp ) );
}
</script>

<template>
  <div class="page templates-page">
    <PageHeader title="Templates">
      <template #actions>
        <UButton
          :to="{ name: 'library' }"
          leading-icon="i-lucide-arrow-left"
          color="neutral"
          variant="ghost"
        >
          Library
        </UButton>

        <UButton
          :to="{ name: 'template-create' }"
          leading-icon="i-lucide-plus"
        >
          New template
        </UButton>
      </template>
    </PageHeader>

    <ContentState
      v-if="initialLoading"
      kind="loading"
      title="Loading templates"
    />

    <ContentState
      v-else-if="loadError"
      kind="error"
      title="Templates could not be loaded"
      :description="loadError"
    >
      <template #actions>
        <UButton
          leading-icon="i-lucide-refresh-cw"
          @click="loadTemplates"
        >
          Retry
        </UButton>
      </template>
    </ContentState>

    <div
      v-else
      class="template-catalog"
    >
      <UAlert
        v-if="error"
        :description="error"
        icon="i-lucide-circle-alert"
        color="error"
        variant="soft"
      />

      <ContentState
        v-if="!catalog.templates.length"
        title="No templates yet"
        description="Create a reusable layout for concept fields."
      >
        <template #actions>
          <UButton
            :to="{ name: 'template-create' }"
            leading-icon="i-lucide-plus"
          >
            New template
          </UButton>
        </template>
      </ContentState>

      <div
        v-else
        class="template-list"
      >
        <AnimatePresence :initial="false">
          <m.article
            v-for="( template, index ) in catalog.templates"
            :key="template.id"
            class="template-card"
            layout
            :initial="{ opacity: 0, y: 8 }"
            :animate="{ opacity: 1, y: 0 }"
            :exit="{ opacity: 0, scale: 0.98 }"
            :transition="{ delay: Math.min( index * 0.025, 0.15 ) }"
          >
            <RouterLink
              :to="{
                name: 'template-edit',
                params: { templateId: template.id }
              }"
              class="template-card__link"
            >
              <span
                class="template-card__icon"
                aria-hidden="true"
              >
                <UIcon name="i-lucide-panels-top-left" />
              </span>

              <div class="template-card__copy">
                <div class="template-card__title">
                  <h2>{{ template.name }}</h2>

                  <UBadge
                    :label="modeLabel( template.mode )"
                    color="neutral"
                    variant="soft"
                  />
                </div>

                <p>Updated {{ formattedDate( template.updatedAt ) }}</p>
              </div>

              <UIcon
                name="i-lucide-chevron-right"
                class="template-card__arrow"
                aria-hidden="true"
              />
            </RouterLink>

            <UButton
              type="button"
              icon="i-lucide-trash-2"
              :aria-label="`Delete ${ template.name }`"
              color="error"
              variant="ghost"
              square
              @click="deleteTarget = template"
            />
          </m.article>
        </AnimatePresence>
      </div>
    </div>

    <ConfirmDialog
      v-model:open="deleteDialogOpen"
      title="Delete template?"
      :description="deleteTarget
        ? `${ deleteTarget.name } will no longer be available for retrieval forms.`
        : ''"
      confirm-label="Delete template"
      :loading="isPending"
      @confirm="confirmDelete"
    />
  </div>
</template>
