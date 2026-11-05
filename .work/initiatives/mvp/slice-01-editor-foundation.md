# Slice 1 — editor foundation

Status: Complete within the declared scope; broader desktop-terminal visual checks remain future compatibility work.

Depends on: None

Estimate: 2–3 developer days, including validation.

## Usable checkpoint

Launch the application and play around in a large, working text box. Typing, editing, navigation, scrolling, paste, and undo/redo are stable within the supported scope. No formatting is required yet.

## Deliverables

- Rust application with Ratatui/Crossterm, a synchronous event loop, and documented launch instructions.
- Large editor area, basic status line, and space reserved for later tools. Avoid enabled-looking controls without behavior.
- Insert text/newlines, Backspace/Delete, arrows, Home/End, and vertical/horizontal scrolling that keeps the cursor visible.
- Bracketed multiline paste and a documented policy for tabs and control characters. Preserve tabs in the document and account for tab stops in layout; do not execute embedded escape sequences.
- Text undo/redo with predictable transaction boundaries; pasted text is one transaction.
- Mouse click-to-position and wheel scrolling. Drag selection remains in Slice 2.
- Unicode-aware cursor/deletion boundaries and display-column mapping. Start with an explicit supported behavior for combining characters and wide glyphs.
- Resize handling, a usable small-window fallback, keyboard help, and a clear quit action.
- Terminal restoration on exit and error paths, including panic cleanup where feasible.

## Stability and verification

Test insertion, deletion, newline joining/splitting, undo/redo, paste grouping, and coordinate mapping for representative ASCII, combining, wide, and tabbed text. Check that edits never split UTF-8 or a supported grapheme. Verify viewport clamping on resize and deletion.

Run the app in the primary development terminal. Exercise normal exit and a controlled error path, confirming that echo, cursor visibility, and screen state are restored.

## Hands-on acceptance

1. Launch using the documented command into an empty editor.
2. Type several lines; navigate back to the middle and insert/delete text.
3. Paste multiline text and undo/redo the paste.
4. Enter a long line and enough lines to scroll in both directions.
5. Edit text containing an accented combining sequence, a wide character, and a tab without corrupting it.
6. Resize smaller and larger; continue editing without losing text or the cursor.
7. Quit and use the shell normally.

## Deferred

Selection, rich styling, code output, persistence, clipboard copy, and tool interactions arrive in later slices. Unsaved text is session-local; help must make that clear. Soft wrapping and large-file optimization are excluded.

## Completion evidence

Implemented on branch `feat/1-editor-foundation`.

- Launch: `cargo run --locked`; user controls and limitations are documented in the root [README](../../../README.md).
- `cargo test --locked`: 14 passing behavior/rendering tests, including Unicode boundaries, multiline mutations, mixed edit histories, undo grouping, viewport resizing, and modal dialogs.
- `cargo fmt --check` and `cargo clippy --locked --all-targets -- -D warnings`: passing.
- `uv run scripts/terminal_smoke.py`: passing actual PTY input checks for typing, navigation, mouse positioning/wheel scrolling, Unicode, atomic multiline paste, undo/redo, help, resizing, quit cancellation, and clean exit.
- The same smoke suite verifies termios restoration and mouse/paste/cursor/alternate-screen cleanup on normal exit, intentional returned errors, and intentional panics.
- Interactive tmux capture checks: editor text/Unicode, help, discard confirmation, and cancel passed.
- Environment: Linux, xterm-256color PTY and tmux, Rust 1.95.0. No outer desktop terminal/font visual parity claim is made.

Agreed decisions: conventional editing, modern Linux support, four-column literal tabs, grouped typing/deletion with a 750 ms pause, atomic paste, Ctrl+Z/Y/Q and F1, nonempty-document discard confirmation, and maximum editor space until tools become functional. Proper Vim mode is recorded as future work in the concept.

Known limits: session-only text, no selection/styles, no soft wrapping, 200 snapshot-based undo transactions, and terminal-dependent glyph widths. Standalone zero-width graphemes display with a dotted-circle placeholder; partially clipped wide glyphs display as blanks at viewport edges. Small windows show a resize prompt and retain the document. These limits are intentional within Slice 1.
