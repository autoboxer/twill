const fieldPattern = /{{\s*(title|prompt|answer)\s*}}/g;

const contentSecurityPolicy = [
  "default-src 'none'",
  "style-src 'unsafe-inline'",
  'img-src data:',
  'font-src data:',
  "connect-src 'none'",
  "media-src 'none'",
  "object-src 'none'",
  "frame-src 'none'",
  "base-uri 'none'",
  "form-action 'none'"
].join( '; ' );

export function createCustomTemplateDocument( source, css, fields, appearance = {}) {
  const content = source.replace( fieldPattern, ( _, field ) => fields[ field ]);
  const fontCss = safeFontCss( appearance.fontCss );

  return [
    '<!doctype html>',
    '<html>',
    '<head>',
    '<meta charset="utf-8">',
    '<meta name="viewport" content="width=device-width, initial-scale=1">',
    `<meta http-equiv="Content-Security-Policy" content="${ contentSecurityPolicy }">`,
    '<style>',
    fontCss,
    baseFrameCss( appearance ),
    css,
    '</style>',
    '</head>',
    `<body>${ content }</body>`,
    '</html>'
  ].join( '' );
}

function baseFrameCss( appearance ) {
  const colorScheme = appearance.colorScheme === 'dark' ? 'dark' : 'light';
  const background = safeCssValue( appearance.background, '#fdfdfb' );
  const surface = safeCssValue( appearance.surface, '#ffffff' );
  const text = safeCssValue( appearance.text, '#303831' );
  const highlightedText = safeCssValue(
    appearance.highlightedText,
    '#182019'
  );
  const mutedText = safeCssValue( appearance.mutedText, '#59645a' );
  const border = safeCssValue( appearance.border, '#d2dad0' );
  const primary = safeCssValue( appearance.primary, '#4d694a' );
  const readingFont = safeCssValue(
    appearance.readingFont,
    'system-ui, sans-serif'
  );
  const readingTextSize = safeCssValue(
    appearance.readingTextSize,
    '1.0625rem'
  );
  const codeBackground = safeCssValue(
    appearance.codeBackground,
    '#202820'
  );
  const codeText = safeCssValue( appearance.codeText, '#e7eee3' );
  const syntaxComment = safeCssValue(
    appearance.syntaxComment,
    '#9aae99'
  );
  const syntaxKeyword = safeCssValue(
    appearance.syntaxKeyword,
    '#d7b9ea'
  );
  const syntaxString = safeCssValue(
    appearance.syntaxString,
    '#b9d9a9'
  );
  const syntaxNumber = safeCssValue(
    appearance.syntaxNumber,
    '#e9c88d'
  );
  const syntaxTitle = safeCssValue(
    appearance.syntaxTitle,
    '#a9d7dd'
  );
  const errorText = colorScheme === 'dark' ? '#f2aaa6' : '#7b302f';
  const errorBackground = colorScheme === 'dark' ? '#321d1b' : '#fff3f2';
  const errorBorder = colorScheme === 'dark' ? '#6b3531' : '#efcfcc';

  return [
    `:root { color-scheme: ${ colorScheme }; --twill-background: ${ background }; --twill-surface: ${ surface }; --twill-text: ${ text }; --twill-text-highlighted: ${ highlightedText }; --twill-text-muted: ${ mutedText }; --twill-border: ${ border }; --twill-primary: ${ primary }; --twill-reading-font: ${ readingFont }; --twill-reading-text-size: ${ readingTextSize }; --twill-code-background: ${ codeBackground }; --twill-code-text: ${ codeText }; --twill-syntax-comment: ${ syntaxComment }; --twill-syntax-keyword: ${ syntaxKeyword }; --twill-syntax-string: ${ syntaxString }; --twill-syntax-number: ${ syntaxNumber }; --twill-syntax-title: ${ syntaxTitle }; }`,
    '* { box-sizing: border-box; }',
    'html, body { min-height: 100%; margin: 0; }',
    'body { padding: 24px; color: var(--twill-text); background: var(--twill-background); font-family: var(--twill-reading-font); font-size: var(--twill-reading-text-size); line-height: 1.7; overflow-wrap: anywhere; }',
    'h1, h2, h3 { color: var(--twill-text-highlighted); line-height: 1.25; }',
    'h1 { font-size: 1.6rem; }',
    'h2 { font-size: 1.35rem; }',
    'h3 { font-size: 1.15rem; }',
    'blockquote { padding-left: 1rem; margin-left: 0; color: var(--twill-text-muted); border-left: 3px solid var(--twill-primary); }',
    'pre { max-width: 100%; overflow: auto; padding: 1rem; color: var(--twill-code-text); background: var(--twill-code-background); border-radius: 0.6rem; white-space: pre; }',
    'code { font-family: ui-monospace, SFMono-Regular, Consolas, monospace; }',
    ':not(pre) > code { padding: 0.12em 0.35em; color: var(--twill-text-highlighted); background: var(--twill-surface); border: 1px solid var(--twill-border); border-radius: 0.3rem; }',
    '.hljs-comment, .hljs-quote { color: var(--twill-syntax-comment); }',
    '.hljs-keyword, .hljs-selector-tag, .hljs-literal { color: var(--twill-syntax-keyword); }',
    '.hljs-string, .hljs-attr, .hljs-template-variable { color: var(--twill-syntax-string); }',
    '.hljs-number, .hljs-symbol, .hljs-bullet { color: var(--twill-syntax-number); }',
    '.hljs-title, .hljs-built_in, .hljs-type { color: var(--twill-syntax-title); }',
    'hr { border: 0; border-top: 1px solid var(--twill-border); }',
    'a { color: var(--twill-primary); font-weight: 550; text-decoration: underline; text-underline-offset: 0.16em; pointer-events: none; }',
    'math { max-width: 100%; font-size: 1.08em; }',
    '.twill-math--block { display: block; max-width: 100%; overflow-x: auto; text-align: center; }',
    '.twill-field > :first-child { margin-top: 0; }',
    '.twill-field > :last-child { margin-bottom: 0; }',
    '.twill-title { color: var(--twill-text-highlighted); font-size: 1.3rem; font-weight: 700; line-height: 1.3; }',
    '.twill-media-image { margin: 1.25rem 0; }',
    '.twill-media-image img { display: block; max-width: 100%; height: auto; margin: 0 auto; border-radius: 0.65rem; }',
    `.twill-media-image--error { padding: 1.25rem; color: ${ errorText }; background: ${ errorBackground }; border: 1px solid ${ errorBorder }; border-radius: 0.65rem; text-align: center; }`
  ].join( '' );
}

function safeCssValue( value, fallback ) {
  if (
    typeof value !== 'string'
    || value.length > 300
    || /[;{}<>\n\r]/.test( value )
  ) {
    return fallback;
  }

  return value.trim() || fallback;
}

function safeFontCss( value ) {
  if (
    typeof value !== 'string'
    || value.length > 200_000
    || value.toLowerCase().includes( '</style' )
  ) {
    return '';
  }

  return value;
}
