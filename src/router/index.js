import { createRouter, createWebHashHistory } from 'vue-router';

import ConceptDetailView from '../views/ConceptDetailView.vue';
import CreateView from '../views/CreateView.vue';
import LibraryView from '../views/LibraryView.vue';
import SettingsView from '../views/SettingsView.vue';
import StudyView from '../views/StudyView.vue';

const routes = [
  {
    path: '/',
    redirect: '/study'
  },
  {
    path: '/study',
    name: 'study',
    component: StudyView,
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
    component: CreateView,
    meta: {
      title: 'Edit concept'
    }
  },
  {
    path: '/library/:conceptId',
    name: 'concept-detail',
    component: ConceptDetailView,
    meta: {
      title: 'Concept'
    }
  },
  {
    path: '/create',
    name: 'create',
    component: CreateView,
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
