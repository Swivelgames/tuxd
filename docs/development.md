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

Run from the repository root. Rust `xtask` commands use isolated PTYs and configuration directories, including cleanup checks on exit, errors, and panic. No Python or shell helpers are required.

```sh
cargo xtask smoke all
# Or target a capability:
cargo xtask smoke editor
cargo xtask smoke files
cargo xtask smoke shortcuts
cargo xtask smoke characters
cargo xtask smoke clipboard
cargo xtask smoke stdin
cargo xtask smoke themes
```

Real Alacritty checks use an isolated Xvfb display and require Alacritty, Xvfb, xdotool, tmux, and ImageMagick:

```sh
cargo xtask render unicode --tmux
cargo xtask render symbols
cargo xtask render characters --tmux
cargo xtask widths
```

`cargo xtask check --full` runs core checks and the entire terminal/rendering matrix. These integration checks currently require Linux x86-64 and fail explicitly on unsupported hosts. Inspect the generated screenshots as well as test results; terminal emulation does not establish font/rendering parity everywhere.

See [Maintainer tooling](tooling.md) for ownership, commands, and the migration coverage map.

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

## Prepare a crates.io publication

1. Publish the source and documentation to the GitHub repository linked in `Cargo.toml`.
2. Confirm the release version and tag include the final manifest and documentation.
3. Validate the packaged source without uploading:

```sh
cargo publish --dry-run --locked
```

4. Publish only when authorized:

```sh
cargo publish --locked
```

5. After the registry makes the version available, verify `cargo install tuxd --locked` in a clean install root.

The package includes source, tests, examples, user documentation, and license notices. Internal `.work/` records are excluded. README images and links use absolute GitHub URLs so the registry page can render them after source publication.

## Release tooling

Use [`cargo xtask`](releases.md) for independently runnable checks, screenshots, packaging, and opt-in publication. Its unpublished crate and lockfile live in `xtask/`, separate from the application.
