# Stage 5 — independent themes and Settings contracts

Status: complete on 2026-11-07, committed and pushed as `e16893e` on
`refactor/code-organization`.
Stage 4 was committed and pushed as `4f5a535` before implementation.
Stage 6 (shared controls and locally composed Tools) is next.

## Implemented boundaries

| Owner | Responsibility |
| --- | --- |
| `libraries::themes` | UI color values, typed role/group catalog, preset catalog, configuration schema, color syntax, validation and pure resolution |
| Role catalog | One definition for stable role ID, label, group, display order, resolved-field lookup and assignment |
| `theme` | Ratatui color/style/block adapter and scoped resolved appearance; restores prior appearance on return or unwind |
| `theme::storage` | Existing theme JSON file reads and validated atomic writes, using supplied paths |
| `configuration::directory` | Application XDG/HOME location policy, independent of both codecs |
| Settings shell | Fixed focus/navigation, page composition, dirty lifecycle, status, hit routing and exclusive interaction mode |
| Appearance page | Own preset/role selection and draft/original state, catalog-derived rows, temporary preview and role editing |
| Shortcuts page | Own binding draft/original, selected action/slot, remembered actions, collapsed subsections and binding capture |
| General page | Leader capture/conversion decisions and timeout editing against the single shortcut draft |
| `configuration::commit` | Validate both drafts, publish changed theme, invoke shortcut publication, restore prior theme on failure |
| `app::settings` | Deliver Settings events, pass narrow drafts/destinations to commit, activate theme and close only on success |

Theme resolution imports no App, Settings, Ratatui, Crossterm, filesystem, paths,
or authored-document types. Its public integration tests exercise it directly.
UI color values intentionally remain separate from ANSI ink. The adapter alone
converts UI colors into Ratatui values; document colors are never substituted.

Role strings remain unchanged in JSON. Appearance now carries `Role` and `Group`
identities instead of indexing a second table of numeric role positions. It gets
labels and ordering from the same catalog used by resolution. The existing preset
ID/label catalog and all palette values remain unchanged. A frozen fixture checks
all 21 roles across all 14 presets against the pre-refactor values.

## Settings lifecycle

`Mode` represents Browse, category selector, preset chooser, color edit, timeout
edit, binding capture, leader capture, leader conversion decision and discard
confirmation. They cannot coexist. The shell dispatches active-mode keys before
underlying page actions. Capture buffers and temporary field inputs remain local;
confirmation updates only drafts. Validation still delegates to the shortcut and
theme libraries, and fixed controls do not consult configurable bindings.

Review found that Delete could reach the underlying Appearance reset while the
unsaved-discard prompt was open. Mode-first dispatch now masks it. A regression
checks that neither Delete nor Ctrl+S changes the draft or leaves that prompt.

Pages construct their own state. Page-specific input and rendering live beside
those responsibilities; shell routing coordinates common focus, status and outcome
semantics. General and Shortcuts deliberately share one shortcut configuration,
so leader conversions and binding capture cannot diverge into separate drafts.
Settings exposes borrowed commit data, not its internal page state or App access.

Preserved: expanded subsections, spacing, remembered/context-selected actions,
small-size layouts, scrollbars, preset preview, independent reset, unsaved Cancel,
leader warnings/default Cancel, literal-preserve/remove choices, capture count,
terminal-key reporting, explicit Save and retained drafts on failure.

## Storage and activation

The application supplies the theme destination independently of shortcut storage.
Theme loading no longer calls `shortcuts::config_path`. The common directory
resolver preserves the existing XDG/HOME policy and errors.

Commit validates both configurations before effects. If the theme changed, it
captures the previous bytes/absence, publishes the new theme, then invokes the
shortcut publisher. That publisher activates bindings only after its write succeeds.
On failure, commit restores the previous theme bytes or removes the newly created
file; restoration errors retain the original error and add the restoration failure.
The application activates its theme and closes Settings only after complete success.

This preserves best-effort rollback, **not crash-atomic publication of two files**.
Common bounded-read/atomic-write mechanics remain Stage 7. Theme JSON version 1,
shortcut schema/legacy support, limits, tab-indented output, Midnight default,
all preset IDs/values and absent Baseline remain intact. No migration was added.

The render adapter retains the existing explicit `theme::with(&resolved, render)`
scope. Its thread-local storage contains only resolved appearance and restores the
previous value even on panic; it is not a service context or persistent active
configuration. Existing render consumers keep their narrow role accessors.

## Remaining work

| Bridge | Removal/completion stage |
| --- | --- |
| Settings field editing, list geometry and prompt/button painting remain feature-local | Shared controls in Stage 6 |
| General/Appearance/Shortcuts handlers coordinate shell focus/status through Settings methods | Compose shared controls in Stage 6; no whole-App dependency remains |
| Theme and shortcut atomic-write adapters retain separate mechanics | Common storage in Stage 7 |
| App retains configuration destination and final active-theme assignment | Final application/root composition in Stage 8 |

## Verification

| Check | Result |
| --- | --- |
| Core tests | 165 passed: 16 library, 128 binary, 20 integration, 1 doctest; 1 debug performance test ignored |
| Formatting/whitespace | `cargo fmt --all --check`, `git diff --check`: pass |
| Clippy | `cargo clippy --locked --all-targets -- -D warnings`: pass |
| Settings/theme PTY | Preview, unsaved discard, explicit Save and restart: pass |
| Leader PTY | General navigation, capture, affected-binding warning, Save and unset: pass |
| Theme PTY | Loading, overrides, authored-document independence and invalid-config fallback: pass |
| Terminal PTY | Fixed/remapped controls, capture, sequences, modal return and file/field flows: pass |

New coverage checks every preserved palette value, unique role IDs and groups,
every override's in-memory serialization/resolution and isolation, independent
configuration destinations, publication failure, rollback failure reporting,
invalid-draft side-effect prevention, exclusive input ownership and catalog-driven
Appearance ordering. Existing rollback tests cover both prior-file and absent-file
restoration. Full changed-source/test review and edited-document review completed.

Logs: `/tmp/tuxd-themes-stage5/`. No new graphical/device walkthrough or performance
measurement was needed for these ownership changes; final integration retains
those gates. No known Stage 5 blocker remains.
