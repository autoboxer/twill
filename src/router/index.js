import { createRouter, createWebHashHistory } from 'vue-router';

import LibraryView from '../views/LibraryView.vue';
import SettingsView from '../views/SettingsView.vue';

const routes = [
  {
    path: '/',
    redirect: '/study'
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
  document.title = `${ to.meta.title } · Twill`;
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
