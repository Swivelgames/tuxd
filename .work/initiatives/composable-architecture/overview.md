# Composable architecture refactor

Status: implementation complete. All eight stages are complete.
See the [ANSI](stage-01-ansi.md), [Clipboard](stage-02-clipboard.md),
[Text/session](stage-03-text-session.md), [Shortcuts](stage-04-shortcuts.md),
[Themes/Settings](stage-05-themes-settings.md),
[Controls/Tools](stage-06-controls-tools.md) and [Workflows/I/O](stage-07-workflows-io.md) records.
See the final [Composition/Cleanup](stage-08-composition.md) record.

## Objective

Simplify tuxd into independent capabilities and explicit local orchestration,
preserving accepted functionality. Apply the same reasoning from statements
through application composition. Fewer caller obligations, duplicated rules, and
implicit relationships are the measures of progress; file count and lines moved
are not. Ask what is unique, what can be isolated, and what must be orchestrated.

## Reading order and authority

| Document | Purpose |
| --- | --- |
| [Architecture](../../../docs/architecture.md) | Implemented composition, dependency rules, ownership, organization |
| [Readability](../../../docs/code-organization-and-readability.md) | Engineering guidance ported from agnostic |
| [Simplification inventory](simplification-inventory.md) | Current evidence and complexity to eliminate |
| [Capability contracts](capability-contracts.md) | Library/component interfaces, lifecycle, workflows |
| [Implementation plan](implementation-plan.md) | Stages, prerequisites, deliverables, completion gates |
| [Validation](validation.md) | Behavior matrix, checks, baseline, outstanding verification |

Architecture describes implementation; inventory retains the original evidence and
records completed simplifications. Historical contract sketches remain illustrative. Update
the plan and evidence together; planned interfaces and previous acceptance are
not completed refactor work.

## Agreed decisions

| Decision | Consequence |
| --- | --- |
| Simplify before separating | A move preserving caller bookkeeping is not a completed boundary |
| Mini-libraries in one Cargo package | No Cargo workspace or published crates required |
| Structured document is authoritative | ANSI encodings and visual layout are derived |
| Complete document operations | Callers never repair offsets or normalize after mutation |
| Independent shortcuts and catalogs | Features contribute metadata; Settings queries it; views report contexts |
| Independent clipboard transports | Callers choose payloads and interpret receipts; drivers know no ANSI/editor policy |
| Shared text component | Editor, Code, review share layout/scroll/hit mechanics with different presentations |
| Local orchestration | Code owns copy-code behavior; workflows own decision ordering |
| Stable registration, instance lifecycle | Hidden panels retain discoverable actions; remount does not reset bindings |
| Narrow dependencies | No whole-App parameter to make extraction convenient |
| Behavior preservation | No hidden change to workflows, defaults, formats, or limits |
| Keep useful dependencies | Retain Ratatui, ratatui-which-key, and ratatui-explorer where they meet the need |

## Scope and exclusions

Includes document operations, editing, Code, clipboard, shortcuts, themes,
controls, focus, panels, file/import workflows, Settings, storage/terminal
boundaries, tests, and documentation. Deliver the whole-application refactor in
complete paths rather than a disruptive rewrite.

Preserve .tuxd.json and configuration contracts, defaults, limits, history,
review decisions, cleanup, and clipboard delivery distinctions. Baseline remains
removed; do not reintroduce aliases/migrations. Existing shortcut legacy support
must not disappear incidentally while configuration code moves.

Deferred: character search, component-emoji rendering, formatting-only clipboard,
multiple documents, Vim mode, code generation, autosave/recovery, remote clipboard
reads, new platforms, large-file engineering, and new themes/defaults. See the
[product backlog](../../deferred-work.md).

## Starting point and checkpoint

- Branch: `refactor/code-organization`.
- Original base: `006372d` (`feat: add unified settings and customizable themes`).
- Preparation and preliminary App/Files/Settings extractions were committed as
	`b674470` (`refactor: checkpoint organization and composable architecture plan`)
	and pushed to `origin/refactor/code-organization` before Stage 1 began.
- Preliminary modules shared broad imports, whole-App access, and large state.
	The eight stages replaced those transitional boundaries.
- Stage 1 replaces the file-owned ANSI parser and mutable document/style access
	with an independent library contract. It was committed and pushed as `c794ba6`.
- Stage 2 isolates clipboard transport and gives Code ownership of its copy request.
	It was committed and pushed as `94ccc79` before Stage 3.
- Stage 3 separates Editing, DocumentSession, and shared text presentation. Its
	implementation was committed and pushed as `d8ac7ce`.
- Stage 4 supplies independent shortcut catalog/configuration/compiler/resolver
	contracts and local typed dispatch. It was committed and pushed as `4f5a535`.
- Stage 5 supplies independent themes, typed roles, Settings page/mode ownership
	and configuration commit coordination. It was committed and pushed as `e16893e`.
- Stage 6 shares controls and isolates Tools ownership. It was committed and pushed
	as `df140a3` before Stage 7.
- Stage 7 separates import review and publication workflows and shares storage
	mechanics. It was committed and pushed as `f1dc426`.
- Stage 8 completes Workspace composition, local clipboard transactions, typed
	file effects and migration cleanup. It is committed and pushed as `3aa3b44`.
- The subsequent [readability pass](readability-pass.md) applies the shared
	Workyard/agnostic guidance to the completed structure.
- The agnostic reference repository was not modified.

Keep stage-local changes reviewable. A stage record identifies completed contracts,
remaining bridges, and validation; a passing stage does not complete the initiative.

## Completion criteria

- Libraries work without application objects; callers do not repair invariants.
- Repeated mechanics have one owner, with policy differences explicit.
- Features handle their own actions without unrelated central branches.
- Focus, input sequences, asynchronous targets, drafts, and workflows have
	explicit lifecycles and regression coverage.
- Accepted behavior passes the validation matrix; limitations remain accurately
	documented.
- Temporary bridges are removed or have a named remaining consumer/removal stage.

## Remaining design choices

No unresolved design decision remains from these eight stages. Type names, enum versus trait, file
granularity, and cache shape are implementation decisions constrained by the
contracts. If simplification exposes ambiguous behavior, inspect tests and prior
acceptance. Record any proposed product change separately rather than adopting
it implicitly during refactoring.
