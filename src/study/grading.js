import { COMMAND_IDS } from '../commands/registry';

export const gradingModeItems = [
  { label: 'Simple', value: 'simple' },
  { label: 'Advanced', value: 'advanced' }
];

export const gradingOptionsByMode = {
  simple: [
    {
      color: 'error',
      commandId: COMMAND_IDS.studyGradeSimpleForgot,
      icon: 'i-lucide-rotate-ccw',
      rating: 'again',
      variant: 'soft'
    },

    {
      color: 'primary',
      commandId: COMMAND_IDS.studyGradeSimpleRemembered,
      icon: 'i-lucide-check',
      rating: 'good',
      variant: 'solid'
    }
  ],
  advanced: [
    {
      color: 'error',
      commandId: COMMAND_IDS.studyGradeAdvancedAgain,
      icon: 'i-lucide-rotate-ccw',
      rating: 'again',
      variant: 'soft'
    },

    {
      color: 'warning',
      commandId: COMMAND_IDS.studyGradeAdvancedHard,
      icon: 'i-lucide-gauge',
      rating: 'hard',
      variant: 'soft'
    },

    {
      color: 'primary',
      commandId: COMMAND_IDS.studyGradeAdvancedGood,
      icon: 'i-lucide-check',
      rating: 'good',
      variant: 'soft'
    },

    {
      color: 'success',
      commandId: COMMAND_IDS.studyGradeAdvancedEasy,
      icon: 'i-lucide-sparkles',
      rating: 'easy',
      variant: 'soft'
    }
  ]
};
