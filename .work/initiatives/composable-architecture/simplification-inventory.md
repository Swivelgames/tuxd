# Simplification inventory

Status: original inspected evidence plus stage outcomes. Evidence paths below
refer to the pre-Stage-1 extraction branch. This inventory describes obligations
to eliminate, not just destination directories.

## Completed simplifications

| IDs | Completed outcome | Completion |
| --- | --- | --- |
| S01 | Private document text/styles; complete edits; independent Editing and DocumentSession ownership | Completed in Stages 1, 3 |
| S02 | Semantic styles have no Ratatui imports; explicit adapter and shared text renderer | Completed in Stages 1, 3 |
| S03 | CLI, Files, and clipboard all call the independent ANSI parser | Typed source/destination review complete in Stage 7 |
| S04 | Native codec and validation separated from filesystem operations | Shared storage and codec reexport removal complete in Stage 7 |
| S05 | Direct encoders; Code owns copy, projection, syntax tokens, encoded row offsets, source-row mapping, viewport and read-only rendering | Catalog and local handler complete in Stage 4 |
| S06 | Code copy serialization/completion removed from central dispatch | Local dispatch complete in Stage 4; final Workspace composition complete in Stage 8 |
| S07 | Independent clipboard service owns worker requests and typed receipts; callers retain policy and notices | Field completion owners complete in Stage 6; review decoupling and owned picker polling complete in Stage 7 |
| S08 | Clipboard targets include stable session identity plus existing editing checks | Explicit paste-review destination and revalidation complete in Stage 7 |
| S09 | Pure display/layout library and one text renderer used by Editor, Code, review | Completed in Stage 3 |
| S10 | Generic scrollbar primitive has no pane IDs; text surfaces own bars/drag | Tools/Indexed routing bridge removed in Stage 6 |
| S11 | Editing has no viewport/widgets; panels own shared surfaces and coordinate cursor reveal | Completed in Stage 3 |
| S12 | Supplied catalog/configuration/compiler/resolver has no paths, application types or concrete keys | Complete in Stage 4; common storage complete in Stage 7 |
| S13 | Feature-owned definitions assembled independently of visible instances; effective contexts reported by owners | Complete in Stage 4; full Workspace state complete in Stage 8 |
| S14 | Typed role/group catalog owns IDs, labels, order and resolution; Appearance uses identities | Complete in Stage 5 |
| S15 | Application configuration directory and supplied theme destination remove shortcut-storage coupling | Complete in Stage 5 |
| S16 | Explicit configuration commit coordinator validates and preserves rollback/error ordering | Coordinator complete in Stage 5; common storage complete in Stage 7 |
| S17 | Shared field model, grid/range/cycle mechanics; caller-specific validation and painting | Complete in Stage 6 |
| S18 | Style/Color/Characters own local state/layout; Tools owns input/view/geometry and typed effects | Complete in Stage 6 |
| S19 | Page-owned drafts and exclusive interaction modes; no whole-App Settings integration | Complete in Stages 5–6 |
| S20 | Typed picker/review outcomes; explicit source and destination; publication updates session only after success | Stage 7 complete; final App/Workspace shell complete in Stage 8 |
| S21 | Session owns identity/editing/path/origin/checkpoint; pane text geometry moved out of App | Workspace arrangement/focus and root composition complete in Stage 8 |
| S22 | Explicit production imports, private component geometry, removed compatibility modules and dead renderer/state | Complete in Stage 8 |

See [Stage 1](stage-01-ansi.md), [Stage 2](stage-02-clipboard.md),
[Stage 3](stage-03-text-session.md), [Stage 4](stage-04-shortcuts.md),
[Stage 5](stage-05-themes-settings.md), [Stage 6](stage-06-controls-tools.md), and
[Stage 7](stage-07-workflows-io.md) for contracts,
bridges, and verification.
All opportunity rows below are addressed. Stage 8 removes the styling facade,
whole-App feature methods, root pane geometry, and production parent wildcard
imports; deletes unreachable menu rendering and obsolete focus-follow state;
and narrows private geometry visibility. See [Stage 8](stage-08-composition.md).

## Observed opportunities

| ID | Current evidence | Complexity / caller obligation | Simplification and retained orchestration | Stage |
| --- | --- | --- | --- | --- |
| S01 | `document.rs`, `styling.rs`, `editor.rs` mutate text/styles and normalize after edits | Consumers know offsets, style coverage, and repair order | Validated ANSI document operations own mutation invariants; Editing chooses inheritance/history | 1 |
| S02 | `styling.rs` has both ANSI serialization and `Ink::render` / `TextStyle::render` | Domain types depend on Ratatui presentation | Keep semantic types/encoding in ANSI; move rendering translation to adapter | 1 |
| S03 | `files/ansi_import.rs` is imported by CLI and editor clipboard | Shared content parsing appears file-owned | ANSI parser returns candidate/diagnostics, with no file or dialog knowledge | 1 |
| S04 | `persistence.rs` combines `FileDocument`, validation, encoding, reads, atomic writes | Formats and filesystem effects cannot be used separately cleanly | Native format adapter consumes validated data; Storage handles bytes/policies | 1, 7 |
| S05 | `styling::clipboard_text` calls `code(document).replace('\n', "")` | Clipboard representation depends on display-only Code formatting | Direct escaped encoder for copy/export; Code projection adds presentation | 1, 2 |
| S06 | `app/actions.rs` handles editor movements, Code scrolling, tools, field copies, file actions | Root knows mechanics and state of unrelated features | Local typed handlers; root routes owners and cross-feature policy only | 2–4, 8 |
| S07 | `app/clipboard.rs` polls picker, editor, and RGB operations; `clipboard::finish` formats notices | Generic completion mixes transport, target policy, and UI language | Clipboard service returns identified receipts/errors; original local coordinator interprets them | 2, 6–7 |
| S08 | `editor_clipboard.rs` implements App methods and opens `files::Dialog::Review` | Clipboard editing reaches through file modal state and whole App | Guarded edit transaction plus shared review result; keep edit policy separate from transport | 2, 7 |
| S09 | `document.rs` includes display fallback/width, `layout.rs` maps rows, `ui.rs` paints them | Semantic text and terminal display policy are interleaved | One pure display/layout contract shared by painting and hit/movement consumers | 3 |
| S10 | `scrollbar::Target` names Editor, Code, Tools, Indexed | A reusable scrollbar requires application routing knowledge | Scrollbar reports local offset changes; owner maps result to its viewport | 3, 6 |
| S11 | `editor.rs` mixes history/mutations and viewport geometry/scrolling | Using editing pulls in display state; Code duplicates viewport behavior | Editing session owns semantics; shared text surface owns viewport; pure mapping bridges movement | 3 |
| S12 | `shortcuts/mod.rs` combines catalog-dependent validation, matching, paths, reads/writes | Resolver construction and persistence know app-specific definitions/environment | Supplied catalog plus config codec/compiler/resolver; application setup supplies paths | 4 |
| S13 | `shortcuts/registry.rs` defines concrete InputScope and UI categories | Engine cannot stand alone without application vocabulary | Feature-owned descriptors assembled once; Settings enumerates catalog; context rules are data | 4 |
| S14 | `theme.rs` repeats role strings in lookup/resolution; Settings uses numeric role indices | Several lists must evolve in lockstep | Typed role descriptors and one role catalog drive resolution and Appearance | 5 |
| S15 | `theme::load` derives its path from `shortcuts::config_path` | Theming depends on unrelated shortcut persistence | Application configuration paths passed to codecs/storage; themes resolve in memory | 5 |
| S16 | Theme/Shortcuts separately implement temp-file JSON writes; Settings coordinates restoration | Shared publication mechanics duplicated; policies risk being accidentally equated | Focused Storage primitive plus format codecs and explicit multi-config save coordinator | 5, 7 |
| S17 | `files/picker/mod.rs::Field`, `panel.rs::edit/move_caret`, Settings field handling | Caret/selection/navigation mechanics repeated with different business validation | Shared fields/control operations; caller-owned filename/RGB/timeout validation | 6 |
| S18 | `panel.rs` contains style/color/character state and index-based groups | One control list reconstructs several distinct capabilities | Independent tool components composed by Tools; shared grid/range/field mechanics | 6 |
| S19 | `settings/mod.rs` has multiple capture/edit/chooser flags; `settings/input.rs` branches over them | Many combinations and event-priority rules live in a large handler | Explicit mutually exclusive interaction states, page-owned state, common controls | 5–6 |
| S20 | `files/workflow.rs` implements most operations on App; review tracks stdin/clipboard flags | Whole-App access and source classification mix with dialog mechanics | Typed source metadata and workflow transitions; picker/review return decisions | 7 |
| S21 | `app/mod.rs` exposes document/pane/tool geometry/focus/request state; `ui.rs::draw` changes layout focus | Global state couples rendering, dispatch, requests, and features | Session/workspace ownership, explicit layout reconciliation, component geometry, narrow render inputs | 3, 8 |
| S22 | Extracted modules rely heavily on `use super::*`; private methods widened for sibling tests | File splitting did not isolate contracts or dependencies | Replace broad imports during capability work; test public behavior or colocate truly private unit tests | All |

Stage references are defined in the [implementation plan](implementation-plan.md).
Every migrated row should record what caller knowledge actually disappeared.

## Simplification candidates within functions and state

| Pattern | Proposed treatment | Important distinction |
| --- | --- | --- |
| Paired Move/Select action branches | Map to movement plus selection mode, then one editing call | Action IDs and configurable bindings remain distinct |
| Repeated poll/empty/disconnected handling | Shared request completion transport | Paste/cut target rules remain operation-specific |
| Dense style/range manipulation | Named validated edits with normalization inside the owner | No per-edit caller repair helper |
| Numeric role/field/focus indices | Typed role/target identifiers where mixed meanings cause ambiguity | Loop positions can remain indices |
| Multiple optional modal states | One explicit interaction state when mutually exclusive | Independent state, such as a viewport and a pending receipt, should stay independent |
| Repeated prompt/field painting | Small reusable controls and appearance adapters | Prompt meaning and validation stay at the caller |
| Multi-stage functions | Guard invalid cases; name meaningful prepare/transform/commit stages | Do not extract every statement or reorder external effects |

## Do not merge these differences

- Display wrapping versus content newlines; fallback glyphs versus stored text.
- Raw ANSI versus escaped ANSI versus Code presentation.
- Authoring colors versus UI theme roles.
- Text-field mechanics versus filename, color, and numeric validation.
- Editor clipboard transactions versus plain field copy/paste.
- Clipboard transport success versus confirmed client delivery.
- Current document revision versus saved-content equality versus session identity.
- File/clipboard/stdin review mechanics versus each caller's commit operation.
- Document no-clobber/overwrite policy versus configuration publication policy.
- Key capture UI versus key normalization/matching/validation.
- Common control navigation versus product-specific action availability.

## Preliminary extraction disposition

| Working-tree change | Keep as useful starting point | Still required |
| --- | --- | --- |
| Settings moved out of Shortcuts | Independent feature entry point | Remove shared giant state/whole-App integration; define page contracts |
| Settings category split | Separates Settings pages from action categories | Catalog-driven metadata and engine independence |
| App method modules | Makes existing flows easier to locate temporarily | Move actual ownership to features; eliminate central execution/polling internals |
| Files state/view/workflow split | Distinguishes rendering and operations | Narrow workflow contracts and shared source-independent review |
| Picker/review directories | Makes capability code discoverable | Extract reusable controls; remove file-specific assumptions |
| ANSI parser moved under Files | No durable architectural benefit | Relocate under independent ANSI capability in Stage 1 |
| Tests extracted to sibling modules | Can aid navigation | Preserve meaningful coverage without widening production APIs for tests |

No wholesale reset is needed. Stage-local replacement lets the app remain runnable
and preserves user changes. Do not present the preliminary paths as the target map.

## Observable measures of improvement

For each stage record concrete examples: a caller no longer normalizes runs, a
scrollbar no longer imports pane IDs, theme resolution no longer finds paths,
or a Code copy result no longer invokes editor-cut completion. Review changed
interfaces and their consumers together. Reduced line count is optional; fewer
dependencies and coordinated steps are the relevant evidence.
