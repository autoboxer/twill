import { COMMAND_IDS } from '../commands/registry';

export const primaryNavigation = [
  {
    commandId: COMMAND_IDS.navigateStudy,
    label: 'Study',
    icon: 'i-lucide-book-open-check',
    to: '/study'
  },

  {
    commandId: COMMAND_IDS.navigateLibrary,
    label: 'Library',
    icon: 'i-lucide-library',
    to: '/library'
  },

  {
    commandId: COMMAND_IDS.conceptCreate,
    label: 'Create',
    icon: 'i-lucide-square-pen',
    to: '/create'
  },

  {
    commandId: COMMAND_IDS.navigateSettings,
    label: 'Settings',
    icon: 'i-lucide-settings',
    to: '/settings'
  }
];
