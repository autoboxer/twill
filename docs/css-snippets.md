# CSS snippets

CSS snippets are advanced appearance overrides stored in the local Twill
library. Each device controls which snippets are enabled. New snippets start
disabled and enabled snippets load after Twill's built-in styles.

Manage snippets in **Settings → Snippets**. Enabled snippets are applied in
case-insensitive name order, so rules in a later-named snippet win when their
specificity is equal. **Disable all** turns off every snippet without deleting
its source.

## Safety limits

Twill parses snippet source before saving it. Version 1 accepts ordinary rules,
custom properties, conditional rules, layers, and keyframes. It rejects:

- `@import` and `@font-face`
- URL and other resource-loading functions, including `url()` and `image-set()`
- Legacy executable constructs such as `expression()` and `behavior`
- Malformed CSS and source larger than 100 KB

Validation prevents resource loading and executable legacy CSS. It cannot stop a
valid rule from hiding, moving, or resizing important controls. Keep a snippet
disabled while editing if the current source makes the interface difficult to
use.

## Stable variables

The following custom properties are Twill's supported theme contract:

- `--twill-background`
- `--twill-surface`
- `--twill-elevated-surface`
- `--twill-active-surface`
- `--twill-text`
- `--twill-muted-text`
- `--twill-dimmed-text`
- `--twill-border`
- `--twill-accent`
- `--twill-radius`
- `--twill-reading-font`
- `--twill-reading-text-size`

For example:

```css
:root {
  --twill-accent: #8b7cf6;
  --twill-radius: 0.25rem;
}
```

## Stable targets

Use the documented `data-twill-*` attributes for structural overrides. Twill's
classes are implementation details and may change.

- `[data-twill-app]` — Application shell
- `[data-twill-navigation]` — Primary navigation container
- `[data-twill-navigation-item]` — Primary navigation links
- `[data-twill-destination="study"]` — A specific navigation destination
- `[data-twill-page]` — Any routed page
- `[data-twill-page="study"]` — A specific routed page
- `[data-twill-study-card]` — Active study card
- `[data-twill-concept-card]` — Concept in the library
- `[data-twill-template-card]` — Template in the template library
- `[data-twill-editor-section]` — Concept or template editor section
- `[data-twill-settings-section]` — Settings section

Supported destination values are `study`, `library`, `create`, and `settings`.
Supported page values are `study`, `library`, `concept-editor`, `concept-detail`,
`templates`, `template-editor`, `settings`, and `startup`. Editor-section values
are `concept-basics`, `concept-content`, `concept-retrieval-forms`,
`concept-organization`, `template-basics`, and `template-design`.
Settings-section values are `general`, `appearance`, `snippets`, `study`, and
`scheduling`.

For example:

```css
[data-twill-page="study"] [data-twill-study-card] {
  border-color: color-mix(in srgb, var(--twill-accent) 45%, var(--twill-border));
}

[data-twill-settings-section="snippets"] {
  box-shadow: none;
}
```
