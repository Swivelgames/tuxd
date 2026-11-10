# Buffers and tabs

First checkpoint toward directory workspaces and templates. Native document format
is unchanged. Template grammar drafts are independent work.

## Ownership and behavior

| Area | Contract |
| --- | --- |
| Buffer | Owns session, editing history, selection/pending style, Editor and Code views, clipboard transactions |
| Workspace | Owns ordered buffers, active index, tab geometry, shared tools/layout/focus |
| File workflows | New adds a buffer; Open/Import reuse the unchanged initial placeholder or add a buffer; duplicate native path selects existing buffer |
| Publication | Save active; Save All queues dirty session IDs and returns to original tab on success |
| Close | Save/Discard/Cancel, initially Cancel; last removal creates an untitled buffer |
| Quit | Reviews dirty buffers in order; Cancel keeps all buffers open, even earlier Discard choices |
| Async | Completions route to originating buffer and retain revision/selection validation; removal invalidates transactions |
| Identity | Canonical native paths prevent duplicate opens and overwriting another buffer through Save As or export |
| Tabs | Inline beside tuxd; numbered untitled names, file/source names, dirty markers, active-visible overflow, mouse activation and per-tab x close buttons |

Default bindings: Ctrl+PageUp/Down switches document buffers; Ctrl+F4 closes;
Ctrl+Alt+S saves all. These are configurable Workspace → Files actions. Existing
Alt+Left/Right bindings continue switching slim tool tabs. File menus show current
bindings. Modal pickers/prompts block document switching.

## Validation

- Unit coverage: per-buffer history/selection/wrap/viewport, ordered switching,
	duplicate opens, cross-buffer publication protection, Save All, quit cancellation,
	last-buffer close, delayed paste targeting, shortcut dispatch and narrow tab strip.
- PTY Files suite adds buffer switching/undo, Save All, duplicate open and close-last.
- Existing tests are updated for inline header tabs and additive Open/Import.
- Full verification results are recorded after completion.

## Later checkpoints

Directory sidebar, workspace persistence, and templates remain separate slices.
No autosave, recovery, external-change detection, tab reordering, or new document
format is introduced here.

## Completed verification

- `cargo xtask check`: formatting, debug tests, clippy with warnings denied for
	app and xtask, and release tests including the normally ignored check passed.
- `cargo xtask smoke all`: all PTY suites passed, including the new buffer scenario.
- `cargo xtask render basic`: isolated bare Alacritty passed; initial and resized
	rendering evidence was captured under `/tmp/tuxd-alacritty-evidence-leGdFd`.
- Nine buffer regression tests passed, including delayed cut/paste ownership,
	Save All for both untitled and already-associated documents, failure cancellation,
	mouse activation, and dirty markers on clipped labels.
- Source and documentation review completed. Device-specific manual walkthroughs
	for this slice are not claimed; the automated suite and isolated Alacritty run
	are the recorded verification.

The next checkpoint is the [directory sidebar](directory-sidebar.md), implemented
on `feat/directory-workspaces`. It preserves these buffer ownership contracts.
