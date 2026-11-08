# Stage 6 — shared controls and Tools ownership

Status: complete on 2026-11-07; committed and pushed as `df140a3` on
`refactor/code-organization` before Stage 7.
Stage 5 was committed and pushed as `e16893e` before implementation.
Stage 7 (workflows and I/O) is next.

## Boundaries and simplifications

| Owner | Responsibility |
| --- | --- |
| `libraries::controls::Field` | Text, grapheme-boundary caret, whole-field selection, replacement, insertion, movement and deletion |
| `libraries::controls` | Bounded grid navigation, cyclic choice navigation and clamped range stepping/pointer mapping |
| `components::field` | Shortcut-to-field adapter and single-line field painting |
| `components::field_clipboard` | Captured field-copy payload and pending write completion |
| `components::scrollbar` | Orientation, thumb geometry, grab calculation and offset mapping; no application target IDs |
| Tools `Panel` | Compose component layouts, remember group focus, own selected control, viewport, hit geometry, input and typed effects |
| Style | Collapse state and style-control layout |
| Color | Target, palette, indexed viewport, gauges, explicit RGB draft and original-target paste completion |
| Characters | Collapse state, selected catalog, glyph definitions and grid layout |
| `app::tools` | Apply style/glyph effects to Editing, coordinate clipboard requests and publish focus/notices |

The old root `panel_ui` renderer and transitional `scrollbar` facade are removed.
Tools drawing borrows only Editing for active/mixed style presentation; it has no
App reference and cannot mutate the document. RGB painting belongs to Color.
Tool text input, keyboard navigation, mouse hit testing and pointer dragging now
operate on the same component state. Geometry is invalidated through `hide`;
pointer release is a complete local operation.

Sidebar and slim tabs use the same Style, Color and Characters instances. The
workspace still chooses arrangement and visibility; changing arrangement does
not recreate RGB drafts, catalogs or pending paste requests.

## Shared mechanics, distinct policies

Picker Location/Name, RGB Hex/channels, Appearance color and General timeout
use one field model. Caller-side rules remain explicit:

| Consumer | Retained policy |
| --- | --- |
| Picker | Remove control characters; preserve path resolution, selected replacement, cursor-following caret marker and explorer navigation |
| RGB | Sanitize terminal text, replace newlines/tabs with spaces, retain 16-byte limit, preserve incomplete/invalid text |
| Appearance | Existing color syntax, typed-input guard and 32-byte paste validation; preview remains staged |
| Timeout | Decimal digits, five input bytes, validated 1–60000 ms only on confirmation |

Fields keep the caret on a grapheme boundary after insertions and deletions,
including edits that join regional indicators. Shared controls do not know paths,
colors, shortcut IDs, App, Ratatui or clipboard transport.

Painting is shared where semantics match. Picker retains its explicit caret
marker and horizontal scrolling; RGB retains its reverse-cell caret and gauges;
Settings retains its compact fixed-control edit presentation. These are deliberate
visual differences, not separate editing implementations.

Cyclic navigation is shared by Tools groups, picker focus, Settings focus,
presets and leader prompt choices. Color and character grids use the same bounded
navigation. RGB keyboard and mouse changes update the same draft operation;
only Apply emits a document color command. Scrollbar grab mapping is shared by
Tools, picker, Settings, text surfaces and file/review dialogs, while owners keep
their distinct click/scroll policies.
The existing shared text surface remains the editor/review scroll owner.

No universal widget hierarchy or extra prompt/radio framework was introduced.
The reusable units are the mechanics actually shared by consumers. Explorer
discovery/selection remains with ratatui-explorer, and prompt action semantics
remain with the workflow that knows what confirmation means.

## Completion ownership

A field copy captures its text before focus can change and owns its pending
write. Color owns a pending paste together with the originally requested field;
completion cannot be redirected by the current Tools selection. Picker already
owns its original-target read and now uses the shared Field for insertion.
App polls these owners and publishes results; it no longer carries RGB request
or field-write transport state directly. The shared clipboard service still
serializes writes and preserves delivery/failure distinctions.

## Verification

| Check | Result |
| --- | --- |
| Tests | 171 passed: 16 library, 130 binary, 24 integration, 1 doctest; 1 debug performance test ignored |
| Formatting and whitespace | `cargo fmt --all --check`, `git diff --check`: pass |
| Clippy | `cargo clippy --locked --all-targets -- -D warnings`: pass |
| Terminal PTY | Tools/RGB, picker fields, remapped shortcuts, modal return, resize, persistence and terminal restoration: pass |
| Settings/theme PTY | Preview, discard, explicit Save and restart: pass |
| Leader PTY | Focus, capture, conversion warning, Save and unset: pass |
| Editor clipboard PTY | Styled copy/cut/paste, review, atomic undo, persistence and cleanup: pass |

New contract tests cover grapheme edits, selection/limits, partial grid rows,
page movement, cyclic focus, range clamps, pointer/key RGB equivalence, invalid
draft rejection and retained drafts across arrangements. Existing tests cover
swatch spacing, independent scrolling, style/character actions, narrow layouts,
clipboard target ownership, picker controls and Settings fixed navigation.

Logs: `/tmp/tuxd-controls-stage6/`. No new real-device walkthrough or performance
claim is made for this stage; final integration retains those gates.

## Remaining boundaries

- Stage 7 owns file/import/review workflows and common storage mechanics. App's
  picker polling still discovers the active Files dialog; that modal coordination
  is not a control responsibility.
- Stage 8 owns final Workspace/root composition and the visibility/import audit.
  Workspace arrangement and active-context policy remain outside Tools.
- Product features and persisted formats/defaults are unchanged.
