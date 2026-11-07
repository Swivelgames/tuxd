# Code generation initiative

Status: Deferred — discovery and design before implementation

## Outcome

Make a styled tuxd document usable in application code across selected languages and frameworks. Determine which output forms users need before committing to generators or a shared generator API.

ANSI syntax highlighting is independent and belongs to [MVP Slice 6](../mvp/slice-06-code-output.md). Existing raw/escaped ANSI exports remain the baseline for this exploration.

## Questions to explore

- Which languages and runtimes should be supported first? Evaluate actual usage before selecting Rust, Python, JavaScript/TypeScript, shell, Go, or other targets.
- Which output forms are useful: string literals, runnable printing examples, named constants, styled spans, or framework widgets?
- Which frameworks warrant dedicated output? Compare direct ANSI with native style/span APIs, including their color, attribute, reset, and Unicode limitations.
- How should generators preserve newlines, tabs, literal backslashes, quotes, braces, control characters and non-ASCII text? Define behavior for runtime formatting and interpolation.
- What does equivalence mean for each target: exact ANSI bytes or equivalent text and style semantics?
- How should users choose a target and its options, inspect the result, and copy/export it in wide and slim layouts?
- Which generated-code dependencies and supported versions are reasonable? How will outputs be compiled or otherwise validated reproducibly?

## Discovery deliverables

1. A language/framework/output-form matrix describing benefits, limitations, dependencies and validation approaches.
2. Small representative output examples for the most useful candidates, using one shared styled document.
3. An agreed output contract and initial target set.
4. An implementation plan with independently usable slices and appropriate generator boundaries.

## Guardrails

Keep the document model and ANSI serialization independent of target languages. Generated output is read-only. Escape document content as data; never treat it as executable source or a format string. Syntax colors and display-only Unicode fallbacks must not leak into generated payloads.

## Candidate starting point, not a commitment

The former Slice 6 proposed a Rust string literal usable with `print!("{}", literal)`, verified by compiling examples and comparing emitted bytes to raw ANSI export. Retain that as one candidate alongside other languages and framework-native representations, not as a predetermined first implementation.

## Out of scope until discovery is agreed

Generator implementation, language/framework commitments, arbitrary code editing, executing user-provided code, and dependency/version promises.
