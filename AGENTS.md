# tuxd working agreements

Read [Code organization and readability](docs/code-organization-and-readability.md)
before meaningful implementation, refactoring, or review. This guidance was
ported from the agnostic product and applies in Rust idioms; its examples are
illustrative, not mandatory file layouts or reasons to build frameworks.

- Use tabs where permitted; use four spaces only when tabs are invalid.
- Group cohesive capabilities and let each own its assembly and private internals.
- Simplify before relocating: identify what is unique, what can be isolated, and
	what should be orchestrated. Apply this from statements through libraries.
- Make operations complete: callers should not repair invariants or coordinate
	private implementation steps. Keep library contracts ignorant of App/features.
- Separate domain policy, application coordination, storage, input, and rendering.
- Keep the composition root small. Add a capability where it belongs, rather than
	adding unrelated branches to central event handlers or views.
- Give each file one primary responsibility. Split for ownership, not line count.
- Prefer named logical states and operation modes over positional conventions.
- Keep module assembly together; separate methods and meaningful operation stages.
- Use useful names, guards, and spacing to expose meaningful stages and preserve
	the order of effects. Avoid catch-all utilities and speculative abstractions.
- Align equivalent responsibilities across their consumers without erasing
	intentional differences. Keep interfaces and visibility as narrow as practical.
- Keep document content/ANSI behavior independent of UI themes and settings.
- Review complete changed source/test files after the final fix, plus edited
	documentation in context. Formatting and passing tests do not prove readability.
- Preserve behavior during organizational refactors. Run formatting, unit tests,
	clippy, and the relevant PTY acceptance checks before reporting completion.

See [Architecture](docs/architecture.md) for the ownership map and extension points.
For refactor history and preserved contracts, read the [initiative](.work/initiatives/composable-architecture/overview.md)
and its contracts, inventory, stage plan, and validation ledger before changing these boundaries.
All eight stages are implemented; keep future changes within their ownership contracts.
