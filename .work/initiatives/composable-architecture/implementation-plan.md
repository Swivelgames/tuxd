# Implementation plan

Status: Stages 1–8 complete. See [Stage 1](stage-01-ansi.md),
[Stage 2](stage-02-clipboard.md), [Stage 3](stage-03-text-session.md),
[Stage 4](stage-04-shortcuts.md), [Stage 5](stage-05-themes-settings.md),
[Stage 6](stage-06-controls-tools.md), [Stage 7](stage-07-workflows-io.md), and the
[validation ledger](validation.md). The final [Stage 8 record](stage-08-composition.md)
documents composition, bridge removal, validation and device coverage limits.

## Execution rules

- Deliver complete behavior paths and keep the application runnable at stage gates.
- Before editing, inspect the responsibility's current consumers and tests, name
	its invariants, and state which caller obligations will disappear.
- Use narrow compatibility facades only while migrating named consumers. Keep
	one authoritative implementation; do not maintain old/new mutable documents.
- Change internals without changing accepted output, defaults, IDs, limits, or UX.
- Preserve chronological order of effects, failure handling, and cancellation.
- Complete meaningful tests and final-file readability review before marking a
	stage complete. Passing tests alone does not establish separation.
- Do not commit/push/merge without an applicable user instruction. This plan is
	not blanket authorization for repository publication.

## Stage map

| Stage | Deliverable | Depends on | Status |
| --- | --- | --- | --- |
| 0 | Documentation, inventory, reproducible preparation baseline | Existing branch | Prepared |
| 1 | Independent ANSI document and representation boundaries | 0 | Complete |
| 2 | Complete Code-copy path and clipboard service | 1 | Complete |
| 3 | Shared text surface and session/editing separation | 1–2 | Complete |
| 4 | Independent shortcut catalog, resolver, local dispatch | 2–3 | Complete |
| 5 | Independent themes and Settings page/configuration contracts | 4 | Complete |
| 6 | Shared controls and locally composed Tools/Settings/picker | 3–5 | Complete |
| 7 | Explicit file/import/configuration workflows and storage | 1–6 | Complete |
| 8 | Final workspace/root composition and bridge removal | 1–7 | Complete |

Dependencies describe the preferred sequence, not a ban on a small prerequisite
extraction. For example, Stage 1 may extract a minimal storage boundary, and Stage 2
may introduce a stable session identifier before Stage 3 completes session ownership.
Record such work in the stage that implements it; avoid a speculative upfront framework.

## Stage 0 — prepare and reconcile

Record the current branch/base, tracked/untracked move inventory, passing core/PTy
baseline, target architecture, capability contracts, preservation matrix, and first
stage worklist. Distinguish prior user manual acceptance from new verification.

Preparation does not certify the preliminary source organization. Before the next
source change, read the complete relevant files, reconcile untracked replacements,
and record any unrelated changes so they are not overwritten. No new extraction
is justified solely by the old subsystem-first proposal.

Exit: documentation links are valid; plan has explicit success/failure gates; the
current source is runnable; no product question blocks Stage 1.

## Stage 1 — ANSI document and representations

### Worklist

1. Characterize exact raw/escaped/native outputs using existing tests/examples,
	including empty/trailing-newline, backslash, Unicode, and all style/color cases.
2. Define authoritative document/fragment types and checked positions/ranges;
	decide invariant ownership before changing storage representation.
3. Consolidate complete insert/remove/replace/extract/style operations. Preserve
	grapheme merging and inheritance semantics; remove caller repair sequences.
4. Move ANSI parsing and diagnostics out of Files. Keep source spans and existing
	rejection/strip behavior; preserve explicit raw/escaped modes.
5. Separate raw/escaped encoders from Code's presentation formatting. Adapt copy,
	export, Code, and native-format consumers without changing output.
6. Move Ratatui conversion out of semantic style types. Retain display behavior
	through an explicit adapter.
7. Separate native document data/codec from filesystem operations. Keep current
	versioned JSON and validation behavior.

Temporary bridge: existing Document/Styles import paths may re-export the new
implementation while consumers migrate. No second authoritative buffer. Record
every remaining bridge and its removal stage.

Exit: ANSI can be exercised without App/Ratatui/Crossterm/I/O; edits return valid
state; all input paths use the same parser and output paths use the same encoders;
raw/escaped/native contracts and current editing behavior pass regression checks.

## Stage 2 — Code copy and clipboard

Introduce the service/request/receipt boundary over existing transports. Keep
worker deadlines and main-thread terminal writes. Move transport notices to
consumers, and identify result owners explicitly. Preserve read/write capability
differences and remote-failure behavior.

Create a Code-owned handler that borrows the active document, encodes escaped
ANSI directly, starts a write, and presents its own completion. Route its current
action through a small bridge until Stage 4 reorganizes the catalog. Do not change
the persisted Copy action identity to make the conceptual handler name match.

Adapt editor/field consumers to shared transport without merging their policies.
Keep stale-target checks and cut-after-success semantics. Deferred file/RGB polling
bridges must have named removal stages, not be reported as a completed service migration.

Exit: complete Code copy uses no editor-cut completion path or display string
postprocessing; exact payloads and all three write transports pass. Unit fakes
exercise failure/unconfirmed/stale completion, and requests cannot route by new focus.

## Stage 3 — text surface, editing, and sessions

Extract pure display/layout from document semantics; preserve a single width and
fallback mapping shared by rendering, navigation, and hit testing. Establish the
shared text surface, local viewport/scrollbars, and content presentations for Editor
and Code. Add review as a consumer when its projection contract is ready.

Move cursor/selection/history/pending style into Editing and identity/file/saved
metadata into DocumentSession. Keep history representation unless an actual
invariant requires change. Maintain narrow pure layout input for visual movement.

Code's projection owns encoded offsets, syntax classes, and source mapping; its
viewport is read-only. Invalidation covers session replacement, content, width,
wrap, display policy, and appearance at the appropriate level.

Exit: both panes use shared mechanics without a widget-dependent editing core;
dirty state survives undo-to-saved; read-only cannot mutate; repeated resizing,
scrolling, selection, fallback rendering, and wrap tests pass.

## Stage 4 — shortcuts and focus

Separate catalog/configuration/compiler/resolver from paths and concrete views.
Feature-owned descriptors assemble into one stable catalog before persisted
bindings load. Keep ratatui-which-key and existing key normalization/serialization.

Introduce component-local typed handlers and explicit focus/context coordination.
Move execution out of the central action match by responsibility; do not replace
it with an equally broad callback/service registry. Shared field actions retain
their current IDs and reuse semantics. Hidden panels remain catalog consumers.

Characterize effective scope overlap and modal eligibility, then preserve it.
Cancel pending sequences on the same transitions; Settings fixed controls remain
independent of configurable dispatch.

Exit: library tests use supplied descriptors without app types; catalog completeness,
repeated construction, conflict/prefix/leader behavior, persisted configuration,
inline hints, modal focus, and scope-change cancellation pass.

## Stage 5 — themes and Settings contracts

Unify role descriptors and remove numeric-role coupling in Appearance. Keep preset
values, IDs, overrides, validation, and Midnight default. Separate loading paths
from resolution and provide resolved appearance through a narrow render context.

Give Settings shell and General/Shortcuts/Appearance pages explicit responsibilities.
Model exclusive interaction modes; draft capture/preview stays local, validation
uses libraries. Extract configuration-save orchestration with existing rollback
semantics; shared storage mechanics can be completed in Stage 7.

Exit: theme resolution works in memory; catalog drives Appearance; previews do not
alter content; Save/Cancel/failure and fixed controls pass; no new migration or
default change; existing staged leader behavior remains intact.

## Stage 6 — common controls and Tools

Compare actual Field implementations before extracting shared text mechanics.
Align caret/selection/navigation and painting where semantics match; retain each
caller's sanitization/validation and clipboard transform. Extract list/grid/range/
radio/tab/prompt/scroll primitives only where the consumers need them.

Compose Style, Color, and Characters as owned components. Make RGB draft state
explicit, including incomplete/invalid field text and synchronized valid values.
Use the same tool instances in sidebar/slim arrangements. Move Settings/picker
controls to shared mechanics and finish any field clipboard completion bridges.

Exit: keyboard/mouse converge on shared operations; no app-specific IDs in generic
controls; color spacing, independent scrolling, Apply/reset, field editing,
character insertion, and Settings/picker navigation remain equivalent.

## Stage 7 — workflows and I/O

Make picker and review return typed selections/decisions. Introduce source metadata
for files/stdin/clipboard and explicit workflow state for pending operations.
Remove clipboard's dependency on Files modal state. Preserve load-validation,
dirty/overwrite/review order, cancel/retry, source directories, and saved state.

Consolidate bounded reads/atomic publication mechanics while retaining per-caller
limits, overwrite/no-clobber behavior, permissions, and error distinctions. Finish
config save orchestration without promising crash-atomic multi-file writes.
Keep CLI OS-string parsing and stdin-to-controlling-terminal handoff unchanged.

Exit: file/stdin/clipboard share parse/review mechanics but commit differently;
failures do not replace active content; delayed edits revalidate; native/ANSI
files, config restoration failures, and terminal startup errors remain covered.

## Stage 8 — final composition and cleanup

Reduce App to assembly/runtime coordination and Workspace to session/layout/focus
policy. Remove obsolete state, broad parent imports, abandoned helpers, duplicate
implementations, and compatibility facades after migrating all consumers. Check
visibility against the actual contracts rather than widening it for tests.

Read every changed source/test file in its final state, compare equivalent peers,
and audit the dependency direction and complete behavior traces. Keep integration
tests at composition boundaries and library tests focused on invariants.

Exit: full validation matrix, relevant graphical/manual checks, and performance
measurement meet preserved contracts; no unexplained bridge remains; architecture
and README describe implemented behavior; completion is recorded honestly.

## Checkpoint record template

For each stage record: scope, source baseline, interfaces changed, consumers
migrated, caller obligations removed, temporary bridges, checks/results, remaining
limitations, and next stage. A planned checkbox is not evidence. Only mark Complete
when both behavior and architectural gates pass.

If a stage fails, keep its issue local and repair or revert only that stage's own
changes. Do not discard unrelated pending work, weaken assertions, broaden scope,
or mark the stage complete because the source compiles.
