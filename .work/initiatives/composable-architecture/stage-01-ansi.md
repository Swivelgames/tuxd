# Stage 1 — independent ANSI document and representations

Status: complete on 2026-11-07, committed as `c794ba6` and pushed to
`origin/refactor/code-organization` before Stage 2 began. The record below describes
the Stage 1 boundary at that checkpoint.

## Source baseline and scope

Before implementation, the existing preparation and preliminary extractions were
committed as `b674470` and pushed to `origin/refactor/code-organization`, as requested.
This stage implements the ANSI boundary, migrates its existing consumers, and
separates native-format and rendering adapters. It does not implement the Clipboard
service, shared text surface, session model, or new product behavior.

The package now has a library target (`src/lib.rs`) and the existing binary target.
There is no new Cargo workspace, package dependency, or competing document model.
The binary imports the same library exercised by `tests/ansi_document.rs`.

## Implemented ownership

| Module | Owns | Excludes |
| --- | --- | --- |
| `libraries::ansi::document` | Private authoritative text/style state, positions, queries, grapheme navigation | Cursor/selection/history, viewport, input |
| `libraries::ansi::operations` | Checked complete insert/remove/replace/extract/style operations | Typing inheritance and undo policy |
| `libraries::ansi::style` | Semantic colors/attributes and canonical style ranges | Ratatui styles and UI themes |
| `libraries::ansi::parse` | Raw/escaped decoding, SGR interpretation, candidate document, original-source diagnostics | Files, prompts, session replacement |
| `libraries::ansi::encode` | Direct raw and escaped ANSI encoders | Display rows, gutters, syntax colors, clipboard transport |
| `libraries::ansi::data` | Deep serializable DTO, validation into an authoritative document | File version envelope and I/O |
| `adapters::native_document` | Existing version-1 `.tuxd.json` envelope and codec | Reading/writing files |
| `adapters::ratatui` | Semantic style/color to Ratatui conversion | Authored-data mutation or serialization |
| `code_projection` | Escaped Code display rows | Export or copy payload construction |

The ANSI module imports only its own modules, standard library types, serde, and
Unicode segmentation. Its tests need no App, terminal, clipboard, or filesystem.

## Public contract and invariants

- `Document::from_text` accepts literal text plus a style, rejecting unsupported
	controls and invalid basic colors. Newline/tab remain supported content.
- `Document::try_from(DocumentData)` validates complete, contiguous, nonempty
	style runs at grapheme boundaries. Equal adjacent styles become canonical runs.
	`to_data` returns detached serializable data; changing it cannot mutate the source.
- `Position` identifies a logical row and UTF-8 byte offset. `offset` and every
	mutation/range operation validate bounds and grapheme boundaries. Reversed ranges
	are rejected, and errors leave the document unchanged.
- `replace(range, fragment)` assembles text and styles, normalizes the resulting
	graphemes once, commits valid state, and returns a valid cursor. A fragment is
	another validated `Document`; no redundant fragment builder was needed.
- `insert` uses a default-styled literal fragment. Callers needing inherited or
	pending styles construct a styled fragment and use `replace`. `remove` replaces
	with an empty fragment. `extract` returns an independent styled document.
- `set_style`, `apply_style`, and `style_state` encapsulate range styling and mixed
	selection queries. `Action::Clear` clears styles through the same operation.
- Joined graphemes retain their leading character's style. Callers cannot mutate
	internal text/style storage and never call a normalization/repair method.
- `previous`, `next`, and `word_target` require valid document positions. Read-only
	`style_at` accepts a source byte within a valid line (including its end), without
	rescanning grapheme boundaries per painted cell. End-of-document style is default.
- `parse(source, Mode)` returns an inert candidate and original-source issues;
	ambiguous unterminated control strings still abort. Review decisions remain external.
- `to_raw_ansi` and `to_escaped_ansi` preserve current resets, SGR ordering, literal
	escaping, and trailing newlines. `escape` converts notation; it is not a general
	terminal-control sanitizer. Code alone adds physical display row breaks.

## Caller knowledge removed

Editor previously removed selection text, inserted text, patched style ranges by
byte offset, and normalized again. It now chooses inheritance/history policy and
submits one replacement. Styled paste uses the same operation; plain paste supplies
a fragment with the current typing style. Extraction and mixed-style queries also
delegate to the document instead of reconstructing ranges.

This removes an invalid intermediate state when replacing a newline before a
combining mark: replacing the newline in `e\n\u{301}` with `X` produces `eX\u{301}` as one edit,
with the inserted fragment's style and one undo step. Tests cover this boundary
both through the public document API and Editor, including restored selection.

Files, stdin/CLI, and clipboard share the library parser. Code copy, selection copy,
and raw/escaped export share direct encoders. Native decoding validates through the
same structured-data boundary. Rendering consumes semantic styles through the adapter.

## Temporary bridges and removal stages

| Bridge | Remaining consumer / reason | Removal stage |
| --- | --- | --- |
| `document.rs` reexports `Document`/`Position` alongside old display helpers | Editor/UI/layout imports stay stable while text presentation is still coupled | 3 |
| `styling.rs` reexports semantic types only | App/Tools/Editor imports migrate with their owners; no duplicate behavior | 3–6; final audit in 8 |
| `persistence.rs` reexports native `encode`/`decode` | Existing file workflows and cross-module tests | 7 |
| Root `code_projection.rs` | Display-only helper until Code owns projection and the shared text surface | 2–3 |
| Native codec uses `io::Result` | Existing workflow error compatibility; performs no I/O | Revisit error adapter in 7 |

App clipboard polling, Files review coordination, Editor viewport/history coupling,
and central action dispatch remain as previously implemented. This stage does not
claim those independent capabilities are complete.

## Verification

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | Pass |
| `cargo test --locked --quiet` | 132 passed: 5 library, 115 binary, 11 public API integration, 1 doctest; 1 performance test ignored in debug |
| `cargo clippy --locked --all-targets --quiet -- -D warnings` | Pass |
| `uv run scripts/terminal_smoke.py` | Pass: editing, styles, geometry, files/picker, shortcuts, clipboard, cleanup |
| `uv run scripts/clipboard_smoke.py` | Pass: exact >10 KiB SSH payload, isolated tmux forwarding, remote failure behavior |
| `uv run scripts/editor_clipboard_smoke.py` | Pass: styled copy/cut/paste, atomic undo, review, persistence |
| `uv run scripts/stdin_import_smoke.py` | Pass: raw/escaped stdin/files, review, save, keyboard handoff, limits, cleanup |
| Release performance measurement | 1,000 lines / 113,999 bytes / 1,000 style runs: median 21.73 ms, p95 22.20 ms, max 41.03 ms edit+frame+undo; validation/native/ANSI roundtrip 7.67 ms |
| Dependency and final-file review | No app, Ratatui, Crossterm, filesystem, or I/O imports in ANSI; final changed source/tests reviewed; whitespace check passed |

Performance command: `cargo test --release --locked modest_document_performance -- --ignored --nocapture`.
Environment: Linux x86_64, Rust 1.95.0, release profile, Ratatui TestBackend 120×40.
Both measurements meet the existing 50 ms p95 / 100 ms roundtrip targets.

The initial clipboard smoke run exposed repeated grapheme validation during style
lookup on long lines. Moving that check back to mutation boundaries restored the
existing fast lookup; the unchanged SSH/tmux acceptance test then passed. Assertions
and transport policy were not weakened. Final formatting/tests/clippy were also
rerun after a readability-only RGB preview expression cleanup.

Public API regressions cover rejected edits without mutation, Unicode joins,
atomic replacement, overlapping styles, fragment extraction, exact raw/escaped
outputs, structured-data validation, and 300 deterministic operations with validated
roundtrips after every step. Existing parser, semantic navigation, native, rendering,
and history tests were preserved or moved to their new owners.

Transient logs are under `/tmp/tuxd-ansi-stage1/`; durable results are recorded here.
No fresh graphical or user device walkthrough was performed: this stage changes
representation ownership, not fallback glyph/layout mechanics. Stage 3 and final
integration retain their graphical/compatibility gates.

Final source fingerprint: `dd47135906db05d242659eaa86a50a6bda0881a1f40393054b72c48f8786336c`.
SHA-256 covers 67 inputs: sorted `src/**/*.rs`, then sorted `tests/**/*.rs`, then
`Cargo.toml` and `Cargo.lock`; each contributes relative path, NUL, file bytes, NUL.
Documentation is excluded. This identifies the uncommitted source, not a commit.

## Next slice

Stage 2 introduces Clipboard request/receipt/transport ownership and a local Code
copy coordinator. It consumes the encoders added here. Preserve main-thread OSC 52
writes, target checks, cut-after-success, deadlines, and unconfirmed-delivery notices.
