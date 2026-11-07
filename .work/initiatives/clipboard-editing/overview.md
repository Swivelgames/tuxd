# Editor clipboard editing

Status: Complete — automated verification and user manual acceptance passed

## Scope and agreed behavior

- Copy (Ctrl+C) and Cut (Ctrl+X) serialize the editor selection as escaped ANSI, including styles and surrounding resets. No selection means no action.
- Select all (Ctrl+A) shares the existing configurable action with text fields.
- Paste (Ctrl+V) and terminal bracketed paste automatically recognize raw ANSI controls or unescaped `\x1b[` notation. Ordinary `\n`, `\t`, and Windows paths alone remain literal.
- Paste without formatting decodes ANSI and applies the editor's current typing style.
- Paste literal text skips ANSI decoding, preserving escaped notation as text. Real control characters still use the editor's existing safe visible notation, and CR/CRLF follow existing newline normalization.
- Both additional paste actions start unbound and can be configured under Workspace → Text Fields.
- Styled paste preserves imported styles; subsequent typing uses the style active before paste. Each paste replaces the selection or inserts at the cursor and is one undoable edit.
- Malformed/unsupported controls reuse the import preview and error table with Strip and paste / Abort. Ambiguous unterminated control strings are rejected by the existing parser.
- Cut deletes only after the clipboard transport reports success. SSH/tmux forwarding does not acknowledge client delivery; the user accepted this limit. Undo restores a successful cut.
- Delayed cut/paste results cannot mutate a different document, changed document revision, cursor, selection, or typing style. Opening another document cancels pending editor mutations. Clipboard failure preserves editor content.

## Scope limits

Style-only copy/paste (Paste formatting) remains deferred. Clipboard reads retain the existing desktop helper support; SSH users can use terminal-side paste. This work does not add remote clipboard querying. Automatic escaped-ANSI detection is intentionally conservative and can interpret literal examples as ANSI; Paste literal text is the explicit bypass.

## Verification

Core regressions cover styled Unicode selection round trips, one-step undo/redo, typing style preservation, raw/escaped/plain/literal paste, import review accept/abort, failed/stale cuts and stale paste results.

`uv run scripts/editor_clipboard_smoke.py` exercises actual editor keybindings, a clipboard helper isolated from the desktop, cut/undo, styled replacement, review accept/abort, saved document equality and terminal cleanup.

## Manual acceptance

1. Select styled text across lines, copy, and paste at a new location; verify styles and undo/redo.
2. Cut a selection, then undo; test no-selection copy/cut as well.
3. Bind Paste without formatting and Paste literal text in Shortcut Settings. Paste escaped ANSI with each and compare the results.
4. Paste an unsupported sequence, inspect the review, then exercise Abort and Strip and paste.
5. Use terminal-side paste over SSH and confirm the same automatic detection behavior.

## Verification results

- Rust tests: 109 passed; one performance measurement intentionally ignored.
- Formatting, Clippy with warnings denied, and diff whitespace checks passed.
- Editor clipboard smoke: styled copy/cut/paste, undo, review abort/accept, exact saved data and cleanup passed.
- Full terminal smoke suite passed, including persistence, shortcuts, field clipboard actions and normal/error/panic restoration.
- Direct SSH and tmux forwarding checks passed with exact payload comparisons and failure feedback.

Existing smoke fixtures were updated where they deliberately pasted escape-looking text: automatic ANSI decoding is now the expected behavior, while literal cases use the proper escaped representation.

The user confirmed the clipboard workflow works well and accepted the implementation for squash-merge.
