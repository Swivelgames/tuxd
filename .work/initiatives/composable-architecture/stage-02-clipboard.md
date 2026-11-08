# Stage 2 — independent clipboard and Code copy

Status: complete on 2026-11-07, committed as `94ccc79` and pushed to
`origin/refactor/code-organization` before Stage 3 began.
Stage 1 was committed as `c794ba6` and pushed before this implementation began.
Stage 3 (shared text surface and session/editing separation) is next.

## Scope and ownership

| Module | Owns | Excludes |
| --- | --- | --- |
| `libraries::clipboard` | Service, owned requests, IDs, one-shot completions, shared write lane | ANSI, App, selections, focus, notices |
| `libraries::clipboard::system` | Environment routing, desktop/TMUX helpers, OSC 52 encoding, capability hints | Caller policy and terminal emission |
| `libraries::clipboard::process` | Helper execution, bounded reads, deadlines, kill/reap | Document mutation and UI |
| `features::code::Code` | Copy payload creation, pending copy, receipt interpretation | Editor cut/paste and file workflows |
| `features::code::projection` | Existing escaped Code display rows | Clipboard payload construction |
| `features::clipboard_notice` | Shared wording for transport facts | Transport execution or target policy |
| `editor_clipboard` | Selection payload, cut/paste targets, eligibility, mutation and review policy | Worker threads and transport routing |
| Picker / App field bridges | Original field, sanitization, local messages | Worker threads and transport routing |

No dependencies, formats, action IDs, key defaults, or product features changed.
The former root `clipboard.rs` and `code_projection.rs` are removed; there is one
transport implementation and one Code projection implementation.

## Complete Code copy path

1. The existing catalog resolves Copy in Code's context and the dispatch bridge
	calls `Code::copy` with the document and clipboard service.
2. Code directly calls `Document::to_escaped_ansi`; viewport, syntax colors,
	wrapping, and gutters cannot affect this payload.
3. The service reserves the shared write lane and starts its driver on a worker.
	Code retains the returned request and presents its starting notice.
4. Between frames, the root polls Code. The request returns a typed receipt or
	error; OSC 52 bytes are written/flushed on this terminal-owning thread.
5. Code clears its request and formats its own completion notice. Editor's cut
	completion is never involved, even after focus or document changes.

This removes the generic `copy_ansi` flag, mixed Code/Editor completion routing,
and caller-owned channels/worker spawning. Root polling is now composition of
owner operations. It still exists as a temporary application assembly point.

## Request lifecycle and transport contract

`Service::with_driver` accepts a synchronous driver for isolated tests. The service
runs it on a worker; each `WriteRequest` or `ReadRequest` owns its completion channel
and ID. Polling a completed request returns nothing, including after output failure.
Read/write capability values are environment hints, not promises of helper success.

Service clones share one write lane. Both worker and request retain its permit:
dropping an owner cannot permit overlapping writes while the old worker still runs.
Dropping a request abandons its response and any pending terminal emission. It does
not undo a clipboard write already performed externally. Read requests are independent.

The system driver preserves existing policy:

- `linux`/`dumb` reject writes; TMUX takes precedence over SSH; direct SSH uses OSC 52.
- TMUX uses the named `tuxd-<pid>` buffer and `load-buffer -w`. Failure never falls
	back to copying into a different host's clipboard.
- Desktop helpers remain wl-copy/wl-paste, xclip, then xsel as applicable to the
	environment. Reads still use desktop helpers only; no remote-read feature is added.
- Each helper keeps its three-second deadline, kill/reap behavior, and bounded
	64 KiB UTF-8 read. OSC 52 keeps its 100 KiB payload limit and exact base64 bytes.
- Workers prepare OSC 52 output but never write stdout. Request finalization emits
	it between frames. TMUX and OSC 52 receipts remain explicitly unconfirmed.

## Editor and field behavior preserved

Editor owns a pending copy with its optional cut target. Only accepted transport
success with a still-valid target deletes the selection. Failure or stale completion
cannot cut. Pending reads retain document/revision, cursor, selection, pending style,
and modal eligibility checks. Loading even an identical document invalidates pending
cut and paste targets. ANSI parsing/review and atomic undo behavior are unchanged.

Fields retain literal payloads and their original target. RGB completion updates the
field captured at request time; changing focus cannot redirect it. Picker completion
belongs to that picker instance and uses its existing path sanitization. A replacement
picker cannot consume the old instance's read result. Bracketed paste remains separate.

## Remaining bridges

| Bridge | Reason / remaining owner | Removal stage |
| --- | --- | --- |
| Code viewport and rendering assembly in App/UI | This stage extracts copy and projection, not text surface state | 3 |
| Code action routed through existing registry | Preserve stable Copy ID and eligibility until local registration exists | 4 |
| Editor clipboard methods still implemented on App | Session/editor ownership not yet separated; guards remain intact | 3, 7 |
| Editor review uses Files modal state | Shared review workflow has not been extracted | 7 |
| RGB/field completion polling in App | Fields need shared component ownership before local coordination | 6–7 |
| Picker constructs a default service for reads | Dependency assembly moves with workflow ownership; all writes already share App's service | 7 |
| App polls owners and locks stdout between frames | Terminal/root composition remains to be finalized | 8 |

## Verification

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | Pass |
| `cargo test --locked --quiet` | 141 passed: 11 library, 118 binary, 11 integration, 1 doctest; 1 performance test ignored |
| `cargo clippy --locked --all-targets --quiet -- -D warnings` | Pass |
| `uv run scripts/clipboard_smoke.py` | Pass: exact >10 KiB direct SSH payload, isolated TMUX forwarding, no wrong-clipboard fallback |
| `uv run scripts/editor_clipboard_smoke.py` | Pass: styled editing, atomic undo, review, persistence, cleanup |
| `uv run scripts/terminal_smoke.py` | Pass: editing, fields, picker, shortcuts, terminal cleanup |
| `uv run scripts/stdin_import_smoke.py` | Pass: raw/escaped inputs, review, save, keyboard handoff, limits, cleanup |
| Final source review and whitespace | Changed files reviewed; no application/UI/ANSI imports in clipboard library; `git diff --check` passed |

Controlled drivers verify exact Code payloads, all receipt kinds, failure without
mutation, one-shot/disconnected completion, terminal-write failure without replay,
dropped owners, shared write exclusion, focus changes, identical-document reload,
field targeting, and picker lifetime. Existing cut/paste regression tests now exercise
owned requests instead of injecting central channels.

Transient logs are in `/tmp/tuxd-clipboard-stage2/`. No fresh graphical/device walkthrough
or performance measurement was performed; layout and rendering were not changed.
Local OSC 52 payload validation does not claim Android clipboard receipt.
