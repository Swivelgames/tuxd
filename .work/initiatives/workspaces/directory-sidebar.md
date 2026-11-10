# Directory workspace sidebar

Branch: `feat/directory-workspaces`. Builds on the v0.3.0 buffer/tab release.

Implementation and automated verification are complete. Directory and drag-input
changes were committed as `62e3a4c` and pushed to GitLab and GitHub. Merge and
release preparation remain separate publication steps.

## Scope and ownership

- `workspace/sidebar/tree.rs`: lazy filesystem discovery, sorting, native/hidden
	filters, expansion/collapse and transactional refresh. No buffer or UI knowledge.
- `workspace/sidebar`: Open Files projection, row focus, tree navigation and scroll
	geometry. Emits open/select/close requests; it does not read document contents.
- Workspace: docked/overlay placement, focus, active buffer selection and sidebar
	visibility. Tabs return when the sidebar is hidden.
- App/files: opening/importing documents, duplicate native identities, dirty-close
	prompts and directory selection through the existing picker.
- Shortcut registry: dedicated sidebar scope; shared explorer navigation and
	filter actions, plus directory open/focus/toggle/refresh commands.

## Decisions

| Concern | Behavior |
| --- | --- |
| Directory switch | Keep all open buffers; replace tree only after successful discovery |
| Startup | `tuxd` opens CWD; `tuxd DIRECTORY` chooses another root. Both validate discovery before entering terminal mode |
| Files | Native-only initially; all-files toggle uses existing import review |
| Hidden | Off initially; shared hidden toggle |
| Symlinks | File symlinks allowed; directory symlinks and special files omitted |
| Discovery | Expand on demand; explicit refresh preserves expanded directories |
| Open Files | Active/dirty indicators and close buttons, with relative paths within the root |
| Narrow | Overlay at <=64 terminal columns; return to Editor dismisses it |
| Persistence | None in this checkpoint; no workspace document format |

## Validation

Unit tests cover lazy discovery, filters, symlink cycles, directory-switch failure,
buffer retention, keyboard navigation, targeted close, responsive geometry and
scrollbar extent. PTY Files adds directory startup, picker, duplicate opens, hidden
and native filters, mouse toggling and slim overlay. Alacritty Workspace captures
wide and narrow layouts. Final results are recorded after checks complete.

Completed verification:

- `cargo xtask check --full` passed: formatting, debug/release tests, app/tooling
	Clippy, every PTY suite, bare/tmux rendering, and dependency notices.
- Width validation: 18,718 samples each in bare Alacritty and tmux, zero mismatches.
- Reviewed workspace captures in `/tmp/tuxd-alacritty-evidence-08Hq5u` (bare) and
	`/tmp/tuxd-alacritty-evidence-eKJQ8U` (tmux), including overlay dismissal.
- The tmux capture allows Escape forwarding to settle before its next key; the
	initial shorter capture interval could combine Escape and quit in the test.
- Full source/readability review and `git diff --check` completed. Manual checks
	in Termux, Zed and Linux virtual console are not claimed for this checkpoint.

## Final input refinement and follow-up

- Consecutive mouse-drag positions are coalesced within bounded work/time limits;
	click, release, modifier and other event boundaries retain their ordering.
- Crossterm's poll-based Unix backend avoids the large-input-burst stall found
	during regression testing. A 3,000-event PTY case verifies final selection,
	release, immediate typing and undo.
- `cargo xtask check --full` passed again with these changes, including stdin
	handoff, terminal restoration and bare/tmux rendering. User retesting confirmed
	dragging feels substantially better.
- Shortcut portability is documented in the [audit](../shortcuts/portability-audit.md).
	The user explicitly deferred diagnostics and default redesign until after this
	slice ships; no terminal user-configuration edits are a product prerequisite.

## Remaining closeout

- No-argument startup now opens CWD. `cargo xtask check` and `cargo xtask smoke all`
	passed after this change, including a child-process CWD discovery/open regression.
- CWD startup, sidebar spacing, shortcut audit and associated notes/tests were
	committed as `1942ca6` and pushed to GitLab and GitHub.
- Squash-merge when requested.
- For a release: regenerate and commit screenshots before the version tag, then
	run the release workflow with the requested publication targets.

No further directory functionality is required by this checkpoint. Workspace
restoration, file watching, directory creation and template/token work remain
future slices. Cross-device manual directory walkthroughs remain additional
validation, not completed evidence.

## Branch readability pass

Reviewed the branch's CLI/startup, terminal event handling, sidebar discovery and
rendering, workspace focus/layout, file picker/menu, shortcut integration, tests,
PTY/rendering tooling and documentation against the repository readability guide.

- Tree refresh owns both filter and entry updates; callers no longer repair filter
	state after discovery failure. A regression covers preservation on failure.
- Sidebar row styling and section headings have named rendering helpers; row
	selection owns the spacer-skipping invariant.
- Pane cycling has a dedicated operation, separate from action dispatch.
- File-menu separators follow named actions instead of positional thresholds.
- Picker filtering and submission use clearer predicates and one submission path.
- PTY editor setup shares sidebar dismissal, and default-CWD discovery has its own
	scenario helper.

Shortcut redesign remains deferred. Cleanup changes are separate from the pushed
checkpoint for review.

Validation: `cargo xtask check --full` passed after the cleanup, covering formatting,
debug/release tests, app/tooling Clippy, all PTY scenarios, bare/tmux rendering and
dependency notices. Both width runs checked 18,718 samples with zero mismatches.
The wide workspace capture was visually reviewed; `git diff --check` passed.
