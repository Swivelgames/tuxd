# Theme presets

Public attribution and retained license texts are maintained in [THIRD_PARTY_NOTICES.md](../../../THIRD_PARTY_NOTICES.md).

Midnight is the default and fallback. Baseline is removed with no migration or compatibility alias. Explicit color overrides remain independent of preset selection. The compact preset chooser previews highlighted entries; Enter/click accepts, Esc cancels. It supports arrows, j/k, Home/End, PageUp/PageDown and mouse-wheel navigation, with a scrollbar when needed.

| Presets | Published palette source |
| --- | --- |
| Midnight, Light | Original tuxd palettes |
| One Dark | [Atom One Dark Syntax](https://github.com/atom/one-dark-syntax/blob/master/styles/colors.less) |
| Dracula | [Dracula OSS palette](https://github.com/dracula/dracula-theme#color-palette-oss) |
| Nord | [Nord CSS palette](https://github.com/nordtheme/nord/blob/develop/src/nord.css) |
| Gruvbox Dark / Light | [Gruvbox palette](https://github.com/morhetz/gruvbox/blob/master/colors/gruvbox.vim) |
| Solarized Dark / Light | [Ethan Schoonover’s Solarized](https://ethanschoonover.com/solarized/) |
| Tokyo Night | [Night background](https://github.com/folke/tokyonight.nvim/blob/main/lua/tokyonight/colors/night.lua), [shared palette](https://github.com/folke/tokyonight.nvim/blob/main/lua/tokyonight/colors/storm.lua) |
| Catppuccin Latte / Frappé / Macchiato / Mocha | [Catppuccin palette](https://github.com/catppuccin/palette/blob/main/palette.json) |

These are tuxd role mappings of the published palettes, not reproductions of another editor's entire highlighting scheme. Core background, text and accent colors are preserved. Secondary text is chosen from legible foreground shades rather than blindly adopting low-contrast comment colors. Selected backgrounds and syntax roles use related palette entries. The document canvas, actual ANSI swatches, and RGB channel indicators remain independent of the UI palette.

The strict text contrast tests cover tuxd's original Midnight and Light presets. Third-party palettes retain their characteristic contrast relationships; these are not advertised as accessibility-certified variants.

Validation covers every preset's resolution and serialization, the absence of Baseline, preview/accept/cancel, overrides and reset, picker bounds at narrow sizes, save/restart, and the existing full terminal walkthrough. The wide-glyph redraw test compares visible cells: a wide glyph's hidden continuation cell can retain TestBackend style metadata without any independent terminal rendering.
