# Character discovery

Status: squash-merged into `main` as `df26015`; included in v0.2.0.

## Delivered

- Bundled Unicode 17.0.0 and Nerd Fonts 3.4.0 metadata; 18,696 curated candidate scalars before runtime display admission.
- Eleven meaningful top-level groups with design-focused types, a dedicated Emoji collection, and icon families; a scrollable Group dropdown with type-to-jump, and expanded-by-default collapsible type sections.
- Emoji categories and glyphs follow CLDR picker order, with exact search matches retaining priority.
- Stable scalar identities; aliases merged without duplicate result cells.
- Offline name/alias/glyph/code-point search, optional group restriction, explicit Clear.
- Configurable Characters-only Ctrl+F; shared navigation, field editing, clipboard, and activation bindings.
- A fluid grid wrapping to the available width with one-cell gaps, filling available vertical space, independent scrollbar/wheel/keyboard navigation, and one-line character identification.
- A stable 23-row minimum block height; expansion into unused space; tool focus preserved across wide/slim transitions.
- Existing selection replacement, typing style, repeat insertion, and atomic undo semantics.
- Controls, marks, invisible formatting, unassigned characters, and unsupported emoji components excluded; runtime admission also checks conservative renderer behavior.
- Search fields use the document renderer's conservative grapheme projection without modifying query bytes.
- Reproducible generator, checksummed upstream inputs, retained notices, user and maintenance guides.

## Validation

- `cargo fmt --check`: pass.
- `cargo test --locked`: 199 passed, one existing ignored test.
- `cargo clippy --locked --all-targets -- -D warnings`: pass.
- `uv run scripts/terminal_smoke.py`: full regression walkthrough passed.
- `uv run scripts/characters_smoke.py`: search/focus/insertion/save/undo, large grids, unsupported exclusion, wide/slim resize, and terminal restoration passed.
- `python3 scripts/alacritty_smoke.py --characters`: actual Alacritty rendering, key input, selection styling, save, settings, resize, and clean exit passed; screenshots inspected.
- Catalog regeneration: byte-identical, checksum-verified; package inventory includes data, licenses, generator, and documentation.
- Release binary built successfully.
- Full catalog/presentation-variant cursor-width probe: 18,718 samples, zero disagreements after projection in bare Alacritty and tmux. See [rendering investigation](character-rendering.md).

## Boundaries

Font coverage remains terminal-dependent. Queries and browsing state are session-local.
Favorites/recents, user catalogs, font detection, and composed-emoji rendering remain deferred.
No new Cargo dependencies or configuration-file changes.
Character discovery, documentation media, and the measured symbol-width mitigation
are included in the v0.2.0 release.
