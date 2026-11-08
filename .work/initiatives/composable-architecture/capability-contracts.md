# Capability and composition contracts

Status: target contracts with implemented ANSI, clipboard, editing/session, and
shared text boundaries recorded in [Stage 1](stage-01-ansi.md),
[Stage 2](stage-02-clipboard.md), [Stage 3](stage-03-text-session.md), and [Stage 4](stage-04-shortcuts.md).
Shortcut engine/catalog/local dispatch, theme/Settings, shared controls, and
workflow/storage contracts are implemented. Final App/Workspace composition remains
a target. Read with the
[architecture](../../../docs/architecture.md) and [migration plan](implementation-plan.md).
Names below communicate responsibilities; choose concrete Rust types during the
relevant stage. Do not turn every heading into a new file, trait, or crate.

## 1. ANSI / styled document

### Authoritative state

The document owns text and normalized style runs. Public mutation cannot leave
invalid positions, gaps/overlaps in style coverage, or invalid color values.
Untrusted structured data is a transport representation until validated into a
document. Existing empty-document and trailing-newline semantics are preserved.

| Part | Responsibility |
| --- | --- |
| Vocabulary | Colors, attributes, style changes, text positions/ranges |
| Model | Authoritative text and normalized styles |
| Operations | Insert, delete, replace, extract fragment, apply/clear styling |
| Parser | Raw/escaped notation and supported SGR semantics |
| Diagnostics | Malformed/unsupported controls, original source spans and reasons |
| Encoders | Raw ANSI, escaped ANSI, structured data |
| Data representation | Serializable content without UI/session state |

Positions must distinguish bytes, logical lines, and display cells where confusion
is possible. A position is valid for a particular document state; checked mutation
must reject invalid input without partial modification. Style boundaries respect
graphemes, including edits that merge previously separate code points. Preserve
the existing leading-character style rule for merged graphemes.

Illustrative operations:

```rust
document.insert(position, fragment)?;
document.replace(range, fragment)?;
document.apply_style(range, change)?;
document.clear_style(range)?;
let fragment = document.extract(range)?;

let raw = document.to_raw_ansi();
let escaped = document.to_escaped_ansi();
let data = document.to_data();
```

An operation completes normalization internally. Consumers may query content but
must not obtain unrestricted mutable style runs or repair offsets after calls.
Avoid duplicating a fully mutable model and builder unless construction actually
needs different guarantees. Validated constructors plus complete operations may
be sufficient.

### Parsing and encoding

Parsing returns a candidate document and diagnostics, with original source retained
or otherwise available for review. Diagnostic spans refer to the input, not the
decoded output. Ambiguous unterminated controls retain existing rejection behavior.
Unsupported controls are inert data; parsing never executes them or displays a
prompt. Review policy belongs to a caller.

Raw output preserves source text/newlines and accepted SGR serialization. Escaped
output escapes ESC, newlines, tabs, and literal backslashes using the existing
contract, without Code display-only line breaks. Parsing/encoding tests distinguish
semantic roundtrip from exact output stability: imported spelling need not survive
canonical encoding, but currently promised output bytes must remain stable.

The native document adapter retains the versioned .tuxd.json envelope. Serialization
does not require filesystem I/O. Existing version/unknown-field validation and
data limits stay visible at the correct boundary. Do not replace file format
fields just because the internal model changes.

### Dependency limit

The core can use text/Unicode and serialization dependencies; it has no Ratatui,
Crossterm, filesystem, clipboard, Settings, or App dependency. Document-style to
Ratatui conversion belongs in an adapter. Source normalization/safe literal input
handling must be explicit; it is distinct from display-only fallback rendering.

## 2. Editing and document sessions

Editing owns the ANSI document, cursor, selection, pending typing style, preferred
movement column, and undo/redo grouping. Commands are semantic, not terminal keys.
The document owns mutation correctness; Editing owns the user's editing semantics.

| Concern | Contract |
| --- | --- |
| Movement/selection | Same underlying movement primitive with an explicit selection mode; preserve reverse-selection collapse and word boundaries |
| Typing | Preserve inherited/pending style and grapheme-safe cursor placement |
| History | Preserve 200-snapshot bound and current grouping, including pause/navigation boundaries |
| Atomic edits | Paste, newline, tab, character insertion, and formatting retain existing undo semantics |
| Display movement | Consume pure layout information for visual rows and hit positions, without widget/backend dependency |
| Viewport | Presentation owns scrolling/layout state; Editing and its panel coordinate reveal-cursor behavior |

Separating viewport state must not change when scrolling follows the cursor,
preferred-column behavior, or wrapped versus logical-line movement. A narrow pure
navigation/layout contract is enough; do not create an Editing ↔ widget dependency
cycle.

Application `DocumentSession` owns stable identity, Editing, file association,
origin metadata, and saved checkpoint. Session identity differs from revision;
replacing a document must invalidate old asynchronous targets even if revisions
coincide. Undo returning to saved content must still clear dirty state; a simple
monotonically increasing revision comparison is insufficient for that behavior.

Only one active document is required now. This ownership permits future multiple
sessions without implementing tabs or workspace metadata in this refactor.

## 3. Shortcuts and focus

### Definitions and runtime

| Object | Responsibility |
| --- | --- |
| Action descriptor | Stable persisted ID, label, category/subsection, defaults, eligible contexts |
| Context descriptor | Stable identity and matching/validation rules, including editable-text restrictions |
| Catalog | Enumerate/lookup definitions, detect duplicate identities, supply Settings and inline hints |
| Binding configuration | Three slots per action, sequences, leader, timeout, encoding/decoding |
| Compiled map | Validated effective bindings, with leader resolved for conflict analysis |
| Resolver | Process normalized keys/context/time and return action, pending, consumed, or unbound |
| Key adapter | Translate terminal-reported events and normalize indistinguishable key encodings honestly |

Feature modules own definitions; application assembly combines them once. The
catalog is complete before saved bindings are loaded. Hidden/unmounted panels
remain represented in Settings. Registration never overwrites saved bindings.
Default inheritance, legacy configuration handling, and validation stay equivalent
to the current implementation.

The resolver can remain backed by ratatui-which-key. Encapsulate the dependency
behind its real responsibilities; do not reimplement matching or add a competing
registry framework without an evidenced need. A catalog can be assembled from
static descriptor slices rather than runtime plugins.

The resolver returns action IDs. Instance-local typed handlers associate actions
with operations; avoid persisted closures or callbacks capturing mutable panels.
The engine knows identifiers/metadata, not concrete application widgets.

### Focus lifecycle

Components report focus changes. Application focus coordination determines the
effective context, especially across modals. Preserve currently eligible actions;
do not invent new parent/global bubbling precedence in this refactor. Explicitly
characterize collisions and modal exclusions before changing dispatch structure.

On context change, cancel pending sequences. Preserve cancellation on mouse,
paste, resize, focus loss, timeout, and opening Settings where currently applied.
Opening a modal remembers eligible return focus; closing restores it if the target
still exists. Hidden/collapsed targets are revealed according to current pane rules.

Component instance identity is distinct from shortcut scope identity. Multiple
instances can eventually share definitions while only the appropriate instance
handles the resolved action. Rendering or remounting never registers duplicates.

### Validation and Settings contract

Resolve leader before comparing sequences. Reject prefix ambiguity within
overlapping effective contexts. Preserve reusable bindings across disjoint contexts
and ordinary-text protection in editable contexts. Preserve hard-coded Settings
openers/navigation and reserved Escape handling. Changing a leader stages the
existing conversions/unsets; removing it retains the accepted literal-preservation
or removal choices. Prompts occur only when existing bindings are affected.

Capture, leader prompts, and draft navigation belong to Settings. Key normalization,
sequence labels, conflict facts, catalog lookup, and configuration validation belong
to Shortcuts. Active bindings change only after successful explicit Save.

## 4. Themes

Themes owns role descriptors, preset descriptors, overrides, color syntax,
configuration data, validation, and resolution. Settings obtains labels/groups
from descriptors rather than parallel numeric-index tables. Typed roles replace
repeated string matches where practical; persisted role IDs remain stable.

Preset plus overrides yields a resolved theme without I/O or ambient application
state. Loading bytes, choosing config paths, and committing a theme are external
operations. Midnight remains default/fallback, all 14 current presets remain, and
Baseline stays absent.

Rendering consumes a resolved theme through an explicit scoped render context or
equally narrow dependency. The context provides appearance, not arbitrary services
or mutable application state. Preview never changes authored document styles.
Changing appearance must not alter ANSI/export/clipboard content.

A Ratatui adapter maps theme roles to styles/widgets. Share a color primitive with
document code only if semantics agree; terminal-default content colors and UI
fallback colors must not be conflated merely because both encode RGB values.

## 5. Clipboard and terminal output

| Service fact | Calling feature decision |
| --- | --- |
| Read/write capability and chosen transport | Offer copy/paste and choose payload |
| Text bytes and transport limits | Choose ANSI, literal text, path, or selection |
| Request identity and completion | Match the original target and update local state |
| Desktop completion or unconfirmed forwarding | Format an honest feature-specific notice |
| Failure/timeout | Preserve document state and offer existing recovery guidance |

Drivers cover desktop helpers, TMUX, and direct OSC 52. Retain current transport
selection and no fallback to the wrong host clipboard after remote failure. Do not
add remote reads; bracketed paste remains an input event independent of reads.
Driver capability descriptions distinguish read support from write support.

The service coordinates worker execution, bounded payloads, deadlines, and request
results. Synchronous fakes support tests. Requests carry identity so a later focus
change does not redirect completion. Define cancellation as ignoring invalidated
results and cleaning up owned resources; do not claim an already transmitted
clipboard write can be undone.

OSC 52 serialization belongs to its driver, but actual terminal output passes
through the serialized terminal writer between frames. No worker writes stdout
concurrently with rendering. A forwarding receipt does not assert client delivery.

### Copy Code orchestration

1. Code contributes its stable actions when the application catalog is assembled.
2. The panel activates its eligible scope when focused.
3. Copy obtains the active session's document and directly encodes escaped ANSI.
4. Clipboard receives text and returns a request identity.
5. Completion returns to the initiating operation; Code presents the receipt.

Viewport position, highlighting, wrapping, and gutters never affect copied bytes.
The root routes an action/result to Code; it does not implement copy serialization.

### Editing clipboard orchestration

Cut captures its original target, writes the selected styled fragment, and only
deletes after accepted transport success if the target remains valid. Preserve the
accepted unconfirmed-forwarding semantics. Copy failure never deletes text.

Paste captures the original session, revision, cursor, selection, and pending style.
On read completion, validate that target and current workflow eligibility. Preserve
current cancellation on intervening changes/modal interactions. Styled/plain/literal
policy is editing-specific; Clipboard does not decode ANSI. Parsing diagnostics go
through the shared review component, with target revalidation before commit.

Code copy, field copy/paste, and editor transactions share transport mechanics but
retain different targeting and transformation policies. Do not merge them into a
universal clipboard workflow with a growing mode switch.

## 6. Presentation and controls

### Shared text surface

The reusable surface owns viewport state, wrapping, gutters, scrolling, mouse hit
testing, cursor/selection painting, clipping, and scrollbar interaction. Content
providers supply text and style spans. Input capabilities explicitly distinguish
editable and read-only consumers. Read-only surfaces have no document mutation
path, even if an action is invoked directly.

| Consumer | Provider | Allowed behavior |
| --- | --- | --- |
| Editor | Document text and authored styles | Existing movement, selection, edits, scrolling |
| Code | Escaped ANSI projection and syntax classes | Existing read-only navigation/copy/wrap |
| Review | Inert original source and diagnostic spans | Existing preview navigation and decision controls |

Common machinery must not accidentally add selection/editing to consumers that do
not currently have it. A provider need not be a trait until alternatives require
one; borrowed view data can be enough.

### Mapping and invalidation

Keep original bytes, logical lines, encoded text offsets, graphemes, wrapped rows,
and terminal cells distinguishable. Code projections retain enough source mapping
for the existing document-linked active gutter. Review highlights original error
spans even when source is escaped for safe display. Cursor hit testing and painting
consume the same width/fallback decisions.

Document revision invalidates serialization; width/wrap/display policy invalidates
layout; theme changes restyle presentation. A session replacement invalidates all
session-derived caches even if revision matches. Cache only when useful; do not add
an elaborate incremental system or accept stale content to optimize prematurely.

Preserve the approved composed-emoji display escapes, wide-glyph clipping, tab
stops, newline selection cells, and tiny-size behavior. These transformations must
never alter the underlying document or exported bytes.

### Shared controls

Stage 6 implements the shared field/grid/range/cycle mechanics and the terminal
field/scrollbar adapters. Tools owns component state, geometry and effects; RGB
owns its draft and pending paste. The [stage record](stage-06-controls-tools.md)
documents retained caller policies and intentional presentation differences.
The table below describes responsibilities, not a requirement for one widget
class per row; prompt/radio behavior composes shared mechanics locally.

| Control | Shared mechanics | Caller policy |
| --- | --- | --- |
| Text field | Caret, selection, editing, scrolling, focus rendering | Filename, hex, numeric, or timeout validation |
| List/grid | Selection, direction/page/boundary navigation, reveal-selected | Files, colors, characters, action entries |
| Range | Bounds, stepping, drag, minimum/maximum | Channel value and apply timing |
| Tabs/radios | Active member, navigation, hit regions | Which view/value is selected |
| Prompt | Buttons, focus, activation, close result | Positive/negative/abort meaning |
| Scrollbar | Orientation, extent, offset, drag mapping | Which owned viewport receives the offset |

Inputs are semantic control operations; adapters translate keys, mouse, and action
IDs. Shared keyboard and mouse paths should converge on the same operation where
their meaning is identical. Each control retains state/geometry locally. Text
sanitization and validation remain explicit policy, not one universal filter.

RGB is a small local model: a staged candidate, temporary possibly-invalid field
text, synchronized valid channel values, and explicit Apply. It composes fields
and ranges; it does not become a special case inside generic controls.

## 7. Panels, workspace, and lifecycle

| Owner | State | Operations / outputs |
| --- | --- | --- |
| Workspace | Active session, arrangement, focus, collapsed panes | Compose panels, reveal/focus target, route cross-feature decisions |
| Editor | Viewport, interaction state, pending clipboard transactions | Editing commands and local status |
| Code | Viewport, wrapping, projection cache, copy request | Read-only output and copy receipt |
| Style | Control focus and active/mixed presentation | Style commands targeting the session |
| Color | Channel target, palette, staged RGB, local focus | Color commands only on existing apply/selection triggers |
| Characters | Catalog/category selection, grid focus | Insert chosen glyph as an edit |

Style/Color/Characters remain the same instances when arranged as sidebar blocks
or slim tabs. Application layout selects bounds and visibility; each component
keeps its behavior. Preserve the <=64-column threshold and current collapse/fill
rules. Mouse, shortcuts, and tab activation share focus transitions.

Feature assembly owns its descriptors, handlers, and view. Root assembly includes
features without knowing every label, field, or action. Use narrow borrowed
dependencies or explicit results; no long-lived mutable reference to Workspace,
global mutable document, or service locator.

Rendering may refresh geometry/layout caches. It must not start requests, persist,
or apply edits. Resize/reconciliation establishes visibility/focus consistently
before input routing. Restoring focus after modal closure must not target a removed
session or hidden component without the normal reveal behavior.

## 8. Document and import workflows

Stage 7 implements typed picker selections, independent review decisions,
file/stdin/clipboard source metadata and explicit document/paste destinations.
`workflows::publication` owns encoding, publication and successful session updates;
`libraries::storage` owns bounded reads and byte publication. See the
[Stage 7 record](stage-07-workflows-io.md) for preserved policies and validation.

Use explicit workflow states and typed outcomes for operations requiring several
decisions. State stores only information valid for that stage. Preserve existing
error/retry/cancel order rather than redesigning UX during extraction.

| Stage/component | Returns | Does not do |
| --- | --- | --- |
| Picker | Path, operation/format choice, or cancellation | Load/replace the active document |
| Storage | Bytes or typed failure | Guess a document format or prompt |
| Decoder/parser | Validated candidate or diagnostics/failure | Install a session or ask the user |
| Import review | Accept stripping or abort | Save files, paste, or replace editor content |
| Dirty/overwrite prompt | A decision | Mutate content or write the destination |
| Workflow coordinator | Next state, service request, or committed outcome | Reimplement any delegated mechanics |

Open/import normally follows source selection → read → parse/validate → optional
review → unsaved-change decision → session installation, preserving actual current
branch behavior. Failed validation leaves the active document intact. Save failure
retains the dirty document and pending decision; cancellation ends only the
appropriate operation. Save/Discard/Cancel remains initially focused on Cancel.

Review source metadata distinguishes file path, stdin, and clipboard. File import
installs an unsaved candidate; clipboard import inserts into its validated target;
stdin import completes startup after terminal handoff. Shared review mechanics do
not imply identical commit behavior.

File browsing retains origin/CWD defaults, filtering, hidden files, navigation,
format radios, path clipboard, and overwrite behavior. Continue using
ratatui-explorer for directory exploration rather than rebuilding discovery.

## 9. Settings and configuration commit

The shell owns navigation, page selection, overall dirty state, fixed controls,
and draft lifecycle. General edits leader/timeout; Shortcuts consumes the action
catalog; Appearance consumes role/preset catalogs. Pages compose shared controls.

Represent mutually exclusive interaction modes directly: browsing, field editing,
binding capture, leader capture/conversion decision, preset selection, and discard
confirmation where applicable. Avoid independent flags that admit impossible
combinations. Preserve remembered category/action, expanded subsections, spacing,
narrow layout, scrolling, and invocation-context navigation.

Validation belongs to the relevant library. A save coordinator validates all drafts,
persists according to the existing contracts, and activates configuration only on
success. Failed saving retains drafts and active configuration. Cancel restores
preview without changing persistent data. Existing independent reset behavior stays.

The present two-file save attempts to restore theme data when shortcut saving
fails; it is not crash-atomic across files. Preserve/test this behavior and report
restoration failures. Do not claim stronger durability or consolidate files as an
incidental refactor. Paths come from application configuration setup, not Shortcuts.

## 10. Storage, CLI, and terminal

Storage reads/writes bytes with supplied bounds and publication policy. Document,
theme, and shortcut codecs own their schemas. Shared mechanics must preserve
intentional differences: document no-clobber versus replacement, permissions,
symlink/special-file handling, missing config defaults, and differing size limits.

CLI parses OS strings into explicit startup requests. File or stdin data is consumed
before terminal event handling. Preserve --raw/--escaped with '-' and controlling
TTY handoff; no argument reinterpretation or automatic stdin detection is added.

Terminal owns partial-setup recovery, normal/error/panic cleanup, enhanced keyboard
mode, bracketed paste, focus/mouse modes, and serialized writes. Keep the one-time
keyboard-stack restoration invariant. UI color policy still preserves authored
content regardless of NO_COLOR. The runtime schedules input/completions/timers;
it does not own feature-specific polling branches.

## 11. End-to-end trace requirements

For each migrated path, a reviewer should trace event → context → resolved action
→ local coordinator → capability operation → result → presentation without reading
unrelated features. Trace both success and failure/cancel paths.

At least these paths must remain explicit:

- Code copy through each supported transport, with exact escaped payload.
- Selection cut success/failure/stale completion and one-step undo.
- Styled/plain/literal paste, review accept/abort, and stale target rejection.
- Open/save/overwrite/dirty cancellation without losing the current document.
- Settings preview/cancel/save/failure and leader conflict/conversion handling.
- Wrapped movement, mouse mapping, scrolling, and repeated normal/slim resizing.
- Piped stdin import through terminal setup and all cleanup exits.

The [validation matrix](validation.md) turns these traces into stage evidence.
