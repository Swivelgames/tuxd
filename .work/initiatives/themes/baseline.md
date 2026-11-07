# Baseline application theme

The first themes step consolidates the existing appearance in `src/theme.rs`. It does not add theme selection, persistence, or a customization dialog.

## Shared roles

| Role | Baseline | Uses |
| --- | --- | --- |
| Text / surface | Terminal defaults | Base text and panel surfaces |
| Accent | Terminal cyan | Titles, active tabs, controls, scrollbar thumbs |
| Border | #3e4e6a | Rounded panel/dialog borders and scrollbar tracks |
| Muted | #8896b3 | Secondary text, inactive controls, gutters, status |
| Field | #171c2c | Input fields, gutter surfaces, review tables |
| Selected | #293653 | Selected controls and active gutter rows |
| Selected text | #dce8ff | Active gutter text and selected tool labels |
| Focus | #354660 | Focused tool control, distinct from selected state |
| Inactive indicator | #364156 | Unselected radio dots |
| Error | Terminal red | Invalid values and unsupported controls |
| Selection overlay | Terminal dark gray | Editor selection, preserving document foreground |

## Deliberate differences

- Application colors are independent of document ink. ANSI palettes, color swatches, selected RGB previews, and black/white swatch labels retain their content-based values.
- RGB gauge fills retain red, green and blue hues, now defined centrally as channel roles.
- Syntax highlighting retains separate sequence, parameter, escape, separator and whitespace roles. These never affect exported ANSI.
- Focus and selection remain separate states. Underline, bold, and inverse modifiers continue to communicate state where colors alone are insufficient.
- Terminal-default surfaces and named ANSI colors follow the terminal palette; the baseline does not force a new background or replace the terminal's palette.

## Intentional consistency changes

- Settings muted text matches gutters and tool controls (#8896b3).
- File-dialog inactive radio dots match tool radio dots (#364156).
- Panel, menu, and dialog borders share the existing tool/settings border role.
- Invalid field feedback uses the same red role as import review and settings errors.
- Status text uses the shared muted role.

## Validation

Existing rendering tests preserve expected document styling, selection overlays, swatches and syntax highlighting. Run `cargo test --locked`, `cargo clippy --locked --all-targets -- -D warnings`, and `uv run scripts/terminal_smoke.py` for rendering and interaction regressions. User review remains the next step for the deliberate appearance adjustments.

## Runtime and persistence follow-up

The renderer now resolves roles through a scoped active Theme, restored after each render (including unwinding). Startup loads the baseline plus validated role overrides from the tuxd user config directory. Missing files are not created. Config provides validated atomic writes for the forthcoming Settings save action. UI surfaces follow the theme while the document canvas explicitly retains document/default colors.

Validation includes theme round-trip/error tests and `scripts/theme_smoke.py`, which checks custom UI colors, unchanged document colors/content, and non-destructive fallback. The interactive editor and additional presets remain outstanding.

## Unified Settings follow-up

Settings now groups Theme/Appearance and the existing Shortcuts categories. Appearance edits the baseline role overrides with valid-input live preview, field cancellation/reset, and a scrollable list. The entire settings session shares Save, Cancel, dirty handling and Reset all. Runtime activation occurs only after both writes succeed; normal shortcut-save failures restore the prior theme file and retain drafts. Cross-file crash atomicity is not promised. Additional presets remain out of scope for this checkpoint.

Acceptance: `scripts/settings_theme_smoke.py` covers preview/discard/save/restart; existing terminal and leader walkthroughs use the new category hierarchy. Unit coverage includes group expansion for context jumps and failed-save restoration with both existing and absent theme files.

## Appearance layout refinement

Appearance now follows the shortcut pages: expanded collapsible subsections, spacing between entries, normal labels, distinct color value fields, swatches and per-role Reset buttons. Roles are grouped by Interface, Controls, Editor & Feedback, Code Highlighting and RGB Controls. Narrow layouts stack controls below labels. Keyboard traversal skips spacing, supports heading collapse/expand, and reaches Reset through Tab or Right. Scrolling uses rendered row heights so the focused control remains visible.

## Built-in presets

Appearance now offers Baseline, Light and Midnight. Switching previews immediately and preserves explicit overrides. Reset theme clears only role overrides while retaining the selected preset and all shortcut edits. Per-role reset inherits the current preset. Save/Cancel and persistence use the existing draft flow.

Light and Midnight use explicit RGB UI palettes; Baseline retains terminal-relative colors. Document ink and the document canvas remain independent. File rows, inactive radio surfaces, and prompt backgrounds now follow the theme instead of resetting to the terminal palette.

Validation includes preset persistence, retained overrides, theme-only reset, text contrast checks for the explicit palettes, and the PTY walkthrough covering preview, cancellation, reset, unchanged shortcuts, save and restart. Terminal palette/font behavior still determines Baseline and document appearance.

## Default and catalog update

Midnight replaces the historical Baseline default. Baseline has been removed without migration. The [preset catalog](presets.md) records the expanded set, compact chooser, upstream palette sources, and adaptation policy. Earlier sections document prior checkpoints.
