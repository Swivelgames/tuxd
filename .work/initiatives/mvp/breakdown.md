# MVP slice breakdown

Status: In progress — Slice 1 complete

Each slice builds on the preceding checkpoint and includes its own stability work. Estimates are planning ranges for one developer and modest document sizes, not deadlines. Revise them after the editor and styling foundations are validated.

| Slice | Usable checkpoint | Estimate | Dependency |
| --- | --- | --- | --- |
| [1. Editor foundation](slice-01-editor-foundation.md) | Launch, type, navigate, paste, scroll, undo, and redo in a real text box | 2–3 days | None |
| [2. Styling loop](slice-02-styling-loop.md) | Select text, apply visible styles/colors, and inspect live ANSI | 3–5 days | Slice 1 |
| [3. Tool panel](slice-03-tool-panel.md) | Use all initial color modes, style controls, and character categories | 2–3 days | Slice 2 |
| [4. Save and export workflow](slice-04-save-and-export.md) | Save/reopen projects, copy output, and export ANSI | 2–3 days | Slice 3 |
| [5. Compatibility and hardening](slice-05-compatibility-and-hardening.md) | Use the integrated workflow reliably across the declared support matrix | 3–5 days | Slice 4 |
| [6. Code output](slice-06-code-output.md) | Inspect highlighted sequences and copy valid Rust snippets | 2–4 days | Slice 5; after MVP |

The first complete styling loop is approximately 5–8 developer days. The MVP is approximately 12–19 days, with another 2–4 days for the initial code-output improvements. Additional language generators are separately estimated once their output contracts are chosen.

## How to implement each slice

Start with its smallest end-to-end workflow, make that runnable, and then complete the scope and acceptance criteria. Keep the app usable during development. Do not merge a nominally complete slice whose central workflow is still stubbed or unstable.

The slice specifications distinguish deliverables, stability requirements, hands-on checks, and deferred work. Checks are planned until implementation evidence is recorded in the corresponding slice. Slice 1 now records its implementation and passing verification; subsequent slices remain planned.

## Sequence rationale

Slice 1 validates editing and terminal lifecycle. Slice 2 proves the hardest product assumption: text and formatting remain coherent through selection and edits, and output matches the document. Slice 3 expands the available operations on that foundation. Slice 4 makes work persistent and reusable outside the app. Slice 5 broadens compatibility and stress coverage. Slice 6 improves how developers consume the generated code.

## Checkpoint evidence

For each completed slice, record the launch command, relevant automated checks and results, the terminal/platform used for the walkthrough, and any remaining limitations in that slice file. Keep status as Planned until work begins, then In progress, then Complete only when its acceptance contract passes.
