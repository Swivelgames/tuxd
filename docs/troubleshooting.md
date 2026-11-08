# Troubleshooting

[Documentation](README.md)

## Find your symptom

| Problem | Try this |
| --- | --- |
| Copy/paste fails on Wayland | Install `wl-copy` and `wl-paste` |
| Copy/paste fails on X11 | Install `xclip` or `xsel` |
| Paste fails over SSH | Use your terminal’s paste command; remote clipboard querying is not provided |
| Copy fails over SSH or tmux | Check terminal OSC 52 and tmux clipboard-forwarding settings |
| Large copy fails | Export to a file; direct OSC 52 is capped at 100 KiB |
| Ctrl+I inserts a tab | Use Italic in Style, or bind another key in Settings |
| Ctrl+Shift+S is indistinguishable from Ctrl+S | Use F4 → Save as |
| Mouse interaction fails in tmux | Check `set -g mouse on`; use Shift+navigation for keyboard selection |
| Nerd glyphs are missing | Use a suitable Nerd Font in your terminal |
| Theme or bindings do not load | Back up the affected config and check its values against the [configuration reference](configuration.md) |

Forwarded clipboard delivery is **unconfirmed**: sending successfully does not prove the terminal accepted it. The Linux virtual console and dumb terminals do not support the clipboard transports above; export remains available.

## Import or pipe fails

Check these in order:

1. **Format:** use Raw for actual ESC bytes; Escaped for notation such as `\x1b[31m`.
2. **Encoding and size:** UTF-8, at most 16 MiB.
3. **Pipe syntax:** use `--raw -` or `--escaped -`, not bare `-`.
4. **Terminal:** stdout must remain a terminal, with a controlling terminal for input.
5. **Completion:** piped input must reach EOF before the UI starts.
6. **Controls:** unterminated control strings may be rejected rather than stripped.

See [import examples](files.md#import-existing-ansi) and [supported ANSI](formats.md#supported-sgr-controls).

## Text or colors look different

| Symptom | Explanation |
| --- | --- |
| Joined emoji, flags, or known width-conflicting symbols display as escapes | Safe display fallback; original text is preserved in save/export/copy |
| Character width differs | Terminal/font handling of ambiguous-width and emoji characters varies |
| Basic/indexed colors differ | These follow the terminal palette |
| RGB looks wrong | True-color support is required |
| Italic, dim, or strike-through looks wrong | Support depends on the terminal and font |
| Theme changes do not recolor the document | Interface themes and document formatting are independent |
| `NO_COLOR` does not disable the preview | Color is part of the document being edited |

Accents, CJK, and most standalone emoji render normally, subject to the terminal font.
Nine text-default emoji symbols use display escapes because tmux and Alacritty
disagree about their cell widths; this prevents stale cells during redraws. tuxd does not install or choose terminal fonts.

## A key is recorded as another key

Some terminal protocols cannot distinguish Ctrl+I/Tab, Ctrl+H/Backspace, or modified shortcuts. tuxd requests enhanced reporting where supported and records the input actually received.

- Open Settings with **F1**, which remains fixed.
- Record an alternative binding.
- Use the File menu or tools when a terminal cannot report your preferred shortcut.

## Supported environment and limits

**Target:** Linux with a UTF-8 terminal. Hands-on walkthroughs have passed in Alacritty, Zed’s terminal, tmux, Termux over SSH, and the Linux virtual console. Other operating systems are not currently supported.

| Area | Current limit |
| --- | --- |
| Documents | One open document; modest-document performance target |
| Undo | Up to 200 transactions, stored as snapshots |
| File I/O | Synchronous, bounded reads |
| Recovery | No autosave or crash recovery |
| Concurrent editing | No external-change detection; avoid modifying an open file elsewhere |
| Mouse selection | Clamps at viewport edges; no drag autoscroll |

See the [backlog](../.work/deferred-work.md) for future work.

## Report a problem

Open an issue in the [project tracker](https://github.com/swivelgames/tuxd/issues) with:

- `tuxd --version` output.
- Linux distribution and terminal/version.
- Whether tmux or SSH is involved.
- Reproduction steps and relevant custom bindings.
- A small, sanitized example document or screenshot, if useful.
