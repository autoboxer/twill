const screen = document.getElementById( 'twill-startup' );
const app = document.getElementById( 'app' );
const reloadButton = document.getElementById( 'startup-reload' );
const reducedMotion = window.matchMedia( '(prefers-reduced-motion: reduce)' );

let phase = 'opening';
let exitTimer;

const startupTimeout = setTimeout( failStartup, 20000 );

reloadButton.addEventListener( 'click', () => location.reload() );
window.addEventListener( 'error', handleStartupError );
window.addEventListener( 'unhandledrejection', handleStartupError );

void import( './main' )
  .then( ({ startApplication }) => startApplication() )
  .catch( failStartup );

export function isStartupPending() {
  return phase === 'opening';
}

export function completeStartup() {
  if ( !isStartupPending() ) {
    return;
  }

  phase = 'leaving';
  clearTimeout( startupTimeout );

  requestAnimationFrame( () => {
    if ( phase !== 'leaving' ) {
      return;
    }

    screen.querySelector( 'svg' )?.setAttribute( 'data-paused', 'true' );

    if ( document.documentElement.dataset.motion === 'reduced'
      || reducedMotion.matches ) {
      removeStartup();
      return;
    }

    screen.addEventListener( 'transitionend', handleExitTransition );
    screen.dataset.state = 'leaving';
    exitTimer = setTimeout( removeStartup, 220 );
  });
}

export function failStartup( cause ) {
  if ( phase !== 'opening' && phase !== 'leaving' ) {
    return;
  }

  phase = 'failed';
  clearTimeout( startupTimeout );
  clearTimeout( exitTimer );
  screen.removeEventListener( 'transitionend', handleExitTransition );
  screen.dataset.state = 'failed';
  screen.removeAttribute( 'role' );
  screen.removeAttribute( 'aria-label' );
  screen.querySelector( 'svg' )?.setAttribute( 'data-motion', 'reduced' );
  document.getElementById( 'startup-error' ).hidden = false;
  reloadButton.focus({ preventScroll: true });
  removeErrorListeners();

  if ( cause ) {
    console.error( 'Twill could not finish opening.', cause );
  }
}

function handleStartupError( event ) {
  failStartup( event.reason ?? event.error );
}

function handleExitTransition( event ) {
  if ( event.target === screen && event.propertyName === 'opacity' ) {
    removeStartup();
  }
}

function removeStartup() {
  if ( phase !== 'leaving' ) {
    return;
  }

  phase = 'ready';
  clearTimeout( exitTimer );
  removeErrorListeners();
  screen.remove();
  app.inert = false;
  app.removeAttribute( 'aria-hidden' );

  if ( document.activeElement === document.body ) {
    document.getElementById( 'main-content' )?.focus({ preventScroll: true });
  }
}

function removeErrorListeners() {
  window.removeEventListener( 'error', handleStartupError );
  window.removeEventListener( 'unhandledrejection', handleStartupError );
}
