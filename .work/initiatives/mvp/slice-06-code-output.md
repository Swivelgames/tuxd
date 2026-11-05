# Slice 6 — better code output

Status: Planned; follow-on after MVP

Depends on: [Slice 5](slice-05-compatibility-and-hardening.md)

Estimate: 2–4 developer days for ANSI inspection improvements and Rust generation. Additional languages require separate estimates.

## Usable checkpoint

Inspect highlighted ANSI tokens and copy a valid Rust snippet that reproduces the document. The existing escaped and raw ANSI workflows remain available and unchanged in meaning.

## Deliverables

- Lightweight ANSI token highlighting distinguishing escape introducers, numeric parameters, text, and resets.
- Human-readable sequence descriptions, such as bold, foreground RGB, or reset, accessible without obscuring the output.
- Escaped ANSI and Rust output tabs, with clear copy/export actions for the selected representation.
- Rust generation with a defined output contract: start with a valid escaped string literal usable with `print!("{}", literal)` rather than using document content as a format string.
- Correct escaping of quotes, backslashes, newlines, tabs, control characters, and Unicode while preserving the underlying ANSI bytes.
- A scrollable code view with optional expanded mode and preserved editor state on return.
- A small generator interface that permits later language additions without changing the document model or ANSI serializer.

The output remains generated and read-only. Highlighting and explanations are view metadata, never copied or exported as part of the code.

## Stability and verification

Test token boundaries and generator cases containing quotes, backslashes, braces, multiline text, Unicode, and escape-like literal text. Compile representative generated Rust examples and compare captured output bytes with raw ANSI export. Avoid automatically executing arbitrary pasted code; generated examples contain document content only as escaped string data.

Verify copy/export uses the selected representation and contains no highlighting markup. Re-run the MVP workflow to ensure output-view changes do not alter editing, styling, or persistence.

## Hands-on acceptance

1. Open a saved styled document and inspect highlighted ANSI and sequence explanations.
2. Switch to Rust, copy its string literal into a minimal example using the documented print pattern, and compile it.
3. Run the example and compare its output with the editor and raw ANSI export.
4. Repeat with quotes, backslashes, braces, newlines, and Unicode in the document.
5. Expand/scroll the code view, return to the editor, and continue editing at the preserved position.
6. Switch back to escaped ANSI and confirm copying/exporting still produces that representation.

## Deferred

Additional languages, framework-specific widget generation, arbitrary code editing, and ANSI import. Choose subsequent generators based on actual usage.

## Completion evidence

Pending implementation: launch command, checks/results, compiled-example byte comparisons, terminal/platform, and limitations.
