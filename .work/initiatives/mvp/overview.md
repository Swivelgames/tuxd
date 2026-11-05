# MVP initiative

Status: In progress — Slices 1 and 2 complete

## Outcome

Deliver a small terminal text editor in which a user can type, select and style text, insert special characters, inspect ANSI, save an editable project, and export its rendered representation.

The [initial concept](../../initial-concept.md) records product direction. The [breakdown](breakdown.md) records sequencing and estimates. Individual slice files define runnable checkpoints and acceptance criteria.

## Milestones

- Slice 1: an editable text box users can play around in.
- Slice 2: visible range formatting and a live ANSI drawer; the first complete design-to-output workflow.
- Slice 3: the complete initial tools panel.
- Slice 4: a reusable workflow with project save/open, copy, and export.
- Slice 5: broader compatibility and integration confidence; the MVP release checkpoint.
- Slice 6: follow-on code-output improvements, tracked here but not required for MVP release.

## Completion contract for every slice

1. The application builds and launches through a documented command.
2. The slice's user-facing workflow is functional end to end, without mock controls or code edits to exercise it.
3. The manual walkthrough in its spec passes in an interactive terminal.
4. Relevant automated checks pass, including regression coverage for document mutations and output semantics where applicable.
5. Previously delivered behavior remains usable. New features cannot invalidate an earlier checkpoint.
6. Known limitations are documented; no known crash, data corruption, broken terminal restoration, or blocker in the slice's promised workflow remains.

Tests should protect behavior and meaningful boundaries. Use focused document/export tests and selected rendering checks, rather than duplicating implementation details in tests. Run formatting, compilation, linting, and appropriate tests for each implementation slice. Configure Rust formatting to use tabs in line with repository instructions.

## MVP acceptance scenario

Create a multiline snippet. Select a word with the mouse and apply bold, italic, and underline. Set foreground and background colors using the available palettes. Insert box-drawing and other glyphs. Edit around styled ranges, then undo and redo. Inspect the corresponding escaped ANSI. Save, close, reopen, and verify text and styles. Export an ANSI file and verify the intended rendering in a supported terminal.

Repeat core selection and tool operations using keyboard navigation. Verify resize behavior, narrow layouts, and clean terminal restoration on normal exit and recoverable failures.

## Scope boundaries

Include one document, explicit newlines, horizontal/vertical scrolling, linear selection, per-pane soft wrapping and logical line numbers, Unicode-aware word editing, code clipboard copy, and deterministic ANSI generation. Small documents are the performance target; large-file optimization is deferred.

Defer rectangular selections, multi-document tabs, arbitrary ANSI import, bidirectional code editing, and terminal cursor-control authoring. Language generation and syntax highlighting belong to slice 6.

## Architecture decisions

- Ratatui and Crossterm, with compatible versions selected during implementation.
- One crate and synchronous event handling.
- Styled document independent of terminal rendering and persistence format.
- Shared coordinate mapping for cursor placement, selection, and hit testing.
- Editing commands unify text/style mutations and undo/redo.
- Read-only generated output; actual escape bytes are distinct from their displayed escaped form.
- Versioned project format selected in slice 4, with validation and safe writes.

## Risks to resolve incrementally

Unicode mapping begins in slice 1 and receives mouse coverage in slice 2. Style boundaries and ANSI transitions are validated in slice 2. Terminal attribute/font limitations are surfaced in slice 3. Clipboard availability and safe persistence are addressed in slice 4. Slice 5 expands the environment and interaction matrix; it is not the first correctness pass.

Choose and record the primary development terminal/platform in slice 1, then define a small compatibility matrix before slice 5. Do not promise universal terminal rendering parity.

Current compatibility target: modern Linux terminals. Slice 1 is validated under Linux PTYs and tmux. Conventional editing is the initial input mode; a proper Vim mode is deferred to a separately scoped later slice.
