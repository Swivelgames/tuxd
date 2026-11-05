# Terminal UX Designer

A small Rust terminal editor for designing styled terminal text. Slice 1 provides a conventional text editor; selection, styling, ANSI inspection, and saving are planned in subsequent slices.

## Run

Use a modern Linux terminal with UTF-8 support and Rust 1.88 or newer:

```sh
cargo run --locked
```

The editor uses the alternate screen. Text is session-local: Ctrl+Q asks before discarding a nonempty document. Saving is not available in Slice 1.

## Controls

| Key / action | Behavior |
| --- | --- |
| Text, Enter, Backspace, Delete | Insert or remove text and explicit newlines |
| Arrows | Move by grapheme or line |
| Home / End | Start / end of line |
| Ctrl+Home / Ctrl+End | Start / end of document |
| Page Up / Page Down | Move one editor viewport vertically |
| Tab | Insert a literal tab, displayed at four-column stops |
| Ctrl+Z | Undo |
| Ctrl+Y / Ctrl+Shift+Z | Redo |
| Left mouse click | Position the cursor |
| Mouse wheel | Scroll three lines; the next edit/navigation returns to the cursor |
| F1 | Open / close help; Esc also closes it |
| Ctrl+Q | Quit; Y discards text, N or Esc cancels |

Ctrl+Shift+Z requires the terminal to report Shift separately from Ctrl+Z. The app requests enhanced keyboard reporting where supported; Ctrl+Y is also available.

Use your terminal's paste command, typically Ctrl+Shift+V. Bracketed paste inserts multiline text as one undoable operation. CRLF and CR become newlines; tabs are preserved. Other Unicode control characters become visible Rust-style escape notation, such as `\u{1b}` for ESC, and cannot execute as terminal commands.

Typing and consecutive deletions are grouped until navigation, a different operation, or a pause of at least 750 ms. Newlines, tabs, and pastes are separate undo transactions. The last 200 transactions are retained. History currently uses document snapshots and is intended for modest documents.

There is no soft wrapping or automatic indentation. Long lines scroll horizontally. Vertical movement remembers the desired display column across short lines. Clicking within a tab or wide grapheme places the cursor at its leading boundary. Standalone zero-width graphemes use a dotted-circle display placeholder; the stored text is preserved. Partially clipped wide glyphs are left blank at viewport edges.

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

The editor core owns text, editing commands, cursor, history, and viewport state independently of rendering. This provides the base for selection and style ranges in Slice 2. A proper Vim mode is future work.

See the [concept](.work/initial-concept.md), [MVP overview](.work/initiatives/mvp/overview.md), and [slice breakdown](.work/initiatives/mvp/breakdown.md).
