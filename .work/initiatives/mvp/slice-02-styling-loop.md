# Slice 2 — first complete styling loop

Status: Complete within the declared scope

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
- Independent F3/clickable soft-wrap toggles for editor and code, with always-on logical line numbers and blank continuation gutters. Both gutters highlight the editor cursor’s active logical line with a background change and bold number; continuation rows share the background.
- Ctrl+C in the focused code pane copies escaped ANSI without presentation-only newlines or gutters, with asynchronous helper execution and explicit success/failure feedback.
- Ctrl/Alt+Left/Right moves by Unicode word boundaries; Shift extends selection. Ctrl/Alt+Backspace deletes a word or selection as one undo transaction.
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

Background/extended colors, additional attributes, glyph palettes, persistence, file export actions, and code highlighting.

## Completion evidence

Implemented on `feat/2-styling-loop`. Launch with `cargo run --locked`.

Agreed behavior:

- Insertion inherits the preceding style, with defaults at document start; tool choices override until cursor movement.
- Replacement inherits the first selected character's style and is one undo transaction.
- Esc clears selection at the active cursor; Left/Right collapses selection to that edge.
- Ctrl+B/I/U applies styles, F6 cycles focus, and F2 toggles the initially collapsed code drawer.
- Eight ANSI foreground colors plus Default appear in the right panel. Narrow windows provide tools through F6.
- Every visible `\n` is followed by a real display line break. Tabs remain visible as `\t`. Raw generated ANSI is independent of the display representation.

Verification:

- 32 Rust tests cover the prior editor behavior plus selection, mixed formatting, boundary inheritance, style range normalization, grapheme merging, replacement undo, rendered attributes, exact ANSI/display/clipboard output, word operations, wrap/resize coordinates, logical-line gutters, and synchronized active-line indicators.
- Formatting and Clippy checks pass.
- Linux PTY checks pass for keyboard selection, bold/italic/underline/cyan, multiline code display, reverse mouse dragging, replacement undo, narrow tools, per-pane wrap, gutters, word bindings, clipboard success/failure through isolated helpers, and all Slice 1 input/cleanup scenarios.
- Terminal cleanup remains checked for normal exit, returned errors, and panics.

Limitations: drag selection clamps at viewport edges without autoscrolling; attributes depend on terminal/font support. Ctrl+I needs distinct keyboard reporting, with F6 tools as a fallback. Very small terminals show the resize prompt. Saving, file export, and code highlighting remain deferred. Clipboard copy uses wl-copy on Wayland or xclip/xsel on X11 and reports unavailable helpers/display connections. Colors are explicitly enabled for this design preview even when NO_COLOR is set.
