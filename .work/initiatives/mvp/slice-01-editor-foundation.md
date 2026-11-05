# Slice 1 — editor foundation

Status: Planned

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

Pending implementation: launch command, checks/results, terminal/platform, and limitations.
