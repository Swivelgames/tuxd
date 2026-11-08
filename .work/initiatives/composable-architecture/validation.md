# Validation and preparation evidence

Status: preparation and Stages 1–8 passed on 2026-11-07. The refactor is complete.
The original preparation baseline below is historical evidence; stage records
identify implementation checks and distinguish fresh automated/local graphical
results from earlier user device acceptance.

## Baseline identity

| Item | Value |
| --- | --- |
| Branch | `refactor/code-organization` |
| Last committed base | `006372d` — unified Settings and customizable themes |
| Working tree | Preliminary App/Files/Settings extractions and category changes; uncommitted replacement directories |
| Core results | 124 passed, 0 failed, 1 ignored performance measurement |
| Source changes during documentation/preparation | None |
| Graphical/manual checks in this preparation run | Not rerun |

Source fingerprint: `030ecdbaf34da16a7c80e9eb944fa5ff48918b4085421762230520a035dda23d`.
This SHA-256 covers 54 inputs: sorted `src/**/*.rs`, followed by `Cargo.toml` and
`Cargo.lock`, feeding each relative path, NUL, file bytes, NUL into the digest.
It identifies the uncommitted source tested; it is not a substitute for a commit.
Documentation is deliberately excluded so recording results does not invalidate it.

## Executed preparation checks

| Command | Result | Observed coverage |
| --- | --- | --- |
| `cargo fmt --all --check` | Pass | Existing Rust formatting |
| `cargo test --locked --quiet` | Pass: 124 passed, 1 ignored | Current unit/rendering/integration regressions |
| `cargo clippy --locked --all-targets --quiet -- -D warnings` | Pass | All-target static checks |
| `uv run scripts/terminal_smoke.py` | Pass | Editing, selection/styles, tools, wrapping, gutters, file operations, picker, shortcuts, field clipboard, error/panic/normal cleanup |
| `uv run scripts/clipboard_smoke.py` | Pass | Exact direct-SSH payload, isolated TMUX forwarding, failed TMUX without wrong-clipboard fallback |
| `uv run scripts/editor_clipboard_smoke.py` | Pass | Styled copy/cut/paste, atomic undo, review abort/accept, persistence, cleanup |
| `uv run scripts/stdin_import_smoke.py` | Pass | Raw/escaped stdin/files, keyboard handoff, review, save, limits, cleanup |
| `uv run scripts/leader_smoke.py` | Pass | General navigation, capture, affected-binding prompt, Save, unset |
| `uv run scripts/theme_smoke.py` | Pass | Loading, overrides, document colors/persistence, invalid-config fallback |
| `uv run scripts/settings_theme_smoke.py` | Pass | Preview, unsaved discard, save/restart |

Transient logs for this run: `/tmp/tuxd-refactor-preparation/00.log` through
`09.log`, in table order. They may be cleaned by the host; the commands and durable
results above provide the reproducible record. Smoke scripts isolate config,
clipboard helpers, and TMUX resources from the user's real state.

## Preservation matrix

| Behavior | Required invariant | Existing checks / planned boundary checks |
| --- | --- | --- |
| Document editing | Unicode/grapheme-safe text, normalized styles, exact stored content | Document/editor/style tests; new direct library tests for invalid input and complete mutations |
| Style inheritance | Current typing/selection/mixed-style and merged-grapheme rules | Editor/style tests and terminal suite |
| History | Same grouping, atomic operations, 200 snapshots, undo/redo | Editor tests; clipboard and terminal suites |
| Dirty state | Saved-content equality, including undo-to-saved; canceled/failed operations preserve document | Files tests; add explicit session checkpoint coverage if extraction exposes a gap |
| ANSI formats | Exact current raw/escaped outputs, proper literal escaping, no display decoration | Style/parser/native tests; compare encoder output across Code/clipboard/export consumers |
| Parsing | Original source diagnostic spans, inert controls, existing reject/strip distinction | Import/review tests, stdin and editor clipboard suites |
| Native format | .tuxd.json schema/version/validation preserved | Persistence/hardening tests and file walkthrough |
| Copy Code | Entire escaped document independent of scroll/wrap/highlighting | Terminal/clipboard suites; add direct Code coordinator fake-service tests |
| Copy/cut selection | No selection no-op; delete only on accepted success and matching target | Editor clipboard tests/suite |
| Delayed operations | Original session, revision, cursor, selection, style, and workflow eligibility | Existing stale tests; add cross-session/same-revision and owner-destruction coverage |
| Paste modes | Styled/plain/literal and conservative automatic detection stay distinct | Editor clipboard tests; preserve raw controls versus literal escapes |
| Clipboard transports | Same routing, payload limits, deadlines, unconfirmed status, no remote-read addition | Clipboard tests/suite; fakes for structured receipts/errors |
| Terminal writes | OSC 52 emitted between frames, never mixed with rendering | Clipboard suite and adapter tests |
| Shortcuts | IDs, defaults, three slots, labels, contexts, prefix conflicts, reserved controls | Shortcut/settings tests and terminal suite |
| Scope transitions | Pending sequence cancellation, modal eligibility, return focus | Existing tests; characterize eligible actions before refactoring focus |
| Catalog lifecycle | Complete when panels hidden, no duplicate registrations/default reset | New catalog/assembly tests justified by new boundary |
| Leader | Conversion/unset choices and resolved conflicts; prompt only when affected | Leader tests/suite |
| Settings drafts | Explicit Save, fixed navigation, unsaved Cancel, retained draft on failure | Settings tests and settings/theme/leader suites |
| Appearance | Midnight/default, all 14 presets, no Baseline, overrides and independent reset | Theme/settings tests and suites |
| Theme isolation | No document/export/clipboard mutations from UI appearance | Theme and rendering tests |
| Configuration failures | Existing load fallback, no load-time writes, save rollback/error reporting | Config/settings tests; exercise restoration failures where feasible |
| Text geometry | One mapping for widths, fallbacks, wrapping, hit testing, selection and gutters | Layout/UI/hardening tests; graphical Unicode regressions when migrated |
| Read-only consumers | Code/review never mutate document even through direct commands | New capability-level read-only tests plus existing rendering/output checks |
| Controls | Equivalent keys/mouse behavior with caller-specific validation | UI tests, field clipboard and terminal suites |
| Tools | Basic/256 spacing, bounded independent scrolling, staged RGB, glyph insertion/undo | UI/panel tests and terminal suite |
| Layout | Collapse/fill, <=64 slim tabs, focus reveal, tiny/resize safety | UI/hardening tests and terminal suite |
| File dialogs | Default directories, CWD, hidden/filter, navigation, format radios, path paste | Files/picker tests and terminal suite |
| File decisions | Dirty/overwrite/review order, Cancel default, errors preserve active document | Files/review tests and terminal suite |
| Filesystem | Existing bounds, special-file refusal, no-clobber, replace permissions/policy | Persistence/hardening tests and CLI checks |
| Startup | OS-string arguments, explicit raw/escaped '-', controlling-TTY handoff | CLI tests and stdin suite |
| Cleanup | Normal/error/panic and partial setup restore terminal modes | Terminal suite, stdin suite, applicable adapter tests |

Some rows list new tests as planned. They are not present merely because they
appear here. Reuse existing tests first, then add tests for new ownership boundaries
or uncovered behavior rather than assertions mirroring internal implementation.

## Stage-specific gates

| Stage | Minimum relevant checks after the source changes |
| --- | --- |
| 1 ANSI | Core checks; document/style/import/native/hardening coverage; terminal, editor clipboard, stdin suites |
| 2 Code/Clipboard | Core checks; clipboard and editor clipboard suites; terminal copy workflow |
| 3 Text/Session | Core checks; full terminal suite; relevant isolated Alacritty Unicode direct/TMUX checks; performance measurement |
| 4 Shortcuts | Core checks; terminal, leader, settings/theme suites; direct catalog/focus tests |
| 5 Themes/Settings | Core checks; theme, settings/theme, leader suites; configuration failure tests |
| 6 Controls/Tools | Core checks; terminal, editor clipboard, leader, settings/theme suites; narrow-size rendering |
| 7 Workflows/Storage | Core checks; terminal, stdin, editor clipboard, theme/settings suites; failure/rollback coverage |
| 8 Final integration | All automated suites; applicable graphical checks; performance; targeted manual compatibility walkthrough |

Core checks mean format, tests, and clippy as above. Run only relevant checks during
an iteration; do not repeat passing broad suites without a source change or new
concern. Final integration is broad because all boundaries have changed.

## Graphical and terminal compatibility

Prior accepted user testing covers Alacritty, TMUX in Alacritty/Zed, Linux virtual
console, and Termux over SSH. See the historical
[compatibility record](../mvp/slice-05-validation.md). This is prior evidence, not
a claim that the future refactor has passed those walkthroughs.

The isolated graphical harness can exercise actual Alacritty without touching the
desktop:

```sh
python3 scripts/alacritty_smoke.py --unicode
python3 scripts/alacritty_smoke.py --unicode --tmux
```

Inspect resulting screenshots and restored-body comparisons when display changes.
It requires its documented Alacritty/Xvfb/xdotool/ImageMagick environment. Mark an
unavailable check as not run, with the reason; do not equate PTY success to visual
terminal parity. Do not claim Android clipboard receipt from a local OSC payload.

Request user validation only for paths that automation cannot establish, after
independent checks pass. A focused final walkthrough should cover styled copy/paste,
focus/shortcuts, pane resizing, and Unicode selection/newlines on the applicable
terminals. Preserve the approved escaped fallback; component-emoji support is not
an acceptance requirement for this refactor.

## Performance

Existing deterministic modest-document benchmark:

```sh
cargo test --release modest_document_performance -- --ignored --nocapture
```

Historical target: edit + frame + undo p95 below 50 ms at 120×40; native/ANSI
roundtrip below 100 ms for the existing fixture. The ignored measurement was not
rerun during documentation preparation. Re-measure after document/layout changes
and at final integration; record actual numbers and environment, not a flaky unit
test deadline. Avoid duplicate full-document conversions per frame if composition
introduces them. No large-file or terminal-transport latency guarantee is added.

## Architectural acceptance

Behavior checks are necessary but insufficient. Review each migrated capability:

- No dependency on App/features from a library; no universal service context.
- ANSI/editing tests need no terminal, filesystem, or panel setup.
- Encoders have independent callers; copy does not strip display formatting.
- Complete operations protect invariants; callers do not mutate raw internals.
- Generic controls contain no application target enums.
- Themes resolve without Shortcuts or paths; metadata has one owner.
- Catalog definitions remain stable while component instances appear/disappear.
- Local handlers own features; root action/polling branches actually shrink.
- Async results match request/owner and validate mutation targets.
- State enums remove invalid combinations without merging independent states.
- Temporary bridges have been removed or are explicitly unfinished work.
- Complete changed files and their peers were reviewed after the final fix.

## Stage 1 evidence

The [Stage 1 record](stage-01-ansi.md) records the pushed `b674470` checkpoint,
implemented contracts, public API tests, remaining bridges, and final source fingerprint.
Core checks passed (132 tests plus the separately executed release measurement),
as did terminal, clipboard, editor clipboard, and stdin suites. Performance was
22.20 ms p95 for edit+frame+undo and 7.67 ms for validation/native/ANSI roundtrip.

The clipboard smoke suite exposed and verified the fix for a long-line style-lookup
slowdown. No acceptance assertions were weakened. Graphical/manual device checks
were not rerun for this ownership change; they remain gates for display extraction
and final integration.

## Stage 2 evidence

Stage 1 was committed and pushed as `c794ba6` before this work. The
[Stage 2 record](stage-02-clipboard.md) details independent transport requests,
Code-owned copy, caller-specific target checks, and remaining bridges.
Formatting, clippy with warnings denied, and 141 tests passed (11 library,
118 binary, 11 integration, 1 doctest; 1 performance test ignored). Clipboard,
editor clipboard, terminal, and stdin PTY suites all passed. Controlled drivers
cover one-shot completion, failures, unconfirmed receipts, owner destruction,
focus changes, identical-document replacement, and field targeting.

No graphical/device walkthrough or fresh performance measurement was performed;
this stage changes request ownership, not text layout. No known Stage 2 blocker remains.

## Stage 3 evidence

Stage 2 was committed and pushed as `94ccc79` first. See the
[Stage 3 record](stage-03-text-session.md) for ownership, bridges, and source identity.
Formatting, clippy, and 148 tests passed (16 library, 118 binary, 13 integration,
1 doctest; 1 performance test ignored in debug). All four PTY suites passed.
Isolated Alacritty Unicode checks passed directly and under TMUX, including exact
restored-body comparisons and screenshot inspection. The release measurement was
26.31 ms p95 edit+frame+undo and 7.64 ms validation/native/ANSI roundtrip, both
within existing targets. No new remote-device walkthrough was performed.

## Stage 4 evidence

Stage 3 was committed and pushed as `d8ac7ce`. The [Stage 4 record](stage-04-shortcuts.md)
details independent catalog/configuration/compiler/resolver ownership, local dispatch,
complete descriptor preservation and remaining bridges. Formatting, clippy and 156
tests passed. Terminal, leader, Settings/theme and theme PTY suites passed. No fresh
graphical/device walkthrough or performance run was performed for shortcut ownership.

## Stage 5 evidence

Stage 4 was committed and pushed as `4f5a535`. The [Stage 5 record](stage-05-themes-settings.md)
details in-memory theme contracts, typed catalog-driven Appearance, Settings pages
and exclusive modes, and supplied-path configuration commit with rollback. Formatting,
clippy and 165 tests passed. Theme, Settings/theme, leader and terminal PTY suites
passed. New tests preserve every role/preset value and cover configuration failure,
restoration failure, invalid drafts, edit ownership and discard prompt masking.
No new graphical/device walkthrough or performance measurement was performed.

## Remaining evidence

This section recorded the remaining work after Stage 5. Stage 8 results below
close the automated, graphical and performance gates. Remote/device manual results
remain historical, not newly asserted for the refactored build.

## Stage 6 evidence

Stage 5 was committed and pushed as `e16893e`. The
[Stage 6 record](stage-06-controls-tools.md) documents shared control semantics,
Tools ownership, retained input/painting policies and remaining Stage 7/8 work.

- 171 tests pass (16 library, 130 binary, 24 integration, 1 doctest); one debug
	performance test remains intentionally ignored.
- Formatting, whitespace and all-target clippy pass.
- Terminal, Settings/theme, leader and editor clipboard PTY checks pass.
- Contract tests exercise Unicode field boundaries, selection and input limits,
	partial grid navigation, ranges, RGB keyboard/pointer equivalence, invalid
	drafts and arrangement-independent state.
- No new real-device/performance result is claimed. Logs are in
	`/tmp/tuxd-controls-stage6/`; final integration still owns those gates.

## Stage 7 evidence

Stage 6 was committed and pushed as `df140a3`. The
[Stage 7 record](stage-07-workflows-io.md) documents typed picker/review outcomes,
source/destination ownership, shared storage and complete publication operations.

- 177 tests pass (19 library, 133 binary, 24 integration, 1 doctest); one debug
	performance test remains intentionally ignored.
- Formatting, whitespace and all-target clippy pass.
- Terminal, stdin import, editor clipboard and Settings/theme PTY suites pass.
- New cases exercise stale review targets, source origins, non-UTF-8 paths,
	bounded streams/files, FIFO rejection, no-clobber and permission policies.
- Logs: `/tmp/tuxd-workflows-stage7/`. Final real-device and performance gates
	remain Stage 8; no new result is claimed here.


## Stage 8 evidence

Stage 7 was committed and pushed as `f1dc426`. The
[Stage 8 record](stage-08-composition.md) closes the composition and migration work.

- 179 tests pass: 19 library, 135 binary, 24 integration, 1 doctest;
	one debug performance test intentionally ignored.
- Formatting, whitespace and all-target clippy pass.
- All seven PTY suites pass: terminal, clipboard, editor clipboard, stdin import,
	leader, theme, Settings/theme.
- Isolated bare Alacritty and Alacritty under tmux pass Unicode selection/newline
	pixel restoration, editing/style/save, Settings, normal/slim resize and collapse,
	clean exit and exact saved-document assertions. Screenshots inspected.
- Release fixture: 25.66 ms median, 28.79 ms p95, 53.40 ms max edit+frame+undo;
	7.99 ms validation/native/ANSI roundtrip. Preserved targets met.
- Dependency audit: no production App references outside application composition;
	no application imports in libraries; no production parent wildcard imports.
- Final changed source/test review and complete-path review performed.

Logs: `/tmp/tuxd-composition-stage8/`. Graphical evidence:
`/tmp/tuxd-alacritty-evidence-57kgcawf/` (bare) and
`/tmp/tuxd-alacritty-evidence-vs_3z8yu/` (tmux).
Linux Virtual Console, Termux/SSH and Zed manual walkthroughs were previously
accepted by the user; they were not rerun during Stage 8. No new terminal transport,
encoding or fallback behavior is introduced. Fresh device walkthroughs remain
optional release confidence checks, not claimed results.
