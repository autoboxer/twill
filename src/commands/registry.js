export const COMMAND_IDS = Object.freeze({
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
  studyReveal: 'study.reveal',
  templateSave: 'template.save'
});

export const commandRegistry = Object.freeze([
  navigationCommand({
    id: COMMAND_IDS.navigateStudy,
    icon: 'i-lucide-book-open-check',
    label: 'Open Study',
    routeName: 'study',
    shortcut: 'Mod+1'
  }),
  navigationCommand({
    id: COMMAND_IDS.navigateLibrary,
    icon: 'i-lucide-library',
    label: 'Open Library',
    routeName: 'library',
    shortcut: 'Mod+2'
  }),
  navigationCommand({
    id: COMMAND_IDS.conceptCreate,
    icon: 'i-lucide-square-pen',
    label: 'New concept',
    routeName: 'create',
    shortcut: 'Mod+N'
  }),
  navigationCommand({
    id: COMMAND_IDS.navigateSettings,
    icon: 'i-lucide-settings',
    label: 'Open Settings',
    routeName: 'settings',
    shortcut: 'Mod+,'
  }),
  localCommand({
    id: COMMAND_IDS.conceptSave,
    allowInEditable: true,
    group: 'Authoring',
    icon: 'i-lucide-save',
    label: 'Save concept',
    shortcut: 'Mod+S'
  }),
  localCommand({
    id: COMMAND_IDS.templateSave,
    allowInEditable: true,
    group: 'Authoring',
    icon: 'i-lucide-save',
    label: 'Save template',
    shortcut: 'Mod+S'
  }),
  localCommand({
    id: COMMAND_IDS.schedulingSave,
    allowInEditable: true,
    group: 'Settings',
    icon: 'i-lucide-save',
    label: 'Save scheduling settings',
    shortcut: 'Mod+S'
  }),
  localCommand({
    id: COMMAND_IDS.studyReveal,
    group: 'Study',
    icon: 'i-lucide-eye',
    label: 'Reveal answer',
    shortcut: 'Space'
  }),
  studyGradeCommand({
    id: COMMAND_IDS.studyGradeSimpleForgot,
    label: 'Forgot',
    rating: 'again',
    shortcut: '1'
  }),
  studyGradeCommand({
    id: COMMAND_IDS.studyGradeSimpleRemembered,
    label: 'Remembered',
    rating: 'good',
    shortcut: '2'
  }),
  studyGradeCommand({
    id: COMMAND_IDS.studyGradeAdvancedAgain,
    label: 'Again',
    rating: 'again',
    shortcut: '1'
  }),
  studyGradeCommand({
    id: COMMAND_IDS.studyGradeAdvancedHard,
    label: 'Hard',
    rating: 'hard',
    shortcut: '2'
  }),
  studyGradeCommand({
    id: COMMAND_IDS.studyGradeAdvancedGood,
    label: 'Good',
    rating: 'good',
    shortcut: '3'
  }),
  studyGradeCommand({
    id: COMMAND_IDS.studyGradeAdvancedEasy,
    label: 'Easy',
    rating: 'easy',
    shortcut: '4'
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
  return command.shortcut
    .split( '+' )
    .map( ( part ) => shortcutPartLabel( part, applePlatform ) )
    .join( applePlatform ? '' : '+' );
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

function navigationCommand({ id, icon, label, routeName, shortcut }) {
  return Object.freeze({
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
  group,
  icon,
  id,
  label,
  shortcut
}) {
  return Object.freeze({
    allowInEditable,
    group,
    handler: ({ binding }) => binding.execute(),
    id,
    icon,
    label,
    requiresBinding: true,
    shortcut
  });
}

function studyGradeCommand({ id, label, rating, shortcut }) {
  return Object.freeze({
    ...localCommand({
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
