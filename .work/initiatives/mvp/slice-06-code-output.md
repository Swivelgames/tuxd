# Slice 6 — ANSI syntax highlighting

Status: Implemented — verification passed; user review pending

Depends on: [Slice 5](slice-05-compatibility-and-hardening.md)

## Usable checkpoint

The existing Code pane makes generated ANSI easier to read through syntax colors. Editing, wrapping, scrolling, copy and export retain their existing behavior.

## Deliverables

- Distinguish SGR introducers/terminators, numeric parameters, separators and resets.
- Highlight visible newline/tab escapes while leaving ordinary text in the default color.
- Treat doubled backslashes as literal document content so escape-like text is not mistaken for generated ANSI.
- Preserve token colors through wrapping, horizontal scrolling, slim layouts and the display-only Unicode fallback.
- Apply highlighting only at render time; raw ANSI and escaped clipboard/export bytes remain unchanged.

## Verification

Test token boundaries and byte offsets around Unicode, literal backslashes and escape-like text. Distinguish reset parameters from zero-valued RGB components. Inspect actual terminal rendering with styled content, wrapped sequences and composed-Unicode fallbacks. Verify copy/export remains independent of highlight styling and rerun relevant rendering/clipboard regressions.

## Deferred

Language/framework code generation now belongs to the separate [code generation initiative](../code-generation/overview.md), which begins with discovery rather than selecting Rust in advance. Output-language tabs, generator APIs, framework adapters and generated-code validation are not part of this slice. Human-readable sequence explanations are also deferred; this slice adds syntax highlighting only.

## Completion evidence

Implemented on `feat/6-ansi-syntax-highlighting`.

- `cargo test --locked --quiet`: 104 passed; one performance measurement intentionally ignored.
- Formatting, Clippy with warnings denied, and diff whitespace checks pass.
- Token regressions cover literal escape-like content, byte offsets after Unicode, RGB zero versus reset, and incomplete/non-SGR sequences.
- Render regression verifies syntax colors and horizontal-scroll offsets without document, raw ANSI or clipboard changes.
- `uv run scripts/clipboard_smoke.py`: exact escaped payloads pass through direct SSH and tmux forwarding; failure feedback also passes.
- `python3 scripts/alacritty_smoke.py` and its `--unicode` variant pass real input, saving, settings, resize/slim/collapse and exit. The Unicode variant also verifies pixel-identical restoration after selection and newline changes.
- Inspected Alacritty screenshots for normal, narrow/wrapped and composed-Unicode fallback output. Syntax colors remain confined to the Code pane.

Launch for review: `cargo run --release -- examples/welcome.tuxd.json`. No code-generation implementation or new output mode is included.
