import { createRouter, createWebHashHistory } from 'vue-router';

import LibraryView from '../views/LibraryView.vue';
import SettingsView from '../views/SettingsView.vue';
import StartupView from '../views/StartupView.vue';

const routes = [
  {
    path: '/',
    name: 'startup',
    component: StartupView,
    meta: {
      title: ''
    }
  },

  {
    path: '/study',
    name: 'study',
    component: loadStudyView,
    meta: {
      title: 'Study'
    }
  },

  {
    path: '/library',
    name: 'library',
    component: LibraryView,
    meta: {
      title: 'Library'
    }
  },

  {
    path: '/library/:conceptId/edit',
    name: 'concept-edit',
    component: loadCreateView,
    meta: {
      title: 'Edit concept'
    }
  },

  {
    path: '/library/:conceptId',
    name: 'concept-detail',
    component: loadConceptDetailView,
    meta: {
      title: 'Concept'
    }
  },

  {
    path: '/create',
    name: 'create',
    component: loadCreateView,
    meta: {
      title: 'Create'
    }
  },

  {
    path: '/templates',
    name: 'templates',
    component: loadTemplatesView,
    meta: {
      title: 'Templates'
    }
  },

  {
    path: '/templates/new',
    name: 'template-create',
    component: loadTemplateEditorView,
    meta: {
      title: 'New template'
    }
  },

  {
    path: '/templates/:templateId',
    name: 'template-edit',
    component: loadTemplateEditorView,
    meta: {
      title: 'Edit template'
    }
  },

  {
    path: '/settings',
    name: 'settings',
    component: SettingsView,
    meta: {
      title: 'Settings'
    }
  },

  {
    path: '/:pathMatch(.*)*',
    redirect: '/study'
  }
];

const router = createRouter({
  history: createWebHashHistory(),
  routes
});

router.afterEach( ( to ) => {
  document.title = to.meta.title
    ? `${ to.meta.title } · Twill`
    : 'Twill';
});

export default router;

function loadConceptDetailView() {
  return import( '../views/ConceptDetailView.vue' );
}

function loadCreateView() {
  return import( '../views/CreateView.vue' );
}

function loadStudyView() {
  return import( '../views/StudyView.vue' );
}

function loadTemplateEditorView() {
  return import( '../views/TemplateEditorView.vue' );
}

function loadTemplatesView() {
  return import( '../views/TemplatesView.vue' );
}
