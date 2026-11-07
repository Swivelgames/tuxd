# Terminal UX Designer — initial concept

Status: Draft

Terminal UX Designer is a small Rust TUI for composing styled terminal text and obtaining the ANSI sequences needed to reproduce it. It should feel like a text editor: a large editable work area, a compact tools panel, and optional generated-code inspection.

## Core workflow

1. Type or paste text into the editor.
2. Select text with the mouse or keyboard.
3. Apply styles and foreground/background colors, or insert special characters.
4. See the result directly in the editor.
5. Inspect, copy, or export the ANSI needed to reproduce the result.

The editor itself is the preview. Generated code appears in a collapsible drawer below the editor; a larger code view can follow later. The tools panel sits on the right and becomes a toggled panel when space is limited.

## Interface draft

```text
 Terminal UX Designer                 untitled *     [Save] [Export]
┌─ Editor ─────────────────────────────────┬─ Tools ──────────────────┐
│                                          │ Style                    │
│  Build something delightful.             │ [B] [I] [U] [S] [Dim]   │
│                                          │ [Clear formatting]       │
│  ┌──────────────────────┐                │                          │
│  │  Your terminal UI    │                │ Color                    │
│  └──────────────────────┘                │ (●) Foreground ( ) Back  │
│                                          │ [ANSI 8/16] [256] [RGB]  │
│                                          │ ■ ■ ■ ■ ■ ■ ■ ■          │
│                                          │ ■ ■ ■ ■ ■ ■ ■ ■          │
│                                          │ [Default]                │
│                                          │                          │
│                                          │ Characters               │
│                                          │ [Box] [Blocks] [Nerd]    │
│                                          │ ┌ ┐ └ ┘ ─ │ ├ ┤ ┬ ┴ ┼  │
├─ Code ▾ ─────────────────────────────────┤                          │
│ [Escaped ANSI] [Rust]          [Copy]     │                          │
│ \x1b[1;36mHello\x1b[0m world             │                          │
└──────────────────────────────────────────┴──────────────────────────┘
 Ln 3, Col 12 · 5 characters selected          Undo · Redo · Help
```

This depicts the intended direction; controls appear as their slices become functional.

## Editing behavior

- Mouse dragging and Shift+arrow keys select text. Tool-panel interaction preserves the selection and insertion position.
- Formatting applies to the selection. With no selection, it changes the style used for subsequent typing.
- Moving the cursor adopts the adjacent document style; an explicit tool choice overrides it until the next cursor move or tool change. Define and test the boundary convention during slice 2.
- Mixed selections show indeterminate formatting. Activating a mixed toggle applies that attribute to the entire selection; activating a fully enabled toggle removes it.
- Changing one attribute preserves all unrelated attributes.
- Character insertion replaces the selection or inserts at the saved cursor position.
- Undo/redo includes text and formatting changes.
- Editor and code panes each provide toggleable soft wrapping and always-on logical line-number gutters. Wrapped continuation rows leave the gutter blank. Wrapping never changes stored text or generated output.

## Tools and output

The tools panel offers bold, italic, underline, strike-through, dim, and clear formatting; foreground/background selection; ANSI 8/16, indexed 256, and 24-bit RGB colors; and box, block, and Nerd glyph categories. RGB starts with hex and numeric component entry.

The initial code drawer displays escaped ANSI as read-only text. Each document newline is shown as a visible `\n` followed by a real display line break; tabs are shown as `\t`. ANSI file export will write actual escape bytes. Ctrl+C in the code pane copies escaped ANSI without presentation-only newlines or gutters. ANSI token highlighting is implemented. Sequence explanations remain deferred; language/framework generation belongs to the separate code generation discovery initiative, with initial targets to be chosen.

Each `.tuxd.json` document file preserves one document’s editable text and styles. Generated ANSI is an output, not the editable source of truth. Slice 4 includes explicit Raw/Escaped ANSI file import for supported styled text. Full terminal screen reconstruction and bidirectional code editing remain outside the MVP.

## Implementation direction

Use one Rust crate, Ratatui rendering, Crossterm events, and a synchronous event loop. Separate document, editing commands, layout, tools, export, and persistence into small modules.

The document stores text and normalized style ranges. Editor state stores cursor, selection, viewport, and pending typing style. Rendering, export, and persistence consume the same document. A shared mapping connects UTF-8 offsets, grapheme boundaries, and terminal display columns.

Evaluate existing textarea components briefly, but do not assume a plain-text widget supports persistent range formatting. Prefer a small owned editor core if adapting a widget would require duplicating document state.

## Scope and constraints

Target modest documents and one open document at a time. No plugin system, layout engine, async infrastructure, terminal emulator, or full IDE behavior is required.

Terminal capabilities affect the visible result. ANSI colors can follow the user's terminal palette; RGB and text attributes depend on support; Nerd glyphs require suitable font coverage. The app cannot choose or install the user's terminal font. Display-width differences across terminals require practical testing.

Never execute pasted escape sequences as terminal commands. Treat pasted control characters through a defined safe text policy, and generate output sequences only from document styles.

## Delivery principle

Every slice is a usable, stable increment. Each ends with a runnable application, a hands-on acceptance walkthrough, relevant automated checks, and regression verification of earlier behavior. Later hardening does not excuse broken fundamentals in earlier slices.

## Agreed initial decisions

- Conventional, nonmodal editing initially; modern Linux terminals are the current compatibility target.
- Literal tabs with four-column display stops and no automatic indentation initially.
- Ctrl+Z undo, Ctrl+Y redo, Ctrl+Q quit, and F1 help.
- Group typing/deletion until navigation, another operation, or a 750 ms pause; each paste is one transaction.
- Confirm discarding a nonempty document while saving is unavailable.
- Maximize the editor initially; introduce the right tools panel when it becomes functional.
- Include mouse click-to-position and wheel scrolling in Slice 1; selection follows in Slice 2.
- Normalize pasted CRLF/CR to newlines, preserve tabs, and display other control characters as escaped text.

## Future work: customizable themes

Add user-selectable and customizable UI themes in a later slice. Theme colors should cover gutters, active-line indicators, pane borders, selection overlays, controls, and status feedback. Keep application UI colors independent of document foreground/background formatting so switching themes never changes generated ANSI or saved content. Decide configuration, persistence, and built-in theme presets when that work is scoped. The current navy/slate gutter palette is the initial appearance, not a permanent customization constraint.

## Future work: proper Vim mode

Add a separately scoped later slice for a coherent Vim editing mode, including modal state, composable motions/operators, counts, and deliberate undo behavior. Decide the full compatibility contract, registers, visual selection, repeat behavior, and search in that slice. Keep document/editing commands independent of input bindings so modal and conventional editing can share the same document operations. A few Vim-like key aliases would not meet this goal.

See the [MVP initiative](initiatives/mvp/overview.md) and [slice breakdown](initiatives/mvp/breakdown.md).

## Technical references

- [Ratatui backends](https://ratatui.rs/concepts/backends/)
- [Ratatui colors](https://docs.rs/ratatui/latest/ratatui/style/enum.Color.html)
- [tui-textarea](https://docs.rs/tui-textarea/latest/tui_textarea/index.html)
- [XTerm control sequences](https://invisible-island.net/xterm/ctlseqs/)

## Future work: clipboard editing and character discovery

- Copy/cut the editor selection as escaped ANSI, preserving the selected text's formatting. Cut must remove text only after successful clipboard transfer and be one undoable edit.
- Paste escaped ANSI from any clipboard source into styled editor content. Define a supported text-style sequence grammar and literal-escape rules; never execute clipboard content as terminal commands. Keep malformed or unsupported input handling explicit and preserve Unicode boundaries.
- Expand the curated character catalogs and add character search by glyph, name, and code point.
- Copy/paste styles independently of text, applying them to a selection or the pending typing style with normal undo behavior.

Editor selection copy/cut and styled/literal/plain paste are implemented in the [clipboard editing initiative](initiatives/clipboard-editing/overview.md). Expanded/searchable characters and style-only copy/paste remain deferred.


## Future work: workspaces and multiple documents

Add project/workspace support as a collection of related document files, with a left-hand file explorer and multiple open files. Define navigation between open documents, per-document history and dirty state, workspace metadata, and save/close behavior in that iteration. The initial `.tuxd.json` format remains a single-document format.


## Stdin import

Allow styled command output to be piped into the app, for example `some_command --colors=always | tuxd`. Treat stdin as import data while opening the controlling terminal for interactive input. Reuse the ANSI import parser and malformed/unsupported-control review flow, and open the result as an unsaved document. Scope CLI mode selection, end-of-input behavior, noninteractive/no-controlling-terminal errors, and input-size handling in that iteration. Implemented with explicit `--raw INPUT` / `--escaped INPUT`, using `-` for stdin; see the [stdin import initiative](initiatives/stdin-import/overview.md).

## Future work: shortcut leader and General settings

Add an optional `<leader>` shortcut, unbound by default. Replace the standalone Sequence Timeout destination with a first category named General, containing Leader and Sequence Timeout. This is a follow-up to Slice 4a; it is not implemented yet.

When capturing a binding, recognize the configured leader as a `<leader>` prefix. The prefix adds one press to the requested capture length rather than consuming one of the requested action inputs:

| Capture started with | Configured leader | Inputs received | Recorded binding |
| --- | --- | --- | --- |
| Enter (one input) | Ctrl+Y | Ctrl+Y, Right | `<leader> Right` |
| 2 (two inputs) | Ctrl+Y | Ctrl+Y, g, g | `<leader> g g` |

After receiving the leader, keep capture open until the requested number of following inputs has arrived. Without a configured leader, preserve the existing capture behavior. The extra press is for the leader prefix; it must not recursively extend capture if the same key appears later in the sequence.

Before implementation, settle how symbolic leader references are persisted and resolved when the leader changes or is unset, and whether the current nine-press limit allows nine following inputs plus the leader. Validate conflicts against resolved sequences using the existing context and prefix rules. For example, Ctrl+Y currently defaults to Redo, so using it as a leader in the same context requires resolving that conflict rather than silently replacing Redo. Retain fixed settings controls, text-input protection, and explicit Save behavior.

## Near-term follow-up: component-emoji rendering

Replace the conservative Unicode escape preview for composed emoji with safe component-emoji rendering. The user approved the escaped fallback for now after reproducing cursor drift, cross-pane spillover and stale glyphs in bare Alacritty and tmux inside Alacritty/Zed.

Preserve the original document bytes and extended-grapheme editing throughout. Establish a shared width strategy for the app, multiplexer and terminal before displaying components; simply reserving more columns does not resolve disagreement in the terminal stack. Cover ZWJ sequences, skin-tone modifiers, regional-indicator flags, variation selectors and keycaps. Verify selection/deselection, insertion/deletion of preceding newlines, wrapping, horizontal scrolling, and clearing old screen cells in both direct and tmux sessions. Do not substitute display escapes into saved/exported/copied content.
