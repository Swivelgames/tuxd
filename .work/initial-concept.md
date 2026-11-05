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
- Explicit newlines and horizontal scrolling come first. Soft wrapping is deferred so terminal layouts retain their shape.

## Tools and output

The tools panel offers bold, italic, underline, strike-through, dim, and clear formatting; foreground/background selection; ANSI 8/16, indexed 256, and 24-bit RGB colors; and box, block, and Nerd glyph categories. RGB starts with hex and numeric component entry.

The initial code drawer displays escaped ANSI as read-only text. ANSI file export writes actual escape bytes, while code copying copies the visible escaped representation. Later work adds ANSI token highlighting, sequence explanations, and language-specific string generation, starting with Rust.

Project files preserve editable text and styles. Generated ANSI is an output, not the editable source of truth. Importing arbitrary terminal output or editing generated code bidirectionally is outside the MVP.

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

## Future work: proper Vim mode

Add a separately scoped later slice for a coherent Vim editing mode, including modal state, composable motions/operators, counts, and deliberate undo behavior. Decide the full compatibility contract, registers, visual selection, repeat behavior, and search in that slice. Keep document/editing commands independent of input bindings so modal and conventional editing can share the same document operations. A few Vim-like key aliases would not meet this goal.

See the [MVP initiative](initiatives/mvp/overview.md) and [slice breakdown](initiatives/mvp/breakdown.md).

## Technical references

- [Ratatui backends](https://ratatui.rs/concepts/backends/)
- [Ratatui colors](https://docs.rs/ratatui/latest/ratatui/style/enum.Color.html)
- [tui-textarea](https://docs.rs/tui-textarea/latest/tui_textarea/index.html)
- [XTerm control sequences](https://invisible-island.net/xterm/ctlseqs/)
