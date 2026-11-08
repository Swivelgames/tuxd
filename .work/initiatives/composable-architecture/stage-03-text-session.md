# Stage 3 — text surface, editing, and document sessions

Status: complete on 2026-11-07; committed and pushed as `d8ac7ce` on
`refactor/code-organization` before Stage 4. Stage 2 was committed as `94ccc79` and pushed before
this work began. Stage 4 (shortcuts and focus) is next.

## Ownership and simplification

| Owner | Responsibility | Does not own |
| --- | --- | --- |
| `libraries::editing::Editing` | Document, cursor/selection, pending style, preferred column, revision, 200-entry history and grouping | Viewport, widgets, input keys, files, clipboard transport |
| `libraries::text::display` | Conservative glyph fallback, tab widths, source-byte/cell mapping, literal input sanitization | ANSI serialization or UI appearance |
| `libraries::text::Layout` | Logical/source positions to wrapped rows/cells and inverse hits | Editing commands and terminal backend |
| `libraries::text::Viewport` | Width/height, wrap, offsets, clamping, cursor reveal, scroll/hit mechanics | Document mutation, App, theme |
| `DocumentSession` | Stable identity, Editing, native path, source origin, saved checkpoint | Rendering or file I/O |
| `components::text_surface` | Shared painting, clipping, gutters, cursor placement, geometry and owned scrollbar interaction | Mutable documents and feature actions |
| `components::scrollbar` | Generic orientation, thumb/offset mapping and rendering | Editor/Code/Tools identities |
| `features::editor::Editor` | Coordinate semantic edits/navigation with cursor reveal and authored-style presentation | Authoritative document/history storage |
| `features::code::Code` | Read-only navigation, viewport, escaped projection/syntax rendering, existing copy ownership | Editing commands or mutable document access |
| `files::review::projection` / review | Inert source/diagnostic mapping, shared preview surface, independent error table | Committing imported content |

The old root `editor.rs`, `document.rs`, `layout.rs`, and `code_highlight.rs` are
removed. Callers import semantic data from ANSI and display behavior from Text.
No new dependency, Cargo workspace, persistence format, shortcut ID, or UI feature
was introduced. Existing tests moved with their owners; the test-only Editor
fixture composes the real Editing and Editor types rather than keeping old logic.

## Editing and session contracts

Editing operations remain complete semantic operations, using the ANSI document's
checked mutations. Typing inheritance, reverse-selection collapse, word movement,
750 ms grouping pause, atomic edits, and snapshot representation are preserved.
The editing library is usable from integration tests without a widget or App.

For visual navigation, Editor supplies `Navigation` with a pure `Layout`, wrap
mode, and page height. Editing chooses the caret/selection result. Editor then
reveals that caret using its viewport. Mouse hits use the same mapping as painting;
scrolling deliberately does not reveal the caret. Resize and wrap retain their
existing follow-cursor rules. No-op deletion/undo and empty insert/paste preserve
whether cursor reveal occurs.

A new/open/import operation constructs a new session identity even for identical
content and revision. Clipboard mutation targets capture identity in addition to
revision, original content, cursor, anchor, and pending style. Existing cancellation
on replacement and modal eligibility checks remain. Undo never changes identity.

Saved checkpoints compare document content rather than revision. Undo-to-saved
clears dirty; redo restores dirty. Imports start unsaved with an origin but no native
path. Successful save updates path/checkpoint, while export retains the existing
origin-selection policy and does not mark the document saved.

## Shared presentation and read-only Code

Editor, Code, and review supply borrowed lines, a layout, gutter policy, and a
position-to-style function to the same surface. The surface cannot edit content.
Editor supplies authored styles plus its selection overlay; Code supplies syntax
styles; review supplies bold error coloring over escaped original-source spans.

The surface owns its text/inner bounds and its scrollbar geometry and drag state.
Root mouse routing delegates text scrollbar events to each owner; it no longer
sets Editor/Code offsets through application-specific scrollbar target variants.
Tools/Indexed retain a small named routing bridge until Stage 6.

Code projection retains logical source rows, ranges in the encoded string that
exclude inserted display newlines, and syntax tokens. Source rows preserve the
active gutter's link to Editor. Copy/export continue to use direct ANSI encoders.
Code accepts only read-only navigation actions and an immutable document for drawing.
Direct editing actions passed to Code are ignored.

Derived projection and layout are rebuilt from authoritative content for each draw.
There is no retained serialization/layout cache to invalidate: replacement/content,
width/wrap, and current appearance are always reflected. The existing performance
target passes without introducing cache keys or an incremental invalidation system.

The approved escaped fallback for composed emoji, four-cell tab stops, wide glyph
clipping, blank continuation gutters, selected-newline cell, slim layout, and pane
collapse behavior remain. Review keeps its five-row preview and separate five-error
table; shared painting now supplies its cell clipping and fallback handling.

## Remaining bridges

| Bridge | Why it remains | Removal stage |
| --- | --- | --- |
| Editor/Code action adapters use the current ActionId registry | Preserve existing IDs/context behavior; movement and selection map to one semantic operation | 4 |
| UI root owns arrangement, borders, wrap buttons and focus reconciliation | Text rendering/geometry is local; whole workspace composition is later | 8 |
| Root routes editor caret clicks and drag-selection gestures | Caret/selection policy stays distinct from read-only surface mechanics | 8 |
| `editor_clipboard` still implements App workflow methods | Targets now use sessions, but modal/review orchestration is not yet extracted | 7 |
| `scrollbar.rs` aliases the generic primitive with Tools/Indexed routing | Remaining controls retain their current dispatch while text surfaces are local | 6 |
| Theme functions supply rendering roles | Independent theme/context contracts are a separate stage | 5 |
| `styling.rs` semantic reexports and native codec facade | Tool/workflow consumers migrate with their owners | 6–7; audit in 8 |

## Verification

| Check | Result |
| --- | --- |
| Formatting and whitespace | `cargo fmt --all --check`, `git diff --check`: pass |
| Core tests | 148 passed: 16 library, 118 binary, 13 public integration, 1 doctest; 1 debug performance test ignored |
| Clippy | All targets, warnings denied: pass |
| Terminal PTY | Pass: editing, geometry, fields, picker/files, shortcuts, cleanup; rerun after local movement adapter extraction |
| Clipboard PTY | Pass: exact SSH payload, isolated TMUX forwarding, failure routing |
| Editor clipboard PTY | Pass: styled editing, atomic undo, review, persistence and cleanup |
| Stdin import PTY | Pass: raw/escaped input, review, save, handoff, limits and cleanup |
| Isolated Alacritty Unicode, direct | Pass: selection, ten inserted/deleted newlines, exact restored-body comparison, save, settings, resize/slim/collapse, exit |
| Isolated Alacritty Unicode, TMUX | Same checks pass |
| Release performance | 1,000 lines / 113,999 text bytes / 1,000 style runs: median 25.49 ms, p95 26.31 ms, max 52.08 ms edit+frame+undo; roundtrip 7.64 ms |

Performance remains below the established 50 ms p95 and 100 ms roundtrip targets.
Stage 1 measured 22.20 ms p95; this stage adds explicit layout coordination without
requiring an incremental cache. Measurement: Linux x86_64, Rust 1.95.0, release,
Ratatui TestBackend 120×40, `cargo test --release --locked modest_document_performance -- --ignored --nocapture`.

New regressions cover widget-free editing, the 200-entry history limit, independent
session identity, undo/redo dirty state, imported/exported origins, replacement
content at matching revisions, Code's read-only behavior, projection encoded offsets,
and replacement rendering after scrolling. Existing navigation, fallback, selection,
resize, scrollbar, gutter, review and deterministic editing assertions remain.

Logs are under `/tmp/tuxd-text-stage3/`. Graphical evidence is under
`/tmp/tuxd-alacritty-evidence-ldvekfvi` (direct) and
`/tmp/tuxd-alacritty-evidence-5vvme9d9` (TMUX). Restored Unicode and slim screenshots
were inspected; the harness verifies the restored body pixel-for-pixel. These checks
are not a fresh Termux/Zed/Linux-console user walkthrough; final integration retains
those targeted manual compatibility checks.
