export const codeLanguageItems = [
  { label: 'Plain text', value: 'plaintext', aliases: 'text txt' },
  { label: 'Bash', value: 'bash', aliases: 'sh zsh' },
  { label: 'CSS', value: 'css' },
  { label: 'HTML / XML', value: 'xml', aliases: 'html xhtml rss atom xjb xsd xsl plist svg' },
  { label: 'JavaScript', value: 'javascript', aliases: 'js jsx mjs cjs' },
  { label: 'JSON', value: 'json' },
  { label: 'Markdown', value: 'markdown', aliases: 'md mkdown mkd' },
  { label: 'Python', value: 'python', aliases: 'py gyp' },
  { label: 'Rust', value: 'rust', aliases: 'rs' },
  { label: 'SQL', value: 'sql' },
  { label: 'TypeScript', value: 'typescript', aliases: 'ts tsx mts cts' }
];

export function normalizeCodeLanguage( language ) {
  const value = typeof language === 'string' ? language.trim().toLowerCase() : '';

  return codeLanguageItems.find( item => item.value === value
    || item.aliases?.split( ' ' ).includes( value ) )?.value ?? 'plaintext';
}
