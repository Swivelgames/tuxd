# Terminal UX Designer

A small Rust terminal editor for designing styled terminal text. Slice 2 provides text selection, bold/italic/underline, eight ANSI foreground colors, wrapping, line numbers, and a live escaped ANSI drawer with clipboard copy. Saving and export are planned in subsequent slices.

## Run

Use a modern Linux terminal with UTF-8 support and Rust 1.88 or newer:

```sh
cargo run --locked
```

The editor uses the alternate screen. Text is session-local: Ctrl+Q asks before discarding a nonempty document. Saving is not available yet.

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
| Up/Down, Enter/Space in tools | Choose and apply a tool |
| Arrows, Page Up/Down in code | Scroll generated output |
| Mouse wheel | Scroll three lines; the next edit/navigation returns to the cursor |
| F1 | Open / close help; Esc also closes it |
| Ctrl+Q | Quit; Y discards text, N or Esc cancels |

Ctrl+Shift+Z requires the terminal to report Shift separately from Ctrl+Z. The app requests enhanced keyboard reporting where supported; Ctrl+Y is also available.

Use your terminal's paste command, typically Ctrl+Shift+V. Bracketed paste inserts multiline text as one undoable operation. CRLF and CR become newlines; tabs are preserved. Other Unicode control characters become visible Rust-style escape notation, such as `\u{1b}` for ESC, and cannot execute as terminal commands.

Typing and consecutive deletions are grouped until navigation, a different operation, or a pause of at least 750 ms. Newlines, tabs, and pastes are separate undo transactions. The last 200 transactions are retained. History currently uses document snapshots and is intended for modest documents.

Both panes have independent soft-wrap toggles, initially off, and always-visible logical line numbers. F3 toggles wrapping for the focused editor/code pane; each pane also has a clickable Wrap button. Gutters have a subtle navy background and a slate-blue active-line highlight, two cells of left padding, a right-aligned number field (at least two cells), and two cells of right padding. Continuation rows have a blank gutter. The editor cursor’s logical line has a highlighted gutter background and bold line number in both panes, including its wrapped continuation rows. Wrapping changes presentation only; with wrapping off, long lines scroll horizontally. Automatic indentation is not enabled. Vertical movement remembers the desired display column across short lines. Clicking within a tab or wide grapheme places the cursor at its leading boundary. Standalone zero-width graphemes use a dotted-circle display placeholder; the stored text is preserved. Partially clipped wide glyphs are left blank at viewport edges. A glyph wider than a wrapped viewport uses a replacement display marker; its original text remains intact.

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

Drag or use Shift+navigation to select text. Clicking a tool preserves the selection. `[x]` means the attribute or color applies throughout the selection, `[ ]` means it is off, and `[-]` means mixed. Toggling a mixed attribute enables it across the selection. Changing a color preserves other formatting. Esc clears selection so you can inspect the result without the gray selection overlay.

With no selection, tools set the style for subsequent typing. Cursor movement adopts the style immediately before the cursor; document start uses defaults. Replacing a selection inherits its first character's style. Text replacement and formatting changes are undoable. When editing joins Unicode characters into one grapheme, that grapheme takes its leading character's style.

The right panel appears on wider terminals. On narrow terminals, F6 opens tools in the editor area; Esc returns. Ctrl+I requires a terminal that distinguishes it from Tab; the tools panel always provides keyboard access to italic.

F2 opens read-only ANSI for the whole document. Every document newline appears as a visible `\n` followed by an actual display line break. Tabs appear as `\t`, and literal backslashes are doubled. These display line breaks are presentation only; raw ANSI generation preserves the original newline bytes. Selection highlighting is never included. Long code lines scroll horizontally after focusing the drawer with F6 or a click.

The interactive preview enables color output even when `NO_COLOR` is set, because colors and attributes are document content. Attribute appearance still depends on your terminal/font. Background and extended colors, file export, syntax highlighting, and drag autoscroll arrive later.

Ctrl+C while Code is focused copies the full escaped output, regardless of scrolling or wrapping. The clipboard contains literal `\x1b`, `\n`, and `\t`; it contains no extra physical newlines, gutters, or wrap markers. Copy uses `wl-copy` on Wayland or `xclip`/`xsel` on X11. Install the appropriate helper for your desktop. A status message reports success or failure, and the editor remains responsive during the operation. The terminal smoke suite uses isolated helpers and never changes your actual desktop clipboard.

Word movement uses Unicode word-boundary segments, skips intervening whitespace (including newlines), and treats punctuation as separate tokens. Left moves to the previous token's start; Right moves to the next token's end. Ctrl/Alt+Backspace uses the same backward boundary. Ctrl+Backspace requires a terminal that reports the modifier distinctly; Alt+Backspace is also supported.
