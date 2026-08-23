export const DEFAULT_APPEARANCE = Object.freeze({
  theme: 'aubergine',
  readingFont: 'inter',
  readingTextSize: 'medium',
  motionPreference: 'system'
});

export const themeOptions = [
  {
    value: 'aubergine',
    label: 'Aubergine',
    description: 'Soft violet on deep charcoal',
    mode: 'dark',
    preview: {
      background: '#0d0c12',
      surface: '#17141d',
      text: '#f0edf5',
      accent: '#a88ae8'
    }
  },
  {
    value: 'dracula',
    label: 'Dracula',
    description: 'Vivid purple with bright contrast',
    mode: 'dark',
    preview: {
      background: '#282a36',
      surface: '#44475a',
      text: '#f8f8f2',
      accent: '#bd93f9'
    }
  },
  {
    value: 'one-dark',
    label: 'One Dark',
    description: 'Atom\'s balanced developer palette',
    mode: 'dark',
    preview: {
      background: '#282c34',
      surface: '#3e4451',
      text: '#abb2bf',
      accent: '#c678dd'
    }
  },
  {
    value: 'tokyo-night',
    label: 'Tokyo Night',
    description: 'Cool blue with neon violet',
    mode: 'dark',
    preview: {
      background: '#1a1b26',
      surface: '#24283b',
      text: '#c0caf5',
      accent: '#bb9af7'
    }
  },
  {
    value: 'catppuccin-mocha',
    label: 'Catppuccin Mocha',
    description: 'Warm pastel accents on navy',
    mode: 'dark',
    preview: {
      background: '#1e1e2e',
      surface: '#313244',
      text: '#cdd6f4',
      accent: '#cba6f7'
    }
  },
  {
    value: 'nord',
    label: 'Nord',
    description: 'Calm arctic blues and frost',
    mode: 'dark',
    preview: {
      background: '#2e3440',
      surface: '#3b4252',
      text: '#eceff4',
      accent: '#b48ead'
    }
  },
  {
    value: 'gruvbox-dark',
    label: 'Gruvbox Dark',
    description: 'Retro warmth with earthy contrast',
    mode: 'dark',
    preview: {
      background: '#282828',
      surface: '#3c3836',
      text: '#ebdbb2',
      accent: '#d3869b'
    }
  },
  {
    value: 'solarized-dark',
    label: 'Solarized Dark',
    description: 'Low-glare teal and balanced accents',
    mode: 'dark',
    preview: {
      background: '#002b36',
      surface: '#073642',
      text: '#839496',
      accent: '#6c71c4'
    }
  },
  {
    value: 'github-light',
    label: 'GitHub Light',
    description: 'Crisp, familiar code-hosting clarity',
    mode: 'light',
    preview: {
      background: '#ffffff',
      surface: '#f6f8fa',
      text: '#1f2328',
      accent: '#8250df'
    }
  },
  {
    value: 'one-light',
    label: 'One Light',
    description: 'Clean Atom-inspired neutrals',
    mode: 'light',
    preview: {
      background: '#fafafa',
      surface: '#f0f0f0',
      text: '#383a42',
      accent: '#a626a4'
    }
  },
  {
    value: 'catppuccin-latte',
    label: 'Catppuccin Latte',
    description: 'Soft lavender on a cool canvas',
    mode: 'light',
    preview: {
      background: '#eff1f5',
      surface: '#e6e9ef',
      text: '#4c4f69',
      accent: '#8839ef'
    }
  },
  {
    value: 'gruvbox-light',
    label: 'Gruvbox Light',
    description: 'Warm paper with retro accents',
    mode: 'light',
    preview: {
      background: '#fbf1c7',
      surface: '#ebdbb2',
      text: '#3c3836',
      accent: '#8f3f71'
    }
  },
  {
    value: 'solarized-light',
    label: 'Solarized Light',
    description: 'Cream canvas with calibrated color',
    mode: 'light',
    preview: {
      background: '#fdf6e3',
      surface: '#eee8d5',
      text: '#586e75',
      accent: '#6c71c4'
    }
  },
  {
    value: 'rose-pine-dawn',
    label: 'Rosé Pine Dawn',
    description: 'Gentle rose tones on warm paper',
    mode: 'light',
    preview: {
      background: '#faf4ed',
      surface: '#f2e9e1',
      text: '#575279',
      accent: '#907aa9'
    }
  }
];

export const readingFontOptions = [
  { value: 'inter', label: 'Inter', description: 'Clean and neutral' },
  { value: 'systemUi', label: 'System UI', description: 'Matches your device' },
  {
    value: 'ibmPlexSans',
    label: 'IBM Plex Sans',
    description: 'Technical and readable'
  },
  {
    value: 'sourceSerif4',
    label: 'Source Serif 4',
    description: 'Comfortable long-form serif'
  },
  {
    value: 'jetBrainsMono',
    label: 'JetBrains Mono',
    description: 'Developer-friendly monospace'
  }
];

export const readingTextSizeOptions = [
  { value: 'small', label: 'Small', detail: '15 px' },
  { value: 'medium', label: 'Medium', detail: '17 px' },
  { value: 'large', label: 'Large', detail: '20 px' }
];

export const motionOptions = [
  {
    value: 'system',
    label: 'System',
    description: 'Follow your device preference'
  },
  { value: 'full', label: 'Full', description: 'Use interface animations' },
  { value: 'reduced', label: 'Reduced', description: 'Limit interface motion' }
];

const allowedValues = {
  theme: new Set( themeOptions.map( ( option ) => option.value ) ),
  readingFont: new Set( readingFontOptions.map( ( option ) => option.value ) ),
  readingTextSize: new Set(
    readingTextSizeOptions.map( ( option ) => option.value )
  ),
  motionPreference: new Set( motionOptions.map( ( option ) => option.value ) )
};

export function normalizeAppearance( value = {}) {
  return Object.fromEntries(
    Object.entries( DEFAULT_APPEARANCE ).map( ([ key, fallback ]) => [
      key,
      allowedValues[ key ].has( value[ key ]) ? value[ key ] : fallback
    ])
  );
}
