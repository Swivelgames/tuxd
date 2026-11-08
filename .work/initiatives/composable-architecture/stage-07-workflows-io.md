# Stage 7 — workflows and I/O

Status: complete on 2026-11-07; committed and pushed as `f1dc426` on
`refactor/code-organization` before Stage 8.
Stage 6 was committed and pushed as `df140a3` before implementation.
Stage 8 (final composition and cleanup) is next.

## Ownership and removed obligations

| Owner | Responsibility | Removed caller obligation |
| --- | --- | --- |
| Picker `Selection` | Selected `PathBuf` and operation/ANSI format | Files no longer mirrors the picker's mutable action or converts submitted paths through lossy strings |
| `components::import_review` | Inert preview, diagnostics, scrolling, focus and typed `Strip`/`Abort` decisions | The view no longer owns file/stdin/clipboard identity or decides how accepted content is committed |
| `workflows::PendingReview` | Parsed candidate, explicit source, destination and review view | No parallel clipboard-review flag or interception of a Files review dialog |
| `workflows::parse_bytes` | Shared UTF-8 decoding and ANSI parsing for file/startup byte sources | Startup and interactive import no longer duplicate decoding/error policy |
| Editor clipboard coordinator | Paste-mode detection, original-target capture and revalidation | No import, construction, or inspection of Files modal state |
| `workflows::publication` | Encode, protect the active native file from export, publish, then update saved/origin state | UI callers cannot forget to update session association after successful publication |
| `libraries::storage` | Bounded stream/file reads and atomic single-file byte publication | Document, configuration and stdin consumers no longer duplicate these mechanics |
| Configuration adapter | Tab-indented JSON, final newline, parent creation, publication | Theme and shortcut codecs retain validation but share publication mechanics |

The same independent ANSI parser still handles clipboard text. Clipboard's raw,
escaped, plain and literal detection remains local policy; it does not acquire
filesystem or startup semantics. Review presentation receives supplied title,
explanation and accept text, with no App, Files or workflow dependency.

## Workflow ordering and state

File selection returns both path and operation. Open validates native data before
asking to abandon a dirty document. ANSI import parses before review; accepted
review proceeds through the existing dirty-document decision before replacement.
Save failures keep the document and pending replacement/quit continuation, allowing
retry. Cancel ends the pending operation. Overwrite remains explicitly confirmed.

`PendingReview` stores the candidate and one destination: document replacement or
paste with a captured target. File and stdin sources commit unsaved documents;
file origin remains available for subsequent pickers. Clipboard review inserts
into the original session/selection only if its identity, revision, content,
cursor, anchor and pending style still match. Abort never edits the document.
The view's decision is consumed once. There is no Files-owned review state.

Review masks normal workspace input, including unbound characters and bracketed
paste. Settings can still open above it and restore its context. Starting a modal
cancels keyboard sequences and pointer interactions. The picker owns its clipboard
completion; Files polls that owner without the runtime matching picker variants.

## Storage policies retained

| Consumer | Read limit | Publication policy |
| --- | --- | --- |
| Native document / ANSI file | 16 MiB | Reject symlink/special destinations, preserve existing regular-file permissions, explicit replacement or no-clobber |
| Stdin ANSI | 16 MiB | No publication; preserve stdin-specific errors and controlling-TTY handoff |
| Theme configuration | 64 KiB | Validate, create parents, atomically replace the entry with private temporary-file permissions; missing load uses defaults |
| Shortcut configuration | 1 MiB | Validate, create parents, same configuration publication; retain legacy decoding and missing-load behavior |

File reads check the opened descriptor, use nonblocking open to reject FIFOs, and
bound actual reads as well as metadata length. Theme loading now applies its
64 KiB bound directly rather than reading up to the document limit first.
Missing, invalid, oversized and I/O failures remain distinguishable. Native schema
encoding/decoding is referenced through its actual adapter; the persistence codec
reexport is removed.

Configuration save still validates both drafts before effects and writes theme
before shortcuts. If shortcut publication fails, existing theme bytes are restored
using atomic publication, or a newly created theme file is removed. Restoration
failures remain attached to the original error. The original byte snapshot retains
its existing semantics, including preserving malformed prior data for rollback.
This is best-effort restoration, **not** a crash-atomic two-file transaction; no
parent-directory fsync guarantee is added.

CLI OS-string parsing, explicit `--raw`/`--escaped`, `-` handling, stdin consumption,
controlling-terminal duplication, and terminal setup/cleanup are unchanged.

## Verification

| Check | Result |
| --- | --- |
| Rust tests | 177 passed: 19 library, 133 binary, 24 integration, 1 doctest; 1 debug performance test ignored |
| Formatting / whitespace | `cargo fmt --all --check`, `git diff --check`: pass |
| Clippy | `cargo clippy --locked --all-targets -- -D warnings`: pass |
| Terminal PTY | File operations, picker, remapping, modal return, resize, terminal error/panic/normal cleanup: pass |
| Stdin PTY | Raw/escaped file and piped input, review accept/abort, keyboard handoff, save, limits and cleanup: pass |
| Editor clipboard PTY | Styled copy/cut/paste, atomic undo, review accept/abort, persistence and cleanup: pass |
| Settings/theme PTY | Preview, discard, Save and restart: pass |

New tests cover independent clipboard review and stale-target rejection, file/stdin
source preservation, non-UTF-8 native path submission, exact read bounds, missing
versus invalid sources, FIFO rejection without a writer, no-clobber publication,
and retained document/private configuration permission policies. Existing tests
cover pending dirty/overwrite failure/cancel flows and configuration restoration
failure. Final changed source/test files were read in full for ownership and
readability review.

Logs: `/tmp/tuxd-workflows-stage7/`. No new graphical/device walkthrough or
performance measurement is claimed for this stage.

## Remaining composition work

Stage 8 owns the App/Workspace assembly, modal routing and visibility/import audit.
The existing Files coordinator still connects prompts to session replacement via
App methods; session installation still resets workspace focus/viewports. These
are application composition responsibilities, not dependencies of the byte-storage
library, ANSI parser, picker, review view or publication operation. Consolidate
that shell without changing the now explicit source/destination/decision contracts.
