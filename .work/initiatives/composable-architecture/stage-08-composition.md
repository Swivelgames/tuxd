# Stage 8 — final composition and cleanup

Status: complete on 2026-11-07, committed and pushed as `3aa3b44` on
`refactor/code-organization`.
Stage 7 was committed and pushed as `f1dc426` before implementation.
All eight implementation stages are complete.

## Ownership

| Owner | Owns | Caller contract |
| --- | --- | --- |
| `main` | Startup input and terminal event loop | Assemble App, install startup content, load configuration, draw/dispatch/poll |
| `app` | Services, active configuration, modal precedence, replacement/quit continuation | Route typed actions/effects; preserve workflow ordering |
| `workspace` | Session, Editor, Code, Tools, arrangement, focus, pane hit regions and pointer routing | Complete replacement and pane operations; draw with supplied chrome |
| `files` | Picker/menu/prompt state, interaction and rendering | Return typed cancel, activation, submitted selection or field-copy request |
| `features::editor::clipboard` | Pending requests, original targets, paste detection, stale checks and guarded edits | Return notice/review effects; accept session, editor, service and availability explicitly |
| `workflows::PendingReview` | Candidate, source, destination and review presentation | Consume a decision once and return a document/paste commit |
| `app::view` | Scoped theme, status selection and modal layering | Compose views without feature views accepting App |
| `workspace::view` | Normal/slim layout and pane chrome | Compose independent Editor/Code/Tools renderers |

Workspace applies Style, Glyph and Focus outcomes locally. A smaller outbound
effect enum contains only clipboard requests, notices and no-op. App therefore
cannot accidentally receive an unhandled document mutation effect. Settings and
Files mask Workspace input without changing the original clipboard request owner.

Session replacement creates a new identity, resets Editor/Code positions, restores
Editor availability/focus, and cancels pointer interactions. Existing wrap and
Code-collapse preferences are retained. App invalidates pending clipboard edits
and review before installation. The independent Workspace tests exercise these
postconditions without application services.

## Simplifications and cleanup

- Removed root `ui.rs`; pane composition now belongs to Workspace, and modal
	layering belongs to App. Code/editor chrome rendering has focused methods.
- Removed App-owned pane/mouse modules and the pane shortcut compatibility path.
	The workspace contributes its immutable descriptor slice directly.
- Removed root `editor_clipboard.rs`; feature transactions now own their pending
	request state, while a small App adapter handles review/notices.
- Removed Files' App-method implementation. File UI returns effects; application
	coordination retains dirty/overwrite/save/retry/quit ordering.
- Removed Files-owned exit and replacement continuation state, obsolete
	focus-follow bookkeeping, and an unreachable duplicate menu renderer.
- Removed the `styling` facade; consumers import the independent ANSI capability.
- Replaced production parent wildcard imports in Settings, Files and Tools with
	explicit dependencies. Enum-variant imports and test-module imports remain
	intentional.
- Made Workspace-only hit regions/drag and Files-only scroll/hit state private.
	Session and pane state remain crate-visible for composition and acceptance tests.
- Moved configuration startup into App assembly. Removed redundant session-path
	assignment after complete session installation.

There is one implementation per capability. Feature-local public exports expose
actual contracts; no migration facade or duplicate mutable document remains.
Application methods live under `app/`, not as extensions hidden in features.

## Preserved end-to-end paths

| Path | Ownership and ordering |
| --- | --- |
| Code copy | Resolved action → Code encoder/request → owned completion → receipt notice; exact escaped ANSI, independent of displayed wrapping |
| Cut | Editor transaction captures session/selection → transport completion → unchanged target and availability check → deletion/undo step |
| Paste | Original-target read → mode-specific parse → immediate edit or review request → consumed decision → target revalidation |
| Open/import | Picker selection → bounded read/validated candidate → optional control review → dirty decision → complete session replacement |
| Save/export | Format-specific encoder → protected destination/publication policy → successful saved/origin update → pending continuation |
| Settings | Fixed local controls → draft/preview → validation and configuration commit → activation only on success; cancel restores underlying modal context |
| Pane interaction | Effective scope → local action or Workspace pointer routing → shared text/control operations → normal/slim composition |
| Stdin | CLI source/mode → bounded read/parse before terminal entry → controlling-TTY input → common review/replacement → restored terminal |

No formats, IDs, defaults, limits, accepted key behavior, clipboard receipt meaning,
Unicode fallback, terminal setup or cleanup policy changed.

## Validation

| Gate | Result |
| --- | --- |
| Rust tests | 179 passed: 19 library, 135 binary, 24 integration, 1 doctest; 1 ignored debug performance test |
| Formatting / whitespace | `cargo fmt --all --check`, `git diff --check`: pass |
| Clippy | `cargo clippy --locked --all-targets -- -D warnings`: pass |
| PTY acceptance | All seven suites pass: terminal, clipboard, editor clipboard, stdin import, leader, theme, Settings/theme |
| Bare Alacritty | Unicode restoration pixel comparison, edit/style/save, Settings, resize/slim/collapse, exit and saved bytes: pass |
| Alacritty under tmux | Same checks: pass |
| Release performance | 25.66 ms median, 28.79 ms p95, 53.40 ms max edit+frame+undo; 7.99 ms validation/native/ANSI roundtrip |
| Dependency/readability review | No production App dependencies outside composition, no application imports in libraries, no production parent wildcard imports; changed source/test files reviewed |

The release fixture is unchanged: 1,000 lines, 113,999 text bytes, 1,000 style runs,
120×40 TestBackend. Its p95 and roundtrip remain within existing targets. The
Stage 3 p95 was 26.31 ms; these single local measurements are acceptance evidence,
not a claim of a statistically established speed change.

Logs: `/tmp/tuxd-composition-stage8/`.
Graphical captures inspected:
`/tmp/tuxd-alacritty-evidence-57kgcawf/` (bare),
`/tmp/tuxd-alacritty-evidence-vs_3z8yu/` (tmux).

## Limits and follow-up

Linux Virtual Console, Termux/SSH and Zed manual walkthroughs retain the user's
previous acceptance. They were not rerun for Stage 8; no fresh remote/device
acceptance is claimed. Automated PTY transport checks and real local Alacritty
checks cover the changed composition paths.

Existing limitations remain: composed emoji may use escaped display fallback;
remote clipboard reads are unavailable; OSC 52/tmux forwarding cannot confirm
OS clipboard receipt; configuration restoration is best effort across two files;
history and derived text layout retain their existing modest-document design.

No refactor stage remains. Deferred product work is still tracked separately in
[the backlog](../../deferred-work.md). Stage 8 is committed and pushed; the branch has not been merged.
The subsequent [readability pass](readability-pass.md) is tracked separately.
