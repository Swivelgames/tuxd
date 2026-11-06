# Terminal UX Designer

A small Rust terminal editor for designing styled terminal text. The tools panel provides text attributes, foreground/background ANSI, indexed and RGB colors, and curated character palettes. The editor and live escaped ANSI drawer start visible with soft wrapping enabled. Slice 4 adds document save/open, reviewed ANSI import, and raw/escaped ANSI export.

## Run

Use a modern Linux terminal with UTF-8 support and Rust 1.88 or newer:

```sh
cargo run --locked
```

Try the included styled document:

```sh
cargo run --locked -- examples/welcome.tuxd.json
```

The editor uses the alternate screen. The title shows the document filename and `*` for unsaved changes. Ctrl+Q offers Save / Discard / Cancel when there are unsaved changes; clean documents close directly.

## Controls

| Key / action | Behavior |
| --- | --- |
| Text, Enter, Backspace, Delete | Insert or remove text and explicit newlines |
| Arrows | Move by grapheme or display row (logical line when unwrapped) |
| Ctrl/Alt+Left/Right | Jump by Unicode word boundaries; add Shift to select |
| Ctrl/Alt+Backspace | Delete the previous word or current selection in one undo step |
| Home / End | Start / end of line |
| Ctrl+Home / Ctrl+End | Start / end of document |
| Page Up / Page Down | Move one editor viewport vertically |
| Tab | Insert a literal tab, displayed at four-column stops |
| Ctrl+Z | Undo |
| Ctrl+Y / Ctrl+Shift+Z | Redo |
| Left mouse click / drag | Position the cursor / select text |
| Shift+arrows, Shift+Home/End | Extend selection |
| Esc | Clear selection and return to the editor |
| Ctrl+B / Ctrl+I / Ctrl+U | Toggle bold / italic / underline |
| F2 or Code button | Toggle the ANSI drawer |
| F3 or pane Wrap button | Toggle soft wrapping for the editor or code pane |
| Ctrl+C in code | Copy the entire escaped ANSI output, excluding display-only line breaks and line numbers |
| F6 | Cycle editor, tools, and open code drawer |
| Tab/Shift+Tab, arrows in tools | Choose a control; Enter/Space applies it |
| Home/End, Page Up/Down in tools | Reach the first/last control or move a page |
| Enter in RGB fields | Apply the staged RGB color; typing replaces a newly focused field |
| Arrows, Page Up/Down in code | Scroll generated output |
| Mouse wheel | Scroll three lines; the next edit/navigation returns to the cursor |
| F1 | Open / close help; Esc also closes it |
| F4 | File menu: New, Open, Save, Save As, Import, Export, Quit |
| Ctrl+N / Ctrl+O | New / open document |
| Ctrl+S / Ctrl+Shift+S | Save / Save As |
| Ctrl+E | Export ANSI (Raw / Escaped selector in the picker) |
| Ctrl+Q | Quit; when dirty, S saves, D/Y discards, C/N/Esc cancels |

Ctrl+Shift+Z requires the terminal to report Shift separately from Ctrl+Z. The app requests enhanced keyboard reporting where supported; Ctrl+Y is also available.

Use your terminal's paste command, typically Ctrl+Shift+V. Bracketed paste inserts multiline text as one undoable operation. CRLF and CR become newlines; tabs are preserved. Other Unicode control characters become visible Rust-style escape notation, such as `\u{1b}` for ESC, and cannot execute as terminal commands.

Typing and consecutive deletions are grouped until navigation, a different operation, or a pause of at least 750 ms. Newlines, tabs, and pastes are separate undo transactions. The last 200 transactions are retained. History currently uses document snapshots and is intended for modest documents.

Both panes have independent soft-wrap toggles, initially on, and always-visible logical line numbers. F3 toggles wrapping for the focused editor/code pane; each pane also has a clickable Wrap button. Gutters have a subtle navy background and a slate-blue active-line highlight, two cells of left padding, a right-aligned number field (at least two cells), and two cells of right padding. Continuation rows have a blank gutter. The editor cursor’s logical line has a highlighted gutter background and bold line number in both panes, including its wrapped continuation rows. Wrapping changes presentation only; with wrapping off, long lines scroll horizontally. Automatic indentation is not enabled. Vertical movement remembers the desired display column across short lines. Clicking within a tab or wide grapheme places the cursor at its leading boundary. Standalone zero-width graphemes use a dotted-circle display placeholder; the stored text is preserved. Partially clipped wide glyphs are left blank at viewport edges. A glyph wider than a wrapped viewport uses a replacement display marker; its original text remains intact.

## Development and validation

```sh
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
uv run scripts/terminal_smoke.py
```

The Linux PTY smoke check builds the app, sends actual terminal input sequences, checks the resulting screen, and verifies restoration of terminal modes on normal exit, a returned error, and a panic. `uv` supplies its pinned Python terminal-emulation dependency without changing project runtime dependencies. The diagnostic paths can also be exercised manually:

```sh
cargo run --locked -- --diagnostic-error
cargo run --locked -- --diagnostic-panic
```

Both intentionally exit unsuccessfully after initializing the terminal. The shell should retain normal echo/cursor behavior afterward.

Rendering uses Unicode grapheme segmentation and display-width estimates. Actual emoji, ambiguous-width, and font rendering can differ across terminal/font configurations. Validation currently covers Linux PTYs and tmux; desktop-terminal visual parity is not guaranteed. In tmux, mouse interaction may require enabling mouse support (`set -g mouse on`).

The editor core owns text, editing commands, cursor, history, and viewport state independently of rendering. Style ranges live in the document and are included in editing history. A proper Vim mode is future work.

See the [concept](.work/initial-concept.md), [MVP overview](.work/initiatives/mvp/overview.md), and [slice breakdown](.work/initiatives/mvp/breakdown.md).

## Styling and code inspection

Drag or use Shift+navigation to select text. Clicking a tool preserves the selection. `✓` marks active attributes/colors and `~` means mixed. Radio groups use a bright dot and a distinct background for the selected value; unselected dots are muted. Keyboard focus has its own background and underline. Toggling a mixed attribute enables it across the selection. Changing a color preserves other formatting. Esc clears selection so you can inspect the result without the gray selection overlay.

With no selection, tools set the style for subsequent typing. Cursor movement adopts the style immediately before the cursor; document start uses defaults. Replacing a selection inherits its first character's style. Text replacement and formatting changes are undoable. When editing joins Unicode characters into one grapheme, that grapheme takes its leading character's style.

The right panel appears on wider terminals. On narrow terminals, F6 opens tools in the editor area; Esc returns. Ctrl+I requires a terminal that distinguishes it from Tab; the tools panel always provides keyboard access to italic.

The Code pane starts open and shows read-only ANSI for the whole document; F2 toggles it. Every document newline appears as a visible `\n` followed by an actual display line break. Tabs appear as `\t`, and literal backslashes are doubled. These display line breaks are presentation only; raw ANSI generation preserves the original newline bytes. Selection highlighting is never included. Code wraps by default; turn wrapping off to scroll long code lines horizontally after focusing the drawer with F6 or a click.

The interactive preview enables color output even when `NO_COLOR` is set, because colors and attributes are document content. Attribute appearance still depends on your terminal/font. Syntax highlighting, drag autoscroll, and customizable application themes arrive later.

Ctrl+C while Code is focused copies the full escaped output, regardless of scrolling or wrapping. The clipboard contains literal `\x1b`, `\n`, and `\t`; it contains no extra physical newlines, gutters, or wrap markers. Copy uses `wl-copy` on Wayland or `xclip`/`xsel` on X11. Install the appropriate helper for your desktop. A status message reports success or failure, and the editor remains responsive during the operation. The terminal smoke suite uses isolated helpers and never changes your actual desktop clipboard.

Word movement uses Unicode word-boundary segments, skips intervening whitespace (including newlines), and treats punctuation as separate tokens. Left moves to the previous token's start; Right moves to the next token's end. Ctrl/Alt+Backspace uses the same backward boundary. Ctrl+Backspace requires a terminal that reports the modifier distinctly; Alt+Backspace is also supported.

## Color and character tools

The sidebar contains separate collapsible Style, Color, and Characters boxes. Click a box title, or focus it and press Enter/Space, to collapse or expand it. Color and character categories use compact dot-separated tab strips. Use the mouse wheel over the panel or keyboard navigation with F6 focus; Home returns to the first control. ANSI16 places basic colors (0–7) in the first row and bright colors (8–15) in the second. The 256 tab exposes every indexed value in seven-column rows, with right-aligned numbers. Both palettes use fixed-width buttons aligned left: Basic has two-space gaps and 256 has one-space gaps. The 256 grid shows at most five rows; mouse-wheel scrolling over it and keyboard navigation scroll its contents independently of Characters. Switching tabs or the foreground/background target only changes the tool view. Choosing a swatch applies it immediately to the selection or pending typing style. Reset restores the targeted channel to its terminal-default color; Clear resets all attributes and both channels. Formatting and character insertion participate in undo/redo.

The compact RGB form has synchronized hex (`#RRGGBB`) and decimal R/G/B fields (0–255). Typing or pasting into a newly focused field replaces its value; Backspace edits its end, and Ctrl+A makes the next input replace it. A separate swatch beside Hex previews a valid candidate. R/G/B gauges accept clicks, drags, and mouse-wheel adjustments. Left/Right adjusts by one, Shift+Left/Right by ten, and Home/End sets 0/255; direct numeric typing still works. Enter or Apply commits it. Invalid/incomplete input shows local feedback and never changes the document.

Box, Blocks, and Nerd palettes show spaced glyph grids. The focused character’s name and Unicode code point appear in a single detail line below the grid. Each activation inserts at the saved editor cursor or replaces the selection using its first character's style. Keyboard focus stays in the palette for repeated insertion; Esc returns to editing. Each insertion is a separate undo step.

ANSI palette appearances follow the terminal's palette. Dim, italic, and strike-through support varies; RGB requires true-color support. Nerd glyphs require a suitable Nerd Font and may appear as missing-glyph boxes otherwise. No font installation or detection is attempted.

Style controls use a compact row of B/I/U/D/S buttons, each previewing its attribute, plus Clear. All pane and popup borders are rounded. Tool surfaces inherit the terminal background; selected controls and keyboard focus retain their own highlights.

Scrollbars appear when content exceeds its viewport: vertical bars for Editor, Code, the tool sidebar, and the indexed-color grid; horizontal bars for Editor and Code when wrapping is off. Click a track to jump, drag its thumb, or scroll the wheel over it. Thumb size reflects the visible portion. Scrolling changes only the viewport, preserving text, selection, and undo history.

## Documents, import, and export

Each `.tuxd.json` file contains one editable document. Open, Save As, Import, and Export share a directory picker powered by `ratatui-explorer`. It starts beside the current document, or beside its import source. An untitled document exported before saving remembers its export directory. Otherwise it starts in the working directory. Ctrl+G returns to the working directory. Relative filenames resolve against the displayed directory; absolute paths and `~/` also work. Save As adds `.tuxd.json` when no extension is entered.

- The explorer is focused initially. Tab / Shift+Tab moves between Location, the file list, the ANSI format selector when present, File/path, and the action buttons. Ctrl+L selects Location; enter a directory and press Enter to browse it.
- In the list, arrows, Home/End, and Page Up/Down select entries. Vim-style `h/j/k/l` navigate parent/down/up/into-directory, and `gg` / `G` select the first / last entry. Shift+M selects the middle visible entry. These letter bindings apply only to the file list. Left/Backspace goes to the parent directory; Right enters a directory. Enter or double-click opens a directory/file; for Save As/Export, choosing an existing file fills the filename for review. Single-click selects; the wheel and scrollbar scroll the list.
- Ctrl+H or Ctrl+. toggles hidden files. Ctrl+F toggles `.tuxd.json`/all-files filtering. Open/Save As default to documents; Import/Export default to all files.
- Both text fields accept terminal paste and Ctrl+V (using `wl-paste` on Wayland or `xclip`/`xsel` on X11). Clipboard reads run in the background. Ctrl+A selects the field contents for replacement; Left/Right, Home/End, and Backspace/Delete edit the path.
- Ctrl+/, Ctrl+?, or F1 toggles a separate shortcuts panel beneath the picker. Key labels have contrasting backgrounds; use the mouse wheel over the panel if its contents exceed the available height. Legacy Ctrl+/ is supported; terminals that encode Ctrl+? identically to Backspace need Ctrl+/ or F1 instead. Ctrl+. and distinct Ctrl+I require terminal support for disambiguated keys; Ctrl+H and the Style tool remain alternatives.
- Import and Export have Raw ANSI / Escaped ANSI radios: click an option, or focus the group and use Left/Right or Space. Enter submits the selected format and filename.
- Enter submits the filename; Esc cancels. Existing destinations still require overwrite confirmation. F4 provides alternatives for terminals that cannot distinguish Ctrl+Shift+S. Menu entries and confirmation choices can also be clicked.

Save preserves editing history. Dirty state compares the current document to its saved snapshot, so undoing back to saved clears `*`. View changes and pending typing styles do not mark the document dirty. New/Open/import replaces the document only after validation and any Save/Discard/Cancel prompt. A successful replacement starts fresh history. Save As/export asks before overwriting an existing destination. Exports cannot replace the active document file.

Version 1 JSON stores `version`, `text`, and `styles.runs`. Runs have UTF-8 byte `start`/`end` offsets and a `style` containing the five attribute booleans plus independent foreground/background colors. Colors use `"Default"`, `{"Ansi": 0}`, `{"Indexed": 42}`, or `{"Rgb": [128, 192, 255]}`. Runs must cover the full text contiguously at grapheme boundaries. Loading rejects unknown versions/fields, invalid ranges/colors, and unsupported stored controls. Writes use a temporary file in the destination directory, flush it, and atomically replace the destination; existing file permissions are retained. Symlink and non-regular write targets are rejected. Reads require regular files of at most 16 MiB.

Open offers ANSI import when the selected filename does not end with `.tuxd.json`, keeping the file selected and asking for an explicit Import action. Cancel leaves the document unchanged. F4 has grouped New/Open, Save/Save As, Import/Export, and Quit entries. Import and Export default to **Escaped ANSI**; their radios also offer Raw ANSI. Both require UTF-8. Raw mode interprets actual SGR escape bytes; escaped mode decodes `\x1b`, `\n`, `\t`, and `\\` before parsing. Escaped input is the exported/clipboard payload, without the Code pane’s extra presentation line breaks. Literal Unicode and actual line breaks are preserved; CRLF becomes LF. Use `examples/import-review.txt` with Escaped import to try the review dialog.

Supported SGR parameters are reset (0), bold/dim/italic/underline/strike (1/2/3/4/9), attribute resets (22/23/24/29), ANSI basic/bright foreground/background colors, default foreground/background (39/49), and semicolon-form indexed/RGB colors (`38/48;5;n`, `38/48;2;r;g;b`). Reset 22 clears both bold and dim. If imported styling splits a grapheme, its leading character’s style wins, matching editor behavior.

Malformed or unsupported controls open **Unsupported ANSI Sequences** with a numbered, wrapped source preview and an error table. Erroneous sequences appear bold and red. Each view shows up to five rows and scrolls independently using the wheel or scrollbar; Tab focuses the preview, table, or action buttons for keyboard navigation. **Import anyway** (or S) strips the listed controls and continues; Abort (or A/Esc) cancels. The original file remains unchanged. A sequence containing unsupported SGR parameters is stripped as a whole, leaving the preceding style active. Truncated CSI stops before the next non-parameter text/control boundary. Unterminated OSC/DCS/control strings fail import because their end cannot be identified safely. No import content is executed by the terminal. Imported documents are untitled and unsaved; saving asks for a document destination.

**Raw ANSI export** writes actual escape bytes, original newlines/tabs, and a final style reset. **Escaped ANSI export** writes exactly the Code clipboard payload, without display-only line breaks. Export leaves the document path and dirty state unchanged. It is also the fallback when desktop clipboard helpers are unavailable.

Automatic styled clipboard paste, stdin import, workspaces/multiple open files, autosave/recovery, and character search remain future work.

The compact unsaved-changes prompt defaults to **Cancel**. Use Tab/Shift+Tab or arrows to select Save, Discard, or Cancel, then Enter; mouse clicks and the existing S/D/C shortcuts also work.
