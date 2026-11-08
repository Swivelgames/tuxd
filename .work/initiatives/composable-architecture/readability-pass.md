# Post-refactor readability pass

Status: complete, ready for review on `refactor/code-organization`.
The Stage 8 checkpoint was committed and pushed as `3aa3b44` before this pass.
The readability pass is complete; publication was authorized after review.
No implementation work remains in the refactor initiative.

## Guidance and scope

The existing [shared guide](../../../docs/code-organization-and-readability.md)
was verified against agnostic's `meta/docs/code-organization-and-readability.md`:
it matched before this pass. The Workyard consistency/readability implementation
record supplied the peer-comparison and named-mode examples. A new Rust practices
section codifies those lessons here, and `AGENTS.md` reinforces them.

The review covered the application structure, independent libraries, features,
components, input/render peers, tests and acceptance tooling. Changes target
concrete comprehension costs rather than requiring every file to change. Existing
Stage 1–8 contracts remain the boundary for this pass.

## Changes and review map

| Area | Change | Preserved contract |
| --- | --- | --- |
| Workspace and App dispatch | Named `Pane` focus and `TextPane` collapse/wrap targets replace numbers and boolean selectors; consumers and tests use the same vocabulary | Editor → Tools → Code cycle, modal masking, collapse fallback, slim tabs, keyboard/mouse behavior |
| ANSI import | Named CSI consumption, control-string consumption and diagnostic-location operations replace deeply nested parsing details | Source offsets, recovery boundaries, atomic SGR application, fatal unterminated strings, exact raw/escaped behavior |
| Tool rendering | Shared factual tab renderer for color and character tabs; Color owns swatch bounds and contrast | Existing padding, hit bounds, focus/selection styling, document-ink previews |
| RGB errors | Match the actual error instead of rechecking it and retaining an unreachable fallback | Same error text, color and position |
| File picker | Input adaptation lives in `picker/input.rs`; directory/selection policy stays in the picker; choosing a file uses early guards | Default focus, selection, directory navigation, submit semantics, double-click, clipboard target ownership |
| Theme presets | Named `Palette` fields replace ten-position color arrays; removed redundant function alias | Every preset and role retains its exact color, checked by the existing compatibility fixture |
| Assembly and spacing | Grouped module declarations/imports in Settings, Workspace, App and Tools; separated library methods and compilation stages | No changes to supported library APIs, dependencies or persistence |
| Acceptance tests | Removed redundant nested test modules in already test-only files; explicit test imports after moving picker input | Same assertions and coverage; named pane fixtures replace numeric conventions |

Start review with `git diff --ignore-all-space` for the semantic structure.
The large raw diff in the two App test files is primarily one removed indentation
level. The new picker input file is a move within the same owner, not a new layer.

## Peer decisions

- Editor and Code continue sharing the text surface but retain distinct editing
	and readonly behavior. No universal panel/controller was introduced.
- Palette and character inset tabs share painting and hit geometry; the slim
	workspace tabs retain their different truncation and wrap-button policy.
- Swatch foreground contrast remains based on document ink, independent of theme.
- Raw and escaped import share control parsing while preserving original-source
	diagnostics. Clipboard, files and stdin continue selecting their own policy.
- Document and configuration publication retain different permission and overwrite
	policies. Configuration's two-file restoration remains explicitly best effort.
- Clipboard workers, permits, completion ownership and cancellation ordering were
	retained. Similar-looking copy/read paths were not merged into an executor.
- Numeric collection indices and simple booleans remain where their meaning is
	local and clear; this pass does not change public library signatures for style.

Complete changed source/test files were reviewed in their final form, with
surrounding documentation and related consumers checked for consistency. Existing
regression and fixture tests cover these behavior-preserving changes; no tests
were added merely to mirror helper extraction or whitespace.

## Validation

| Gate | Result |
| --- | --- |
| `cargo test --locked` | 179 passed: 19 library, 135 binary, 24 integration, 1 doctest; performance test ignored in debug |
| `cargo fmt --all --check` / `git diff --check` | Passed |
| `cargo clippy --locked --all-targets -- -D warnings` | Passed |
| PTY suites | All seven passed: terminal, clipboard, editor clipboard, stdin import, leader, theme, Settings/theme |
| Bare Alacritty and Alacritty/tmux | Real keyboard input, selection styling, Unicode restoration, save, Settings, resize/slim/collapse and clean exit passed |
| Release performance | 24.29 ms median, 24.97 ms p95, 51.91 ms max edit+frame+undo; 7.68 ms validation/native/ANSI roundtrip |
| Dependency audit | Independent libraries retain no App/Workspace/Settings/Files/Panel dependencies; no dependency changes |

Logs are in `/tmp/tuxd-readability/`. Alacritty evidence is in
`/tmp/tuxd-alacritty-evidence-sl74bh0u/` (bare) and
`/tmp/tuxd-alacritty-evidence-y1jmtetb/` (tmux); representative wide and slim captures
were inspected. The performance fixture remains 1,000 lines, 113,999 text bytes,
1,000 style runs and a 120×40 TestBackend. These timings are acceptance evidence,
not a statistically established speed improvement.

The final picker/theme changes were followed by another terminal/theme/Settings
PTY run. Linux Virtual Console, Termux and Zed manual walkthroughs were not rerun;
the user's previous acceptance remains the record for those environments.

No product behavior, defaults, file formats or deferred-feature scope changed.
