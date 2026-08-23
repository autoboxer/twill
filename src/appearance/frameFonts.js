import ibmPlexSansItalic from '@fontsource-variable/ibm-plex-sans/files/ibm-plex-sans-latin-wght-italic.woff2?url';
import ibmPlexSansNormal from '@fontsource-variable/ibm-plex-sans/files/ibm-plex-sans-latin-wght-normal.woff2?url';
import interItalic from '@fontsource-variable/inter/files/inter-latin-wght-italic.woff2?url';
import interNormal from '@fontsource-variable/inter/files/inter-latin-wght-normal.woff2?url';
import jetBrainsMonoItalic from '@fontsource-variable/jetbrains-mono/files/jetbrains-mono-latin-wght-italic.woff2?url';
import jetBrainsMonoNormal from '@fontsource-variable/jetbrains-mono/files/jetbrains-mono-latin-wght-normal.woff2?url';
import sourceSerif4Italic from '@fontsource-variable/source-serif-4/files/source-serif-4-latin-wght-italic.woff2?url';
import sourceSerif4Normal from '@fontsource-variable/source-serif-4/files/source-serif-4-latin-wght-normal.woff2?url';

const frameFonts = {
  ibmPlexSans: {
    family: 'IBM Plex Sans Variable',
    italic: ibmPlexSansItalic,
    normal: ibmPlexSansNormal,
    weight: '100 700'
  },

  inter: {
    family: 'Inter Variable',
    italic: interItalic,
    normal: interNormal,
    weight: '100 900'
  },

  jetBrainsMono: {
    family: 'JetBrains Mono Variable',
    italic: jetBrainsMonoItalic,
    normal: jetBrainsMonoNormal,
    weight: '100 800'
  },

  sourceSerif4: {
    family: 'Source Serif 4 Variable',
    italic: sourceSerif4Italic,
    normal: sourceSerif4Normal,
    weight: '200 900'
  }
};
const frameFontCssCache = new Map();

export function loadFrameFontCss( readingFont ) {
  const font = frameFonts[ readingFont ];

  if ( !font ) {
    return Promise.resolve( '' );
  }

  if ( !frameFontCssCache.has( readingFont ) ) {
    const request = createFontCss( font ).catch( ( cause ) => {
      frameFontCssCache.delete( readingFont );
      throw cause;
    });

    frameFontCssCache.set( readingFont, request );
  }

  return frameFontCssCache.get( readingFont );
}

async function createFontCss( font ) {
  const [ normal, italic ] = await Promise.all([
    loadDataUrl( font.normal ),
    loadDataUrl( font.italic )
  ]);

  return [
    `@font-face { font-family: "${ font.family }"; font-style: normal; font-display: swap; font-weight: ${ font.weight }; src: url("${ normal }") format("woff2-variations"); }`,
    `@font-face { font-family: "${ font.family }"; font-style: italic; font-display: swap; font-weight: ${ font.weight }; src: url("${ italic }") format("woff2-variations"); }`
  ].join( '' );
}

async function loadDataUrl( source ) {
  const response = await fetch( source );

  if ( !response.ok ) {
    throw new Error( `Frame font could not be loaded: ${ response.status }` );
  }

  const blob = await response.blob();

  return new Promise( ( resolve, reject ) => {
    const reader = new FileReader();

    reader.addEventListener( 'load', () => resolve( reader.result ), {
      once: true
    });
    reader.addEventListener( 'error', () => reject( reader.error ), {
      once: true
    });
    reader.readAsDataURL( blob );
  });
}
