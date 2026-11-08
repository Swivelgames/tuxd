# Stage 4 — independent shortcuts and local dispatch

Status: complete on 2026-11-07, committed and pushed as `4f5a535` on
`refactor/code-organization`.
Stage 3 was committed and pushed as `d8ac7ce` before implementation.
Stage 5 (themes and Settings contracts) is next.

## Implemented boundaries

| Owner | Responsibility |
| --- | --- |
| `libraries::shortcuts::Catalog` | Validate supplied action/context identities; provide immutable definitions, defaults and missing-action inheritance |
| `Configuration<A, K>` | Serializable binding data, three slots, leader transformations, sequence labels and inline hints |
| Catalog compiler | Validate version, timeout, normalized/reserved keys, editable contexts, resolved leader and overlapping prefixes; build ratatui-which-key map |
| `Resolver<S, K>` | Consume supplied map, key, effective context and time; return Action/Pending/Consumed/Unbound; cancel/expire sequences |
| `shortcuts::key` | Existing Crossterm normalization, structured key encoding, labels, trusted default parsing and BindingKey adapter |
| Feature `shortcuts` modules | Stable descriptors for Editor, Code, Files/prompts, picker, Tools, pane coordination and shared field actions |
| Application registry | Stable persisted enum, UI vocabulary, immutable descriptor assembly and Settings metadata queries |
| `shortcuts::storage` | Existing XDG path, bounded reads, legacy decoding and atomic publication |
| Application shortcut Runtime | Compose catalog/config/map/resolver; publish active bindings only after successful save |
| Feature handlers and `app::focus` | Execute local typed actions and determine effective modal/control context |

The engine has no App, feature, Settings, Crossterm, terminal, path or filesystem
imports. Public integration tests supply numeric action/context IDs and a tiny
nonterminal key adapter. Ratatui-which-key still owns sequence lookup; there is no
parallel trie or mutable callback registry.

The existing application configuration wrapper is serde-transparent. Its field
shape, snake-case action IDs, structured keys, version 2, three slots, defaults,
legacy conversion, 1 MiB read bound, tab-indented output and failure behavior remain.
The wrapper delegates leader transformations, hints and validation to the library;
it does not duplicate those rules. Filesystem work remains outside that library.

## Catalog assembly and local ownership

Descriptors assemble once through immutable LazyLock values before defaults or
saved bindings load. Construction, rendering, hiding and reopening panels cannot
register actions or overwrite custom bindings. Duplicate action/context identities
and undeclared contexts fail catalog construction.

The shared Copy/Paste/SelectAll IDs are registered once in the shared field
catalog. Each eligible consumer handles the same ID locally; Code does not add a
second persisted Copy identity. Settings queries the assembled application
metadata, retaining categories, subsections, ordering, spacing and descriptions.
The frozen pre-refactor catalog fixture checks every label, default, category,
subsection, order and effective scope against the new assembly.

`app/actions.rs` now routes owners rather than enumerating individual editing,
scrolling, clipboard and file commands. Editor owns semantic editing/movement;
Code owns copy/wrap/navigation; Files owns file-command mapping and modal dispatch;
pane coordination owns reveal/collapse/focus policy. Field copy routes through the
field owner and retains its original no-scroll behavior.

The root deliberately offers workspace-eligible Editor commands before routing the
focused owner: bold/italic/underline and undo/redo retain their existing availability
from Code and Tools. This is the existing eligibility policy, not new bubbling.

## Preserved scope and lifecycle rules

| Situation | Effective behavior |
| --- | --- |
| Editor / Code / tool control | Exactly the previous scope; tools report radio/tab/grid/Hex/range context locally |
| Picker | Its focused control reports Explorer/Location/Filename/Format/FileButtons |
| Menu / dirty or overwrite prompt / review | Modal context masks workspace actions; no fallback dispatch into panes |
| Settings | Fixed controls intercept events before configurable resolution; closing retains underlying context |
| Closing a file modal | Underlying workspace focus was retained; successful replacement still deliberately focuses Editor |
| Scope change | Pending sequence is canceled before matching the new context |
| Mouse, paste, resize, focus loss, Settings opener | Existing explicit cancellation remains |
| Prefix mismatch or Escape | Consumed without leaking prefix keys into text |
| Timeout | Existing millisecond timeout; no delayed standalone action execution |
| Leader | Resolve before conflict checks; retain symbolic conversion, direct-binding unset, literal-preservation/removal choices |

Bare printable prefixes remain forbidden in editable scopes. Identical bindings
remain valid in disjoint scopes. Reserved openers remain `F1`, `Ctrl+/`, and `Ctrl+,`, including
normalized legacy aliases. Ctrl+? is not reintroduced. Escape with any modifiers
remains invalid as a leader; only bare Escape cancels an in-progress sequence.
No new shortcut, scope precedence, UI hint, or user-facing setting was introduced.

## Remaining bridges

| Bridge | Reason and removal stage |
| --- | --- |
| App-owned pane/layout state and numeric pane focus | Effective scope now has one coordinator; full Workspace ownership remains Stage 8 |
| Tools action methods still operate on App | Control/state composition and field clipboard completion remain Stage 6 |
| File and editor clipboard workflows still operate on App | Command mapping is local, but typed workflow decisions/narrow dependencies remain Stage 7 |
| Theme derives its path from application shortcut storage | Separate configuration path and theme contracts in Stage 5 |
| Settings save rollback remains application orchestration | Settings page/save contracts Stage 5, common storage mechanics Stage 7 |

Application ActionId/InputScope/Category/Subsection remain application vocabulary.
Their presence in application descriptors is intentional; the independent engine
accepts supplied types and does not import them. Field action scopes remain shared
until the corresponding controls are composed in Stage 6.

## Verification

| Check | Result |
| --- | --- |
| Formatting and whitespace | `cargo fmt --all --check`, `git diff --check`: pass |
| Core tests | 156 passed: 16 library, 122 binary, 17 integration, 1 doctest; 1 debug performance test ignored |
| Clippy | `cargo clippy --locked --all-targets -- -D warnings`: pass |
| Terminal PTY | Pass: remapping, capture, sequences/conflicts, inline hints, modal return, reload/failure, fields/files and terminal cleanup |
| Leader PTY | Pass: General navigation, capture, affected-binding warning, explicit Save and unset |
| Settings/theme PTY | Pass: preview, unsaved discard, Save and restart |
| Theme PTY | Pass: persisted overrides, document independence and invalid-config fallback |

New public tests cover supplied catalogs, identity/context rejection, disjoint
reuse, prefix conflicts, editable-text protection, caller-supplied reservations,
timeout/context/Escape cancellation, leader conversion and in-memory serialization.
Application regressions cover the complete catalog fixture, hidden/repeated panel
construction, retained custom hints, modal exclusions/return focus and modified
Escape. Existing strict persistence, legacy migration, Settings draft, adapter,
shared-field and clipboard tests remain in the full suite.

Logs: `/tmp/tuxd-shortcuts-stage4/`. No fresh graphical/device walkthrough or
performance measurement was required: this stage changes shortcut ownership,
not text layout or serialization. Final integration retains those gates.
