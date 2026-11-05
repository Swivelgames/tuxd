# Slice 3 — complete initial tool panel

Status: Planned

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

## Completion evidence

Pending implementation: launch command, checks/results, terminal/platform, and limitations.
