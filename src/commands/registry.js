export const COMMAND_IDS = Object.freeze({
  commandPaletteOpen: 'command.palette.open',
  commandReferenceOpen: 'command.reference.open',
  conceptCreate: 'concept.create',
  conceptSave: 'concept.save',
  navigateLibrary: 'navigate.library',
  navigateSettings: 'navigate.settings',
  navigateStudy: 'navigate.study',
  schedulingSave: 'scheduling.save',
  studyGradeAdvancedAgain: 'study.grade.advanced.again',
  studyGradeAdvancedEasy: 'study.grade.advanced.easy',
  studyGradeAdvancedGood: 'study.grade.advanced.good',
  studyGradeAdvancedHard: 'study.grade.advanced.hard',
  studyGradeSimpleForgot: 'study.grade.simple.forgot',
  studyGradeSimpleRemembered: 'study.grade.simple.remembered',
  studyMasteryMissed: 'study.mastery.missed',
  studyMasteryRecalled: 'study.mastery.recalled',
  studyQueueEdit: 'study.queue-edit',
  studyReveal: 'study.reveal',
  studyUndoLastGrade: 'study.undo-last-grade',
  templateSave: 'template.save'
});

export const commandRegistry = Object.freeze([
  localCommand({
    id: COMMAND_IDS.commandPaletteOpen,
    context: 'Outside editors and dialogs',
    description: 'Search and run available actions.',
    group: 'Application',
    icon: 'i-lucide-search',
    label: 'Command palette',
    shortcut: 'Mod+Shift+P'
  }),
  localCommand({
    id: COMMAND_IDS.commandReferenceOpen,
    context: 'Outside editors and dialogs',
    description: 'View the available keyboard shortcuts.',
    group: 'Application',
    icon: 'i-lucide-keyboard',
    label: 'Keyboard shortcuts',
    shortcut: 'Mod+/'
  }),
  navigationCommand({
    id: COMMAND_IDS.navigateStudy,
    description: 'Go to the study queue.',
    icon: 'i-lucide-book-open-check',
    label: 'Open Study',
    routeName: 'study',
    shortcut: 'Mod+1'
  }),
  navigationCommand({
    id: COMMAND_IDS.navigateLibrary,
    description: 'View and organize concepts.',
    icon: 'i-lucide-library',
    label: 'Open Library',
    routeName: 'library',
    shortcut: 'Mod+2'
  }),
  navigationCommand({
    id: COMMAND_IDS.conceptCreate,
    description: 'Open a new concept editor.',
    icon: 'i-lucide-square-pen',
    label: 'New concept',
    routeName: 'create',
    shortcut: 'Mod+N'
  }),
  navigationCommand({
    id: COMMAND_IDS.navigateSettings,
    description: 'Change application preferences.',
    icon: 'i-lucide-settings',
    label: 'Open Settings',
    routeName: 'settings',
    shortcut: 'Mod+,'
  }),
  localCommand({
    id: COMMAND_IDS.conceptSave,
    allowInEditable: true,
    context: 'Concept editor',
    description: 'Save the current concept.',
    group: 'Authoring',
    icon: 'i-lucide-save',
    label: 'Save concept',
    shortcut: 'Mod+S'
  }),
  localCommand({
    id: COMMAND_IDS.templateSave,
    allowInEditable: true,
    context: 'Template editor',
    description: 'Save the current template.',
    group: 'Authoring',
    icon: 'i-lucide-save',
    label: 'Save template',
    shortcut: 'Mod+S'
  }),
  localCommand({
    id: COMMAND_IDS.schedulingSave,
    allowInEditable: true,
    context: 'Scheduling settings',
    description: 'Save the scheduling form.',
    group: 'Settings',
    icon: 'i-lucide-save',
    label: 'Save scheduling settings',
    shortcut: 'Mod+S'
  }),
  localCommand({
    id: COMMAND_IDS.studyQueueEdit,
    context: 'Study, on the current card',
    description: 'Queue the current concept to edit after the session.',
    group: 'Study',
    icon: 'i-lucide-list-plus',
    label: 'Edit concept later',
    shortcut: 'F2'
  }),
  localCommand({
    id: COMMAND_IDS.studyReveal,
    context: 'Study, before answer reveal',
    description: 'Reveal the current answer.',
    group: 'Study',
    icon: 'i-lucide-eye',
    label: 'Reveal answer',
    shortcut: 'Space'
  }),
  localCommand({
    id: COMMAND_IDS.studyUndoLastGrade,
    context: 'Study, after a grade',
    description: 'Remove the last saved grade and return to that answer.',
    group: 'Study',
    icon: 'i-lucide-undo-2',
    label: 'Undo last grade',
    shortcut: 'Mod+Z'
  }),
  studyGradeCommand({
    id: COMMAND_IDS.studyGradeSimpleForgot,
    mode: 'Simple',
    label: 'Forgot',
    rating: 'again',
    shortcut: '1'
  }),
  studyGradeCommand({
    id: COMMAND_IDS.studyGradeSimpleRemembered,
    mode: 'Simple',
    label: 'Remembered',
    rating: 'good',
    shortcut: '2'
  }),
  studyGradeCommand({
    id: COMMAND_IDS.studyGradeAdvancedAgain,
    mode: 'Advanced',
    label: 'Again',
    rating: 'again',
    shortcut: '1'
  }),
  studyGradeCommand({
    id: COMMAND_IDS.studyGradeAdvancedHard,
    mode: 'Advanced',
    label: 'Hard',
    rating: 'hard',
    shortcut: '2'
  }),
  studyGradeCommand({
    id: COMMAND_IDS.studyGradeAdvancedGood,
    mode: 'Advanced',
    label: 'Good',
    rating: 'good',
    shortcut: '3'
  }),
  studyGradeCommand({
    id: COMMAND_IDS.studyGradeAdvancedEasy,
    mode: 'Advanced',
    label: 'Easy',
    rating: 'easy',
    shortcut: '4'
  }),
  localCommand({
    id: COMMAND_IDS.studyMasteryMissed,
    context: 'Study, mastery retry after reveal',
    description: 'Mark the retry as still missed.',
    group: 'Study',
    icon: 'i-lucide-rotate-ccw',
    label: 'Still missed',
    shortcut: '1'
  }),
  localCommand({
    id: COMMAND_IDS.studyMasteryRecalled,
    context: 'Study, mastery retry after reveal',
    description: 'Mark the retry as recalled.',
    group: 'Study',
    icon: 'i-lucide-check',
    label: 'Recalled',
    shortcut: '2'
  })
]);

const commandsById = new Map( commandRegistry.map( ( command ) => [
  command.id,
  command
]) );

export function commandDefinition( commandId ) {
  const command = commandsById.get( commandId );

  if ( !command ) {
    throw new Error( `Unknown command: ${ commandId }` );
  }

  return command;
}

export function commandShortcutLabel( command, applePlatform = isApplePlatform() ) {
  return commandShortcutParts( command, applePlatform )
    .join( applePlatform ? '' : '+' );
}

export function commandShortcutParts( command, applePlatform = isApplePlatform() ) {
  return command.shortcut
    .split( '+' )
    .map( ( part ) => shortcutPartLabel( part, applePlatform ) );
}

export function commandAriaShortcut( command, applePlatform = isApplePlatform() ) {
  return command.shortcut
    .split( '+' )
    .map( ( part ) => ariaShortcutPart( part, applePlatform ) )
    .join( '+' );
}

export function commandMatchesKeyboardEvent(
  command,
  event,
  applePlatform = isApplePlatform()
) {
  const parts = command.shortcut.split( '+' );
  const key = parts.at( -1 );
  const modifiers = new Set( parts.slice( 0, -1 ) );
  const expectsControl = modifiers.has( 'Control' )
    || ( modifiers.has( 'Mod' ) && !applePlatform );
  const expectsMeta = modifiers.has( 'Meta' )
    || ( modifiers.has( 'Mod' ) && applePlatform );

  return event.ctrlKey === expectsControl
    && event.metaKey === expectsMeta
    && event.altKey === modifiers.has( 'Alt' )
    && event.shiftKey === modifiers.has( 'Shift' )
    && normalizedEventKey( event.key ) === normalizedShortcutKey( key );
}

export function isApplePlatform() {
  const platform = navigator.userAgentData?.platform ?? navigator.platform ?? '';

  return /mac|iphone|ipad|ipod/i.test( platform );
}

function navigationCommand({ description, id, icon, label, routeName, shortcut }) {
  return Object.freeze({
    context: 'Outside editors and dialogs',
    description,
    group: 'Navigation',
    handler: ({ router }) => router.push({ name: routeName }),
    id,
    icon,
    label,
    shortcut
  });
}

function localCommand({
  allowInEditable = false,
  context,
  description,
  group,
  icon,
  id,
  label,
  shortcut
}) {
  return Object.freeze({
    allowInEditable,
    context,
    description,
    group,
    handler: ({ binding }) => binding.execute(),
    id,
    icon,
    label,
    requiresBinding: true,
    shortcut
  });
}

function studyGradeCommand({ id, label, mode, rating, shortcut }) {
  return Object.freeze({
    ...localCommand({
      context: `Study, ${ mode } grading after reveal`,
      description: `Grade the current answer as ${ label }.`,
      group: 'Study',
      icon: 'i-lucide-list-checks',
      id,
      label,
      shortcut
    }),
    rating
  });
}

function shortcutPartLabel( part, applePlatform ) {
  const labels = applePlatform
    ? {
      Alt: '⌥',
      Control: '⌃',
      Meta: '⌘',
      Mod: '⌘',
      Shift: '⇧',
      Space: 'Space'
    }
    : {
      Alt: 'Alt',
      Control: 'Ctrl',
      Meta: 'Meta',
      Mod: 'Ctrl',
      Shift: 'Shift',
      Space: 'Space'
    };

  return labels[ part ] ?? part.toUpperCase();
}

function ariaShortcutPart( part, applePlatform ) {
  if ( part === 'Mod' ) {
    return applePlatform ? 'Meta' : 'Control';
  }

  return part;
}

function normalizedEventKey( key ) {
  if ( key === ' ' ) {
    return 'space';
  }

  return key.toLowerCase();
}

function normalizedShortcutKey( key ) {
  return key.toLowerCase();
}
