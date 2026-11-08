# Third-party notices

Original tuxd code and documentation are licensed under [GPL-3.0-only](LICENSE). Third-party material retains its upstream notices and licenses.

## Theme palettes

The theme implementations in `src/libraries/themes/presets.rs` adapt published palette colors to tuxd’s interface roles. They are not copies of entire editor themes. tuxd changes role mappings, secondary text, selection colors, and syntax roles.

| tuxd presets | Upstream project | Upstream license / retained notice |
| --- | --- | --- |
| Midnight, Light | Original tuxd palettes | GPL-3.0-only |
| One Dark | [Atom One Dark Syntax](https://github.com/atom/one-dark-syntax) | [MIT](licenses/themes/one-dark.txt) |
| Dracula | [Dracula Theme](https://github.com/dracula/dracula-theme) | [MIT](licenses/themes/dracula.txt) |
| Nord | [Nord](https://github.com/nordtheme/nord) | [MIT](licenses/themes/nord.txt) |
| Gruvbox Dark / Light | [Gruvbox](https://github.com/morhetz/gruvbox) by Pavel Pertsev | [MIT/X11](licenses/themes/gruvbox.txt) |
| Solarized Dark / Light | [Solarized](https://github.com/altercation/solarized) by Ethan Schoonover | [MIT](licenses/themes/solarized.txt) |
| Tokyo Night | [Tokyo Night](https://github.com/folke/tokyonight.nvim) by folke | [Apache-2.0](licenses/themes/tokyo-night.txt) |
| Catppuccin variants | [Catppuccin Palette](https://github.com/catppuccin/palette) | [MIT](licenses/themes/catppuccin.txt) |

[Pinned notice sources](licenses/themes/sources.json) record the upstream revisions reviewed. Gruvbox declares MIT/X11 in its README and MIT in its package metadata, rather than providing a standalone license file; its retained notice identifies that source and author.

## Character metadata

| Data | Source | Retained license |
| --- | --- | --- |
| Unicode names, aliases, categories, blocks, and properties | [Unicode Character Database 17.0.0](https://www.unicode.org/Public/17.0.0/ucd/) | [Unicode License v3](licenses/characters/UNICODE.txt) |
| Emoji categories and names | [Unicode Emoji 17.0](https://www.unicode.org/Public/17.0.0/emoji/emoji-test.txt) | [Unicode License v3](licenses/characters/UNICODE.txt) |
| Nerd Font icon names and code points | [Nerd Fonts 3.4.0](https://github.com/ryanoasis/nerd-fonts/tree/v3.4.0) | [Upstream license, including MIT for source files](licenses/characters/NERD-FONTS.txt) |

[Pinned data sources](licenses/characters/sources.json) and the [catalog maintenance guide](docs/character-catalog.md) describe regeneration. tuxd redistributes derived metadata, not font binaries or chart PDFs.

## Rust dependencies

[Dependency notices](licenses/dependencies.txt) contain each locked package’s version, upstream license declaration, source link, and retained license/copyright texts. The inventory includes build dependencies and packages for other target platforms; it is broader than any one compiled binary.

| Notable component | Upstream project | Declared license |
| --- | --- | --- |
| Terminal rendering | [Ratatui](https://github.com/ratatui/ratatui) | MIT |
| Terminal events | [Crossterm](https://github.com/crossterm-rs/crossterm) | MIT |
| File explorer | [ratatui-explorer](https://github.com/tatounee/ratatui-explorer) | MIT |
| Shortcut resolver | [ratatui-which-key](https://crates.io/crates/ratatui-which-key/0.14.0) | LGPL-3.0-or-later |

Some published packages omit standalone license files. [Supplement sources](licenses/dependency-supplements/sources.json) record retained upstream texts, using the package’s VCS revision where available. The Windows winapi support packages lack packaged VCS metadata; their supplements identify the upstream revision reviewed instead.

Regenerate the inventory after dependency changes:

```sh
cargo fetch --locked
python3 scripts/update_dependency_notices.py
```

The generator runs offline and fails if a package lacks notice material. Review new declarations and supplements rather than treating generation as legal approval.

## Distribution

When packaging tuxd, include `LICENSE`, this file, and `licenses/` alongside the executable and provide the corresponding source and build instructions for that release. Preserve upstream copyright and permission notices. The notice bundle alone does not satisfy source-distribution obligations.

Review the retained LGPL terms for `ratatui-which-key` when preparing binary distributions, including how recipients can rebuild with a modified dependency. Do not describe every dependency as GPL or assume tuxd’s top-level license replaces upstream terms.

## Documentation media

The screenshots and GIFs under `docs/assets/` were captured from tuxd with the included showcase document. They are original project media. No terminal fonts or third-party screenshots are redistributed with them.
