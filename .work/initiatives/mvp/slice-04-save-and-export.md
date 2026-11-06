# Slice 4 — save, import, and export workflow

Status: Complete

Depends on: [Slice 3](slice-03-tool-panel.md)

Estimate: 3–5 developer days, including validation.

## Usable checkpoint

Create a styled document, save it, reopen it without losing formatting, import supported Raw/Escaped ANSI, and export ANSI for use outside the app. Failed or canceled operations preserve the active document and give actionable feedback.

Run `cargo run --locked -- examples/welcome.tuxd.json` to start with a styled example. Use F4 for the full file menu. `examples/import-review.txt` exercises Escaped ANSI import and its control-review prompt.

## Delivered behavior

- Versioned, tab-indented `.tuxd.json` files contain one document's text and normalized style ranges. Default, ANSI basic/bright, indexed, and RGB color identities are preserved. UI state and pending typing style are session state.
- New/Open/Save/Save As use a shared keyboard- and mouse-accessible directory picker. The filename and dirty marker appear in the editor title. An optional document path can be supplied at launch.
- Ctrl+N/O/S, Ctrl+Shift+S, Ctrl+E, and Ctrl+Q provide direct access; F4 is the fallback when terminals cannot distinguish modified shortcuts. Dialog actions can also be clicked.
- Save/Discard/Cancel protects unsaved documents before New/Open/import/Quit. Open/import candidates are fully parsed and validated before replacement. Successful replacement starts fresh history; saving preserves history.
- Dirty state compares the document against its saved snapshot. Undoing back to saved clears the marker. Imported documents are untitled and dirty, including empty imported documents. Saving an import asks for a document destination.
- Loading validates version, fields, style/color values, UTF-8/grapheme range boundaries, and complete contiguous coverage before changing the active document.
- Writes use a temporary file in the destination directory, flush its contents, and atomically replace the destination. Existing permissions are preserved on saves. Save As/export confirms existing destinations. Symlink/non-regular write targets are rejected.
- Raw ANSI export writes actual escapes and preserved text/newlines with a final style reset. Escaped export matches the Code clipboard payload without presentation-only line breaks. Export neither changes the document path nor clears dirty state; the active document file cannot be used as an export destination.
- Existing Code clipboard copy and its helper-unavailable feedback remain available; file export is the fallback.

## ANSI import contract

Import and Export default to Escaped ANSI; their dialogs provide explicit Raw/Escaped radio choices. Sources must be UTF-8. Escaped mode decodes the exported notation (`\x1b`, `\n`, `\t`, and escaped backslashes) before ANSI parsing. Ordinary Unicode and actual line breaks are preserved; CRLF is normalized to LF.

Supported SGR includes reset; bold, dim, italic, underline, strike-through; individual disable/reset codes; ANSI basic/bright colors; default colors; and semicolon-form indexed/RGB colors. Reset 22 clears both bold and dim. Imported styles follow the editor's leading-character rule when sequences fall within a grapheme.

Malformed escapes and unsupported controls are collected before applying the candidate. The Unsupported ANSI Sequences dialog shows a numbered, wrapped source preview with erroneous sequences in bold red, followed by a Seq./Error table. Both views show up to five rows and scroll independently. Diagnostics retain original source locations internally. Tab, Arrow/Page keys, the mouse wheel, and draggable scrollbars navigate the review.

Import anyway requires an explicit button activation or S key. It removes the listed sequences, preserving surrounding text and preceding supported style state. A sequence containing unsupported SGR parameters is reported and removed as a whole. A/Esc aborts without changing the document. Dirty-document protection still applies after accepting the candidate. Pasting response letters cannot answer confirmations.

Recovery stops at defined control/parameter boundaries; malformed escape prefixes do not swallow following Unicode. Unterminated OSC/DCS/control strings fail import because their boundary is ambiguous. Invalid UTF-8 is also an error. Input and diagnostics are never executed as terminal commands.

## Implementation

Implemented on `feat/4-save-and-export`, based on Slice 3 squash-merge `141ab74`.

- `persistence.rs`: version-1 schema, strict validation, bounded regular-file reads, atomic writes.
- `ansi_import.rs`: source-aware escape decoding, SGR parsing, candidate generation, and diagnostics.
- `files.rs`: file workflow state, pending replacements, dirty/save tracking, picker/confirmation/review dialogs, and modal input/rendering.
- `file_picker.rs`: shared `ratatui-explorer` directory browsing, location/filename editing, filtering, mouse navigation, scrollbar, and asynchronous clipboard paste.
- `import_review.rs`: source preview, error highlighting, diagnostic table, and independent scrolling.
- `examples/`: a styled document and an escaped ANSI review example.

The complete format and shortcuts are documented in the README.

## Stability and completion evidence

- `cargo fmt --check`, `cargo test --locked`, and `cargo clippy --locked --all-targets -- -D warnings`: pass. 68 tests cover document round trips, unsupported versions, invalid ranges/controls/colors, overwrite and symlink handling, failed writes, dirty-state transitions, save-before-replace/quit, export bytes, import review/strip/abort, source mapping, malformed recovery, and Unicode invariants.
- `uv run scripts/terminal_smoke.py`: passes on Linux PTYs. The real-input workflow saves and reopens a document, exports both forms, preserves edits on failed open and canceled import replacement, strips reviewed controls, verifies the imported style, and checks clean CLI reopen/quit.
- Earlier editing, selection, palettes, RGB, wrapping, gutters, scrollbars, clipboard, narrow-layout, and normal/error/panic terminal-restoration checks remain passing.
- The included `.tuxd.json` sample is decoded and round-trip validated by a test.

## Hands-on acceptance

1. Launch the styled example, edit it, Save As a new file, then reopen it.
2. Undo/redo around a save and observe the dirty marker.
3. Invoke New/Open/Quit with unsaved changes; exercise Save, Discard, and Cancel.
4. Try invalid input and a missing/unwritable destination; verify edits remain available.
5. Export Raw ANSI and Escaped ANSI and compare their intended representations.
6. Import `examples/import-review.txt` as Escaped ANSI. Review the unsupported clear-screen control, abort once, then strip and continue. Save the untitled result as `.tuxd.json`.
7. Try a longer review list and narrow terminal; navigate the dialogs and scrollbars.
8. Browse directories with mouse/keyboard, toggle hidden files and filtering, paste a location or filename with terminal paste and Ctrl+V, save into that directory, and verify overwrite confirmation.

## Limits and deferred work

Modern Linux terminals remain the compatibility target. Reads are limited to 16 MiB and require regular files; file I/O is synchronous. The file picker supports local directories; directory creation and remote browsing are not included. Persistence does not include autosave, recovery, file watching, or external-change conflict detection. ANSI import handles styled text, not terminal screen reconstruction, and unsupported syntax is explicitly reviewed.

Stdin import (`some_command --colors=always | tuxd`), automatic styled clipboard paste, selection copy/cut, style-only copy/paste, expanded/searchable character catalogs, workspaces with a left-hand explorer and multiple open files, recent-file menus, and language code generation remain separately tracked future work.

## File picker refinements

The explorer starts focused and supports h/j/k/l, gg/G, and Ctrl+. as a hidden-file alias. Import/Export use in-dialog Raw/Escaped radios; Ctrl+E opens Export directly. Open offers explicit ANSI import for non-`.tuxd.json` files. Ctrl+I remains italic (terminals that send Tab cannot distinguish it). The picker uses a rounded inset explorer, right-aligned actions, and a separate Ctrl+/ or Ctrl+? shortcuts panel with contrasting key backgrounds.

Picker follow-up: legacy Ctrl+/ decoding is covered, F1 is a shortcuts fallback, Enter on an ANSI radio submits, and Shift+M selects the middle visible explorer entry.

The File menu groups seven actions with separators. Import/Export default to Escaped ANSI. Unsupported ANSI Sequences uses a source preview with red/bold error spans and a Ratatui error table; both have independently scrollable five-row limits. Import anyway strips and continues without modifying the source file.

File dialogs retain an imported document’s source directory (or an untitled document’s export directory), with Ctrl+G returning to CWD. The shortcuts panel uses a fixed grouped grid of six essentials; all other bindings remain active.
