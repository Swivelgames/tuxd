# Development and validation

[Documentation index](README.md) · [Project overview](../README.md)

Build from a checkout with `cargo build --release --locked`. Cargo’s target directory may be overridden by your environment; `cargo metadata --no-deps --format-version 1` reports the effective path.

Read [AGENTS.md](../AGENTS.md), the [readability guide](code-organization-and-readability.md), and [architecture](architecture.md) before meaningful changes. The architecture migration is complete; the [initiative record](../.work/initiatives/composable-architecture/overview.md) preserves its contracts and validation evidence.

## Core checks

```sh
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
```

## Terminal acceptance

Run from the repository root. `uv` supplies the Python scripts’ pinned dependencies. PTY checks exercise real input and terminal behavior, including cleanup on exit, errors, and panic.

```sh
uv run scripts/terminal_smoke.py
uv run scripts/clipboard_smoke.py
uv run scripts/editor_clipboard_smoke.py
uv run scripts/stdin_import_smoke.py
uv run scripts/leader_smoke.py
uv run scripts/theme_smoke.py
uv run scripts/settings_theme_smoke.py
```

Real Alacritty checks use an isolated Xvfb display and require Alacritty, Xvfb, xdotool, and ImageMagick:

```sh
python3 scripts/alacritty_smoke.py
```

Inspect the generated screenshots as well as the script results. Automated terminal emulation does not establish font/rendering parity on every terminal.

## Performance and cleanup diagnostics

```sh
cargo test --release --locked modest_document_performance -- --ignored --nocapture
cargo run --locked -- --diagnostic-error
cargo run --locked -- --diagnostic-panic
```

The last two commands intentionally fail after terminal initialization; terminal echo, cursor, and screen state should recover. They are developer diagnostics, not normal startup modes. Set `TUXD_PERF_FIXTURE=/tmp/tuxd-performance.tuxd.json` on the performance command to write its representative fixture.

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for the pull-request workflow, [SECURITY.md](../SECURITY.md) for private vulnerability reports, and [third-party notices](../THIRD_PARTY_NOTICES.md) for attribution and dependency notice maintenance.

## Package and license

The package, binary, and library crate are named `tuxd`. The version comes from Cargo metadata; `tuxd --version` must work without a terminal. The project is licensed under [GPL-3.0-only](../LICENSE). Keep distribution documentation aligned with the artifacts actually available; do not advertise unpublished registry packages or release downloads.
