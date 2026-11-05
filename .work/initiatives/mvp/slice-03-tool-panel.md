# Slice 3 — complete initial tool panel

Status: Complete

Depends on: [Slice 2](slice-02-styling-loop.md)

Estimate: 2–3 developer days, including validation.

## Usable checkpoint

Compose a small styled terminal snippet using foreground/background colors, all initial palettes, and special-character insertion. Every visible tool performs its intended operation and participates in undo/redo where it changes the document.

## Deliverables

- Strike-through, dim, and clear-formatting controls alongside existing attributes.
- Foreground/background radio selection.
- ANSI 8/16 palette with a basic-eight/bright-colors distinction, indexed 256 palette, and RGB tab with hex and numeric components.
- A terminal-default color choice for each channel. This differs from selecting an explicit black or white value.
- Color validation with local feedback; incomplete or invalid input leaves the document unchanged.
- Box-drawing, block, and Nerd glyph tabs. Start with useful curated sets rather than a complete glyph catalog.
- Character insertion at the saved cursor or replacement of the selection, using the editor's typing-style rule.
- Keyboard navigation and mouse hit testing for all tools; narrow layouts expose the panel through a toggle.
- Short help explaining terminal-dependent attributes, ANSI palette colors, true color, and Nerd font coverage.
- ANSI generation extended to every introduced style and color mode.

Switching a palette tab or the foreground/background target changes only the tool view. A document mutation occurs when a value is applied. Clear formatting resets both color channels and attributes for the selection, or the pending typing style when no text is selected.

## Stability and verification

Test both color channels in all modes, default resets, RGB boundaries and invalid values, clear formatting, and undo/redo. Cover combinations such as bold plus dim when removing one attribute, since terminal reset semantics may affect both.

Check glyph replacement/insertion with wide or unsupported glyphs and confirm the buffer remains valid. Verify keyboard focus and editor selection survive palette interaction. Re-run earlier acceptance workflows.

## Hands-on acceptance

1. Select text, choose a foreground from ANSI colors, and a background from the indexed palette.
2. Apply an RGB value; enter an invalid value and confirm the text remains unchanged.
3. Reset one color channel to default without changing the other.
4. Apply dim/strike-through, then clear formatting and undo it.
5. Insert box corners and lines, a block, and a Nerd glyph; repeat insertion over a selection.
6. Reach the same tools by keyboard and use them after narrowing the terminal.
7. Inspect ANSI for each color mode and verify undo/redo restores text and styles.

## Deferred

Full glyph search/catalogs, font installation, automatic capability guarantees, visual RGB pickers, persistence, and external output actions.

## Implementation preparation

Prepared on `feat/3-tool-panel`, based on Slice 2 squash-merge `d7d6128`. Implementation is complete. Existing selection, undo/redo, wrapping, gutters, and clipboard behavior remain regression requirements.

Replace the foreground-only color representation with a document color type that distinguishes terminal default, ANSI basic/bright, indexed, and RGB values, independently for foreground and background. Separate semantic tool actions from their current numeric UI indices so palette sizes and tab navigation do not determine document operations. Rendering and ANSI generation must consume the same style model.

Implement in usable increments: expanded style actions and color model; foreground/background and ANSI palette; indexed/RGB controls; then curated character palettes. Validate each increment through the existing editor/code workflow.

Agreed interaction decisions:

- Keep compact stacked Style, Color, and Characters sections, with scrolling and a modestly wider sidebar where space permits. Retain narrow-terminal access through F6.
- Show basic and bright rows together in the ANSI 8/16 tab; keep 256 and RGB as separate tabs. Target/mode switches do not apply a color.
- Apply swatches immediately. Stage RGB hex/component edits until Apply or Enter; invalid input does not change the document. Show the candidate color before applying.
- Start with curated box, block, and Nerd glyph sets. Show the selected glyph's name/code point. Do not attempt font installation or automatic font detection.
- Keep keyboard focus in the character palette after insertion for repeated use; Esc returns to the editor. Each insertion/replacement is one undo step.

Customizable application themes are recorded as future work in the initial concept; document color controls in this slice do not change the application's UI theme.

## Completion evidence

Implemented on `feat/3-tool-panel`. Run with `cargo run --locked` in a modern Linux terminal.

- Added semantic style actions, independent default/ANSI/indexed/RGB color channels, dim, strike-through, clear formatting, and matching ANSI serialization.
- Added separate, scrollable and collapsible Style / Color / Characters boxes, basic/bright ANSI rows, all 256 indexed swatches, synchronized staged RGB fields, candidate preview, and validation feedback.
- Added curated Box / Blocks / Nerd glyphs with names/code points and independent undo transactions. Existing selection, replacement style inheritance, and focus behavior are preserved.
- Editor wrap, Code wrap, and Code visibility now default to on, per the user's refinement. F2/F3 and clickable toggles still work independently.
- Added sidebar keyboard navigation (Tab/Shift+Tab, arrows, Home/End, Page Up/Down), mouse hit areas/wheel scrolling, collapsible boxes, compact Ratatui Tabs strips, spaced glyph grids, distinct selection/focus treatments, and narrow-terminal overlay access via F6.

Verification on Linux:

- `cargo fmt --check` and `git diff --check`: pass.
- `cargo test --locked`: 42 tests pass, including both color channels, ANSI transitions, independent bold/dim resets, RGB validation/staging, clear/undo, Unicode glyph replacement, mouse selection preservation, and keyboard reachability of every palette control at 80×24, 40×12, and 20×5.
- `cargo clippy --locked --all-targets -- -D warnings`: pass.
- `uv run scripts/terminal_smoke.py`: pass. Actual PTY input covers earlier editing/styling/clipboard workflows plus startup defaults, dim/strike controls, indexed background, staged and invalid RGB, repeated narrow-overlay glyph insertion and undo. Normal/error/panic terminal restoration passes.

Limitations: no persistence yet; colors and attribute appearance depend on terminal capabilities and palette; Nerd glyph coverage depends on the installed font. RGB fields use replace-on-focus and end editing, not a full secondary text-editor widget. Glyph catalogs are curated. Themes remain future work. Desktop font rendering has not been certified by the PTY checks.

UI refinement: removed the enclosing Tools border and verbose per-glyph labels. Each tool now has its own padded border and collapse control. Tabs use padded labels and dot separators; foreground/background radios separate selected state from keyboard focus. Glyph names/code points occupy one detail line beneath the grid. Collapse and view changes preserve selection and do not enter document history.

Further UI refinement: compact styled B/I/U/D/S buttons, two rows of eight ANSI swatches with Reset beneath, shared terminal-default block backgrounds, and rounded borders throughout Editor, Code, tools, and popups.

Final spacing polish: collapsed boxes use two rows with no empty interior; tab dividers share the border color; title styling stops before the following border cell. Swatch highlights use uniform widths, Reset is bold without a background, glyph highlights fit the displayed character, and style buttons have balanced padding with Clear right-aligned.

Palette grid refinement: Basic uses equal gaps between its eight fixed-width swatches; indexed colors use six fixed-width swatches per row with right-aligned numbers. Both grids start at the same left inset, and Reset follows the final (possibly partial) row.

Indexed palette viewport: seven columns, right-aligned numbers, one-space gaps, and five visible swatch rows. Its wheel scroll is independent; keyboard focus reveals hidden swatches. Basic uses two-space gaps. The final partial indexed row and stable Characters position are regression-covered.

Scrollbar refinement: overflow-only bars cover Editor/Code vertical scrolling, unwrapped horizontal scrolling, the outer tool sidebar, and the independent indexed-color viewport. Track clicks, thumb dragging, and wheel input share viewport bounds. Regression coverage drags every scrollbar to its end while checking document, cursor, selection, and undo preservation.

Sidebar layout correction: reserve a dedicated outer scrollbar column without overwriting tool borders, and keep a one-cell inset before both swatch grids. The wide sidebar includes enough room for eight Basic swatches with two-space gaps.

RGB form refinement: bounded hex field and separate candidate swatch, three adjacent interactive component gauges, and Apply/Reset on a shared row. Mouse click/drag/wheel and keyboard component adjustments remain staged; Enter/Apply commits one undoable change. Invalid input still provides feedback without document mutation.
