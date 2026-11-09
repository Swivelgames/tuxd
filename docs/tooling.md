# Maintainer tooling

All repository automation lives in the unpublished `xtask` crate. Run `cargo xtask --help` from the checkout root. The application has no dependency on this crate or its separate lockfile.

## Commands

| Task | Command |
| --- | --- |
| Rust formatting, tests, lint, release performance test | `cargo xtask check` |
| Complete Linux acceptance/rendering matrix | `cargo xtask check --full` |
| Regenerate checksum-pinned catalog | `cargo xtask generate catalog` |
| Regenerate dependency notices offline | `cargo xtask generate notices` |
| PTY scenarios | `cargo xtask smoke all` |
| Real Alacritty input/rendering | `cargo xtask render basic` |
| Unicode, symbols, or picker rendering | `cargo xtask render unicode`, `symbols`, or `characters`; add `--tmux` |
| Measure protected catalog cell widths | `cargo xtask widths` |
| Diagnose unprotected characters | `cargo xtask widths --raw` (mismatches can be expected) |
| Regenerate documentation images and GIF | `cargo xtask screenshots` |
| Capture media outside the checkout | `cargo xtask screenshots --output /tmp/tuxd-media` |
| Smoke-test a prebuilt executable | `cargo xtask verify-binary PATH VERSION` |
| Prepare/publish releases | See [Release workflow](releases.md) |

## Ownership

| Component | Owns | Does not own |
| --- | --- | --- |
| `generate/catalog` and `catalog_policy` | Pinned downloads/checksums, Unicode and icon parsing, admission/grouping, deterministic TSV | Runtime character search or UI |
| `generate/notices` | Locked package metadata, license discovery, text normalization and deduplication | Application dependency selection |
| `terminal/session` | PTY lifetime, input, bounded output reads, parsed screen, resize, terminal restoration | Scenario-specific expected behavior |
| `terminal/fixtures` | Test document setup and application builds | Scenario assertions |
| `terminal/clipboard`, `terminal/tmux` | Isolated clipboard adapter and scoped private tmux server | User clipboard or user tmux sessions |
| `process/child` | Bounded waits and guaranteed child cleanup | Scenario policy |
| `terminal/scenarios` | Editor, files, shortcuts, characters, clipboard, stdin, themes assertions | PTY implementation |
| `terminal/unix` | Controlling-terminal/redirected-stdin checks, Unix paths, raw cursor-query mode | Portable scenario orchestration |
| `desktop/display` | Private Xvfb display, isolated Alacritty window, keyboard/capture operations | Capture content or application behavior |
| `desktop/media`, `rendering`, `widths` | Documentation captures, pixel restoration tests, cursor-width comparison | User desktop state |
| `check` and `release` | Compose independently runnable capabilities | Their private implementation details |

`portable-pty` supplies PTY/process operations; `vt100` replaces the former `pyte` screen parser. Rust helpers replace generated shell clipboard scripts. Raw terminal bytes remain available for exact OSC52 and cleanup assertions, alongside parsed text and cell styles. [portable-pty](https://docs.rs/portable-pty/0.9.0/portable_pty/), [vt100](https://docs.rs/vt100/0.16.2/vt100/).

Child processes and isolated tmux/Xvfb resources have scoped cleanup. Failures retain rendering evidence but do not replace personal files, clipboard contents, settings, or the active desktop. Waits and cursor queries have deadlines.

## Platform boundaries

- Generators, command dispatch, archive handling, and publication use portable Rust APIs. Catalog downloads use HTTPS with checksum verification.
- Linux uses rustls for HTTPS; Windows uses its native certificate store. Tooling can be cross-checked without compiling a Windows C TLS dependency.
- Acceptance and desktop commands currently require Linux x86-64. They explicitly reject unsupported platforms, rather than marking unrun checks successful.
- Native terminal/font behavior still needs relevant manual testing. Neither PTY emulation nor width measurements establish font coverage.
- Screenshot capture still requires Alacritty, Xvfb, xdotool and ImageMagick. The tmux variants require tmux. Python and `uv` are no longer dependencies.

## Coverage preserved from `scripts/`

| Former script | Replacement | Preserved contracts |
| --- | --- | --- |
| `update_character_catalog.py` | `generate catalog` | Pinned hashes, admission, names/aliases, Nerd families, CLDR emoji order; byte-identical TSV |
| `update_dependency_notices.py` | `generate notices` | All locked platforms, nested/custom/supplemental licenses, normalized text hashes; only generator attribution changes |
| `capture_docs.py` | `screenshots` | 14 themes, overview, settings, social preview, eight-frame GIF, isolated showcase/config |
| `terminal_smoke.py` | `smoke editor`, `files`, `shortcuts`, `clipboard` | Editing/style/wrap/mouse, file dialogs, malformed startup, shared fields, binding capture/conflicts, error/panic cleanup |
| `characters_smoke.py` | `smoke characters` | Search, collapse, repeated insertion/undo, emoji, resize, unsupported exclusion |
| `clipboard_smoke.py` | `smoke clipboard` | Exact >10 KiB OSC52 payload, tmux forwarding/buffer, helper failure |
| `editor_clipboard_smoke.py` | `smoke clipboard` | Styled copy/cut/paste, undo, import review accept/abort, exact persistence |
| `stdin_import_smoke.py` | `smoke stdin` | Raw/escaped pipes and files, EOF keyboard handoff, reviews, UTF-8/size limits, no-controlling-terminal errors |
| `leader_smoke.py` | `smoke shortcuts` | Capture, affected-binding warning, explicit save/unset |
| `theme_smoke.py` | `smoke themes` | Overrides, fallback, original document/config preservation |
| `settings_theme_smoke.py` | `smoke themes` | Preview/discard, save/restart, presets and role reset |
| `alacritty_smoke.py` | `render` | Real keyboard input, saved styles, resize/slim/collapse, pixel-identical restoration after newlines |
| `character_width_smoke.py` | `widths` | Every sample compared to real cursor reports, bare and tmux, raw diagnostic mode |
| `packaged_binary_smoke.py` | `package` / `verify-binary` | Extracted binary version, editing/save, settings, resize, terminal cleanup |

Escape input boundaries are intentional: a standalone Escape must be allowed to resolve before sending an ordinary character, or the terminal can interpret the pair as an Alt chord. Preserve those boundaries when changing scenarios.
