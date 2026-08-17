const fieldPattern = /{{\s*(title|prompt|answer)\s*}}/g;

const contentSecurityPolicy = [
  "default-src 'none'",
  "style-src 'unsafe-inline'",
  'img-src data:',
  "font-src 'none'",
  "connect-src 'none'",
  "media-src 'none'",
  "object-src 'none'",
  "frame-src 'none'",
  "base-uri 'none'",
  "form-action 'none'"
].join( '; ' );

export function createCustomTemplateDocument( source, css, fields ) {
  const content = source.replace( fieldPattern, ( _, field ) => fields[ field ]);

  return [
    '<!doctype html>',
    '<html>',
    '<head>',
    '<meta charset="utf-8">',
    '<meta name="viewport" content="width=device-width, initial-scale=1">',
    `<meta http-equiv="Content-Security-Policy" content="${ contentSecurityPolicy }">`,
    '<style>',
    baseFrameCss(),
    css,
    '</style>',
    '</head>',
    `<body>${ content }</body>`,
    '</html>'
  ].join( '' );
}

function baseFrameCss() {
  return [
    ':root { color-scheme: light; }',
    '* { box-sizing: border-box; }',
    'html, body { min-height: 100%; margin: 0; }',
    'body { padding: 24px; color: #273029; background: #fbfcfa; font-family: system-ui, sans-serif; line-height: 1.7; overflow-wrap: anywhere; }',
    'h1, h2, h3 { color: #172019; line-height: 1.25; }',
    'h1 { font-size: 1.6rem; }',
    'h2 { font-size: 1.35rem; }',
    'h3 { font-size: 1.15rem; }',
    'blockquote { padding-left: 1rem; margin-left: 0; color: #59645c; border-left: 3px solid #789780; }',
    'pre { max-width: 100%; overflow: auto; padding: 1rem; color: #e7eee3; background: #202820; border-radius: 0.6rem; white-space: pre; }',
    'code { font-family: ui-monospace, SFMono-Regular, Consolas, monospace; }',
    ':not(pre) > code { padding: 0.12em 0.35em; color: #172019; background: #edf0ec; border: 1px solid #d7ded5; border-radius: 0.3rem; }',
    '.hljs-comment, .hljs-quote { color: #9aae99; }',
    '.hljs-keyword, .hljs-selector-tag, .hljs-literal { color: #d7b9ea; }',
    '.hljs-string, .hljs-attr, .hljs-template-variable { color: #b9d9a9; }',
    '.hljs-number, .hljs-symbol, .hljs-bullet { color: #e9c88d; }',
    '.hljs-title, .hljs-built_in, .hljs-type { color: #a9d7dd; }',
    'hr { border: 0; border-top: 1px solid #d7ded5; }',
    'a { color: #3f7251; font-weight: 550; text-decoration: underline; text-underline-offset: 0.16em; pointer-events: none; }',
    'math { max-width: 100%; font-size: 1.08em; }',
    '.twill-math--block { display: block; max-width: 100%; overflow-x: auto; text-align: center; }',
    '.twill-field > :first-child { margin-top: 0; }',
    '.twill-field > :last-child { margin-bottom: 0; }',
    '.twill-title { color: #172019; font-size: 1.3rem; font-weight: 700; line-height: 1.3; }',
    '.twill-media-image { margin: 1.25rem 0; }',
    '.twill-media-image img { display: block; max-width: 100%; height: auto; margin: 0 auto; border-radius: 0.65rem; }',
    '.twill-media-image--error { padding: 1.25rem; color: #7b302f; background: #fff3f2; border: 1px solid #efcfcc; border-radius: 0.65rem; text-align: center; }'
  ].join( '' );
}
