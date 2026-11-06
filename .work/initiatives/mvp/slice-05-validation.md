# Slice 5 validation record

Status: Complete — automated verification and user acceptance passed across the target terminal matrix.

## Environment and support matrix

Measured on Linux 7.2.14-zen1-1-zen, x86_64, AMD Ryzen 7 5800X3D, Rust 1.95.0. The current desktop session is X11. The user’s target environments are Alacritty, the default TTY, Termux on Android over SSH, and Zed’s integrated terminal. The user confirmed that “default TTY” means the Linux virtual console. The rows below distinguish target environments from verified evidence.

| Environment | Evidence | Remaining limitations |
| --- | --- | --- |
| Linux PTY, xterm-256color, pyte 0.8.2 | Automated typing, selection, attributes, indexed/RGB colors, dialogs, resizing, custom keymaps, clipboard payloads, persistence, exports, cleanup | A terminal protocol simulation; does not certify desktop font rendering or emoji cell widths |
| Ratatui TestBackend 0.30.2 | Combined pane/resize coverage down to 1×1; rendering/performance fixture | In-memory rendering, excluding terminal I/O and compositor latency |
| Alacritty 0.17.0 (94e7c887), primary | Real rendering/input in isolated Xvfb: selection styling, save, settings, wide/slim/tiny resizing, collapse and clean exit; screenshots inspected | Software GL and default Noto Sans Mono font; does not certify every user font or emoji width |
| Linux virtual console | User reports successful manual testing | Console/font configuration was not reported; no broader font guarantee inferred |
| Termux on Android → SSH → Linux host | User reports full walkthrough successful; clipboard forwarding also confirmed | Version, keyboard and remote TERM not reported; client-specific clipboard size limits remain unknown |
| Zed integrated terminal | User reports full walkthrough successful | Version and font not reported; no guarantee for other host keybinding configurations |

foot is installed but is not part of the user’s requested matrix.

### Walkthrough priorities

Use the same fixture and save/export comparison in each environment. Record actual versions, TERM, font and input method alongside observations.

- **Alacritty:** establish the primary baseline for styles, ANSI16/256/RGB, Unicode widths, selection, mouse/scrollbars, custom shortcuts, clipboard and terminal restoration.
- **TTY:** verify keyboard-only access to all controls, usable colors and borders, attribute/glyph limitations, and cursor alignment. Confirm unsupported visual styling does not alter saved text or exported styles. Record clipboard behavior and error feedback.
- **Termux over SSH:** test the available Android keyboard/extra-key inputs, modifiers and function keys, multi-key sequences and timeout under connection latency, paste, narrow layouts and terminal resizing. Distinguish terminal-side paste from the application’s remote clipboard helper; verify where copied content actually lands before claiming Android clipboard support.
- **Zed:** test which shortcuts reach tuxd versus being handled by the host editor, setting alternate bindings, terminal paste, mouse selection, resizing and font widths.

Cross-environment display differences may be documented limitations when editing remains usable and document data is preserved. Any required fallback or new clipboard transport should be scoped from observed failures.

## Added acceptance coverage

- Three deterministic seeds, 1,200 mixed operations each: combining marks, CJK, emoji modifiers and ZWJ sequences, flags, Indic text, tabs/newlines, selection, styles, deletion, history and resizing.
- After each operation: cursor/anchor grapheme boundaries, validated project encode/decode equality, ANSI export/import equality, deterministic output. Periodic explicit edit/undo/redo checks preserve exact documents.
- A 1,000-line styled Unicode fixture survives pane changes and repeated wide/slim/tiny rendering without document mutation.
- CLI regression: non-UTF-8 Linux filenames open normally. Argument parsing now preserves OS strings instead of panicking during UTF-8 conversion.
- Startup with FIFO, directory or missing file returns an error before terminal setup; FIFO reads cannot hang the process.
- Existing PTY acceptance continues to cover failed helpers, malformed configuration, failed document loads, unsupported import controls, dirty/overwrite cancellation, normal/error/panic terminal restoration, and saved shortcut restart behavior. Clipboard helpers and XDG configuration are isolated from the user's settings.

## Performance target and fixture

Target: release-mode edit + frame construction + undo p95 below 50 ms at 120×40 for this modest document. This is an internal responsiveness budget, excluding terminal transport/display latency. Project validation plus project and ANSI round trips should finish below 100 ms. Timing is reported explicitly rather than enforced as a flaky unit-test deadline.

The deterministic fixture has 1,000 lines, 113,999 UTF-8 text bytes, 1,000 alternating style runs, tabs, accents, CJK, emoji and box drawing. Generate an editable project and run 40 measurement samples with:

```sh
TUXD_PERF_FIXTURE=/tmp/tuxd-performance.tuxd.json cargo test --release modest_document_performance -- --ignored --nocapture
cargo run --release -- /tmp/tuxd-performance.tuxd.json
```

Observed after the Unicode display fallback on the environment above: median 20.81 ms, p95 21.82 ms, maximum 42.60 ms. Validation + project roundtrip + ANSI roundtrip: 11.29 ms. The measured target passes; no performance optimization was warranted. The fixture lives as a deterministic generator in `src/hardening.rs`, avoiding a large duplicated checked-in JSON file.

## Reproduction and remaining acceptance

```sh
cargo fmt --all --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
uv run scripts/terminal_smoke.py
uv run scripts/clipboard_smoke.py
python3 scripts/alacritty_smoke.py
cargo test --release modest_document_performance -- --ignored --nocapture
```

Automated suite: 100 ordinary tests plus one explicitly ignored performance measurement. Run the measurement separately as above.

The terminal-coverage acceptance requirement is satisfied: automated graphical verification covers Alacritty directly and through tmux, and the user reports successful Linux virtual console testing and full Termux/Zed walkthroughs. Environment details not supplied by the user remain explicitly unknown. Installed versions and PTY TERM values alone are not counted as acceptance evidence.

Known limits: fonts and terminals may disagree about emoji/ambiguous character widths; Nerd glyphs require a suitable font. Desktop clipboard integration requires working wl-copy/wl-paste or xclip/xsel. Legacy terminal encodings can merge physical keys such as Ctrl+I/Tab. History is bounded to 200 snapshots; large-document engineering remains deferred. Performance measurements exclude display transport and do not establish a large-file guarantee.

## Resolved SSH clipboard gap

Original user report: Code-panel Ctrl+C did not reach Android from Termux over SSH, but tmux copy-mode selection does. This verifies that tmux copy reaches the client clipboard in that user workflow; it does not establish application clipboard support.

Before this fix, tuxd used only host-side wl-copy/xclip/xsel. The inspected local tmux server has `set-clipboard external`, an xterm clipboard terminal feature, and no copy-command. OSC 52 is the likely forwarding mechanism. This local configuration is evidence for this session, not a claim about every remote tmux server.

The [official tmux clipboard documentation](https://github.com/tmux/tmux/wiki/Clipboard) distinguishes tmux-originated clipboard writes from application-originated OSC 52: external permits the former while rejecting the latter. Therefore merely emitting OSC 52 inside tmux is insufficient for this configuration.

Implemented transport selection: tmux first when TMUX is present; direct OSC 52 when SSH_CONNECTION or SSH_TTY is present; otherwise existing desktop helpers. The tmux helper loads a process-specific named buffer via stdin with `load-buffer -w`, avoiding shell quoting and argument-size limits. OSC 52 uses base64 and is written on the UI thread between frames. Known Linux-console/dumb terminals return an explicit unsupported-transport error. Failed remote forwarding does not fall through to the host desktop clipboard.

Direct OSC 52 has a 100 KiB input limit and never truncates or fragments clipboard contents. Client limits may be smaller. Forwarded status reports delivery unconfirmed; this is not proof of Android clipboard receipt. Clipboard reads/paste remain separate.

Verification: `uv run scripts/clipboard_smoke.py` passes for exact >10 KiB escaped ANSI payloads through a direct-SSH PTY and an isolated tmux server with `set-clipboard external`, plus failed tmux-helper feedback. Tests decode the emitted client-facing OSC and compare it byte-for-byte with the expected payload. Unit tests cover routing, control-safe encoding, Unicode, empty input and the size boundary. The broader terminal smoke suite isolates inherited TMUX/SSH variables to avoid touching the user's real clipboard.

The application transport gap is resolved. After testing the implementation, the user confirmed all three copy paths work: tmux forwarding, direct SSH OSC 52, and local desktop helpers. This supplies end-to-end clipboard acceptance in addition to the automated forwarding checks. Payload sizes and maximum client-specific limits were not reported, so no additional size guarantee is inferred.

## Alacritty rendering and input

`python3 scripts/alacritty_smoke.py` runs the actual Alacritty binary on an isolated Xvfb display, with temporary configuration and a disposable copy of the welcome document. Requires Alacritty, Xvfb, xdotool and ImageMagick. It does not interact with the user's desktop or clipboard. The test saves screenshots in a reported temporary artifact directory.

Verified typing, Shift+arrow selection, bold, exact saved project contents, Shortcut Settings, resizing across the slim-layout threshold, repeated pane collapse/expand, and normal exit. Screenshots of the initial document, settings, and narrow layouts were visually inspected. The fixture displays basic/indexed/RGB styling, bold/italic/underline, accents, CJK and box drawing. At extremely small dimensions, titles and status hints clip; the document remains intact and resizing back restores the normal layout. This test does not establish emoji width parity or mouse acceptance in all terminals.

Evidence from this run: `/tmp/tuxd-alacritty-evidence-4oia3ju5`. Temporary screenshots are local evidence; the checked-in script reproduces them.

## Composed-emoji rendering defect and fallback

User testing found cursor/selection drift, stale glyphs after prepending/deleting newlines, and spillover into adjacent panes. This occurred both in bare Alacritty and tmux inside Alacritty/Zed. The original generic Alacritty check did not exercise composed emoji and therefore did not cover this defect. It was reproduced in the isolated graphical harness.

The document renderer measured whole graphemes, whereas the tested terminal stacks render some component sequences at different widths. Reserving extra space alone did not resolve terminal/multiplexer disagreements. The user approved a conservative escaped Unicode preview for now and requested component-emoji rendering as near-term follow-up.

Implemented a shared display representation for layout and painting: width-sensitive clusters, ZWJ sequences and regional indicators become ASCII Unicode escapes; normal accents/CJK/single emoji remain intact. This is display-only: source bytes, styling ranges, grapheme navigation/deletion, persistence, ANSI export and clipboard payloads remain original. Editor, Code and import preview use the same representation. Narrow wrapped rows abbreviate an oversized fallback with an ellipsis; horizontal scrolling can reveal portions of its ASCII display.

Regression coverage checks display widths, hit testing across fallback cells, exact source/style round trips, and the complete framebuffer after ten newline insertions and deletions. The graphical harness now supports `--unicode` and `--tmux`; it compares initial versus restored body screenshots pixel-for-pixel, excluding title/status changes. It also verifies saved text and styling, settings, resize/collapse and exit. The isolated tmux configuration uses Ctrl+A as prefix so Ctrl+B reaches tuxd, and waits for Escape disambiguation before continuing.

Reproduce both paths:

```sh
python3 scripts/alacritty_smoke.py --unicode
python3 scripts/alacritty_smoke.py --unicode --tmux
```

Final regression runs passed in bare Alacritty and tmux + Alacritty, including zero differing pixels in the restored body. Screenshots were inspected for restored, selected and narrow layouts. Local evidence: `/tmp/tuxd-alacritty-evidence-_wzimqen` (direct) and `/tmp/tuxd-alacritty-evidence-zgyq63nd` (tmux). All 100 ordinary Rust tests pass, plus the separately run performance test, Clippy, formatting and the complete terminal smoke suite. Subsequently, the user confirmed Linux virtual console testing passed and the full Termux/Zed walkthroughs were complete and successful.

## Completion

Slice 5 is complete. Automated checks, direct/tmux Alacritty graphical regressions, and user-reported Linux virtual console, Termux and Zed acceptance have passed. No known implementation blocker remains within the documented scope. Component-emoji rendering remains a separately recorded follow-up; the approved display-only escaped fallback is the current behavior.

The user also confirmed the final escaped fallback looks correct and navigation feels natural. Slice 5 is accepted for squash-merge into `main`.
