# Character redraw investigation

## Reproduction

Reported: General symbols corrupt the display when scrolling/redrawing through
tmux over SSH. Locally reproduced the cell-width disagreement with Alacritty
0.17.0 and tmux 3.8; the remote SSH terminal itself was not measured.

Cursor-position reports after writing each scalar showed no width disagreements
for bare Alacritty, but nine for tmux among the original non-Nerd catalog entries.
These reports measure terminal cell advance, not font coverage or glyph ink.

| Collection | Scalars | tuxd / bare Alacritty | tmux |
| --- | --- | --- | --- |
| General symbols | U+261D, U+26F9 | 1 cell | 2 cells |
| Dingbats | U+270C, U+270D | 1 cell | 2 cells |
| Pictographs | U+1F3CB, U+1F3CC, U+1F574, U+1F575, U+1F590 | 1 cell | 2 cells |

A renderer calculating one cell while tmux advances two can leave stale cells or
misplace subsequent output during incremental redraws. Full repaint alone does
not repair the underlying disagreement.

## Pairing experiments

Each of the nine scalars was measured bare, with VS15 (text presentation), and
with VS16 (emoji presentation). All three forms advanced one cell in the tested
Alacritty and two inside tmux. Pairing therefore did not provide a portable fix.
[Unicode UTS #51](https://www.unicode.org/reports/tr51/) defines presentation
selectors; it does not establish one terminal-cell implementation across stacks.

## Applied mitigation

- The shared text projection escapes these nine scalars, including selector variants.
- Catalog admission already rejects characters requiring display fallback, so the
	picker no longer offers them. Raw metadata remains intact for reproducible generation.
- Editor, Code, and editable fields use the same projection/width policy.
- Documents, undo, saving, export, and copying preserve the original characters.
- Other symbols and standalone emoji retain native rendering.

`python3 scripts/character_width_smoke.py` checks the entire candidate catalog
(including Nerd icons), selector variants, and representative composed sequences
in isolated Alacritty and tmux. After mitigation: **18,718 samples, zero width
mismatches in either stack**. `--raw` bypasses projection to reproduce disagreements;
its failure exit status is intentional when a mismatch is detected.

`python3 scripts/alacritty_smoke.py --symbols --tmux` exercises selection,
newline insertion/deletion sufficient to scroll, restoration, saving, and resize.
The restored document body is compared pixel-for-pixel with the initial body.

## Component emoji follow-up

| Approach | Finding / requirement |
| --- | --- |
| Append a presentation selector | Insufficient for these measured conflicts; do not silently change document bytes |
| Force all emoji to two cells | Breaks terminals which consume one cell for text-default symbols |
| Render components separately | Requires a display projection that prevents joining, accounts for each visible component, and maps the whole source grapheme to editing/hit testing |
| Native composed rendering | Requires terminal/multiplexer capability or width measurement, plus redraw tests for ZWJ, modifiers, flags, selectors, and keycaps |
| Conservative escaped projection | Implemented for known conflicts; deterministic across tested stacks and lossless for document data |

Do not enable composed emoji wholesale as part of a picker fix. Keep the shared
projection seam, add a measured/capability-aware rendering policy, and validate
Alacritty, tmux, Zed, Termux/SSH, and Linux VC before replacing fallback behavior.
