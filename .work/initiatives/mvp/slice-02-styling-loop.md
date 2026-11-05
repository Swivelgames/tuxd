# Slice 2 — first complete styling loop

Status: Planned

Depends on: [Slice 1](slice-01-editor-foundation.md)

Estimate: 3–5 developer days, including validation.

## Usable checkpoint

Select text and visibly bold, italicize, or underline it. Apply a basic ANSI foreground color and see live escaped ANSI in a collapsible drawer. Editing and undo/redo preserve formatting correctly.

## Deliverables

- Linear mouse-drag and Shift+arrow selection, including multiline selection and reverse dragging.
- Shared hit testing that accounts for editor bounds, scrolling, tabs, and grapheme display width; dragging outside the editor clamps safely to the viewport edge. Drag autoscroll can follow later.
- Selection survives clicking tools. Tool focus does not overwrite the saved editor cursor or selected range.
- Bold, italic, underline, and basic ANSI foreground controls, accessible by mouse and keyboard.
- Normalized style ranges, mixed-selection state, pending typing styles, and a documented style-inheritance rule at insertion boundaries.
- Attribute changes preserve unrelated properties. Mixed toggles enable the attribute across the selection; fully enabled toggles disable it.
- Replacing a selection and inserting/deleting across style boundaries update both text and styles in one undoable transaction.
- Live read-only escaped ANSI output for the whole document; toggling its drawer preserves editor state.
- Deterministic ANSI generation using explicit resets/reapplication where needed. Begin from a defined default state and end with a reset so styling does not leak into following output.

Selection highlighting is a temporary UI overlay, never document formatting or exported output. Provide an obvious way to clear selection and inspect the actual colors/styles.

## Stability and verification

Test style application/removal, mixed selections, overlapping ranges, edits at boundaries, multiline replacement, and text/style undo/redo. Test foreground changes independently of attributes.

Verify exact ANSI output for plain and styled text, transitions back to defaults, and trailing resets. Cover Unicode mouse-position mapping and drawer resize behavior. Re-run slice 1's walkthrough.

## Hands-on acceptance

1. Type a sentence and drag-select a word.
2. Apply bold, italic, and underline; clear the selection and see the result in a terminal supporting those attributes.
3. Select again and apply cyan without losing the attributes.
4. Select a mixture of styled and plain text; check mixed indicators and toggle behavior.
5. Use Shift+arrows to select across a newline and apply formatting.
6. Insert/delete text at a style boundary and undo/redo both text and style changes.
7. Open the code drawer; verify it updates as formatting changes and displays literal escape notation.
8. Close the drawer and continue editing at the preserved position.

## Deferred

Background/extended colors, additional attributes, glyph palettes, persistence, copy/export actions, and code highlighting.

## Completion evidence

Pending implementation: launch command, checks/results, terminal/platform, and limitations.
