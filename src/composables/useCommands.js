import {
  computed,
  inject,
  onBeforeUnmount,
  onMounted,
  provide,
  shallowReactive,
  unref
} from 'vue';

import {
  commandAriaShortcut,
  commandDefinition,
  commandMatchesKeyboardEvent,
  commandRegistry,
  commandShortcutLabel,
  commandShortcutParts,
  isApplePlatform
} from '../commands/registry';

const COMMANDS_KEY = Symbol( 'commands' );
const NATIVE_INTERACTIVE_SELECTOR = [
  'a[href]',
  'button',
  'input',
  'select',
  'summary',
  'textarea',
  '[contenteditable]:not([contenteditable="false"])',
  '[role="button"]',
  '[role="checkbox"]',
  '[role="combobox"]',
  '[role="gridcell"]',
  '[role="link"]',
  '[role="listbox"]',
  '[role="menuitem"]',
  '[role="menuitemcheckbox"]',
  '[role="menuitemradio"]',
  '[role="option"]',
  '[role="radio"]',
  '[role="searchbox"]',
  '[role="slider"]',
  '[role="spinbutton"]',
  '[role="switch"]',
  '[role="tab"]',
  '[role="textbox"]',
  '[role="treeitem"]'
].join( ', ' );

export function provideCommands( router ) {
  const runtime = createCommandRuntime( router );

  provide( COMMANDS_KEY, runtime );
  onMounted( runtime.start );
  onBeforeUnmount( runtime.stop );

  return runtime;
}

export function useCommands() {
  const commands = inject( COMMANDS_KEY );

  if ( !commands ) {
    throw new Error( 'Command runtime is not available.' );
  }

  return commands;
}

export function useCommandHandler( commandId, options ) {
  const commands = useCommands();
  let unregister = () => {};

  onMounted( () => {
    unregister = commands.register( commandId, options );
  });

  onBeforeUnmount( () => {
    unregister();
  });

  return computed( () => commands.command( commandId ) );
}

function createCommandRuntime( router ) {
  const applePlatform = isApplePlatform();
  const bindings = shallowReactive( new Map() );

  function command( commandId ) {
    const definition = commandDefinition( commandId );
    const shortcutLabel = commandShortcutLabel( definition, applePlatform );
    const shortcutParts = commandShortcutParts( definition, applePlatform );

    return {
      ...definition,
      ariaKeyshortcuts: commandAriaShortcut( definition, applePlatform ),
      available: isAvailable( definition ),
      enabled: isEnabled( definition ),
      shortcutLabel,
      shortcutParts,
      tooltip: `${ definition.label } (${ shortcutLabel })`
    };
  }

  function list() {
    return commandRegistry.map( ( definition ) => command( definition.id ) );
  }

  function register( commandId, options ) {
    const definition = commandDefinition( commandId );

    if ( !definition.requiresBinding ) {
      throw new Error( `Command does not accept a local handler: ${ commandId }` );
    }

    const token = Symbol( commandId );
    const existingBindings = bindings.get( commandId ) ?? [];

    bindings.set( commandId, [ ...existingBindings, { ...options, token }]);

    return () => {
      const remainingBindings = ( bindings.get( commandId ) ?? []).filter(
        ( binding ) => binding.token !== token
      );

      if ( remainingBindings.length ) {
        bindings.set( commandId, remainingBindings );
      } else {
        bindings.delete( commandId );
      }
    };
  }

  async function execute( commandId ) {
    const selectedCommand = command( commandId );

    if ( !selectedCommand.available || !selectedCommand.enabled ) {
      return false;
    }

    await selectedCommand.handler({
      binding: activeBinding( commandId ),
      router
    });

    return true;
  }

  function start() {
    window.addEventListener( 'keydown', handleKeydown );
  }

  function stop() {
    window.removeEventListener( 'keydown', handleKeydown );
  }

  function activeBinding( commandId ) {
    return bindings.get( commandId )?.at( -1 ) ?? null;
  }

  function isAvailable( definition ) {
    return !definition.requiresBinding || Boolean( activeBinding( definition.id ) );
  }

  function isEnabled( definition ) {
    const binding = activeBinding( definition.id );

    if ( definition.requiresBinding && !binding ) {
      return false;
    }

    return binding?.enabled === undefined || Boolean( unref( binding.enabled ) );
  }

  function handleKeydown( event ) {
    if (
      event.defaultPrevented
      || event.repeat
      || event.isComposing
      || event.keyCode === 229
      || document.querySelector( '[role="dialog"]' )
      || shouldPreserveNativeInteraction( event )
    ) {
      return;
    }

    const editing = isEditingTarget( event.target );
    const selectedCommand = list().find( ( candidate ) => (
      candidate.available
      && candidate.enabled
      && ( !editing || candidate.allowInEditable )
      && commandMatchesKeyboardEvent( candidate, event, applePlatform )
    ) );

    if ( !selectedCommand ) {
      return;
    }

    event.preventDefault();
    execute( selectedCommand.id );
  }

  return {
    command,
    execute,
    list,
    register,
    start,
    stop
  };
}

function shouldPreserveNativeInteraction( event ) {
  if ( /^F(?:[1-9]|1[0-2])$/u.test( event.key ) ) {
    return false;
  }

  if ( event.altKey || event.ctrlKey || event.metaKey ) {
    return false;
  }

  return event.target instanceof Element
    && Boolean( event.target.closest( NATIVE_INTERACTIVE_SELECTOR ) );
}

function isEditingTarget( target ) {
  return target instanceof HTMLElement && (
    target.isContentEditable
    || [ 'INPUT', 'SELECT', 'TEXTAREA' ].includes( target.tagName )
    || Boolean( target.closest( '[role="combobox"], [role="listbox"]' ) )
  );
}
