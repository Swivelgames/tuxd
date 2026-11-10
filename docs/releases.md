# Release a version

Use **`cargo xtask`** from a clean `main` checkout after merging the work to release. Choose an explicit stable version, such as `0.3.0`, and write reviewed release notes outside the checkout.

## Pre-v1 versioning

| Change | Version increment |
| --- | --- |
| Features, fixes, and other compatible improvements | Patch (`0.3.0` → `0.3.1`) |
| Major breaking changes | Minor (`0.3.x` → `0.4.0`) |

Before v1, minor releases are reserved for major breaking changes. New features
alone do not require a minor release.

## Preview, then run

```sh
# Prints the plan; does not change source, commit, tag, push, or publish.
cargo xtask release 0.3.0 --all

# Validate, regenerate media, commit, tag, and package locally.
cargo xtask release 0.3.0 --notes /tmp/release-notes.md --execute

# Or prepare and publish everywhere.
cargo xtask release 0.3.0 --notes /tmp/release-notes.md --execute --all
```

Cargo may compile the tooling on first use, including downloading its dependencies. The release preview itself performs no release operations.

## Publication switches

Combine switches as needed. Publication is **opt-in**, and `release` and `publish` both require `--execute` to make changes.

| Switch | Effect |
| --- | --- |
| None | Prepare locally; no push or publication |
| `--github` | Push `public`, upload and verify GitHub draft assets, then publish the release |
| `--gitlab` | Push `origin`, create a GitLab release with assets |
| `--crates-io` | Push `public` so source is available, publish the crate, verify a registry installation |
| `--all` | All three destinations and both Git remotes |
| `--push` | Push both remotes without requiring hosted releases or registry publication |

`origin` must point to `gitlab.com:swivel/tux-design`; `public` must point to `github.com:swivelgames/tuxd`. Fetch and push URLs are checked. Pushes include `main` and the version tag, are atomic per remote, and never force history.

## Platform support

The coordinator uses Rust filesystem/process APIs without a shell. Archives and checksums use Rust libraries, not platform-specific `tar` or `sha256sum` commands.

| Capability | Where it runs |
| --- | --- |
| Plan, version update, Git operations, GitHub/GitLab upload | Portable Rust tooling, with Git/Cargo and the selected hosting CLI installed |
| crates.io publication | Uses Cargo verification and installation; requires a host supported by tuxd |
| `check` | Rust checks on the host; the application's own target support still applies |
| `check --full`, `screenshots` | Linux x86-64; Rust PTY and Xvfb/Alacritty helpers |
| `package`, complete `release --execute` | Linux x86-64; currently produces the reviewed static-musl artifact |

Unsupported required steps fail explicitly. They are never silently omitted. A portable coordinator does **not** promise that tuxd itself or the full terminal test suite supports every operating system.

For a release initiated elsewhere, perform preparation on a Linux runner and transfer the prepared directory alongside a clean checkout at the release tag. `publish` can then verify the receipt and upload from that checkout. This workflow does not yet install or trigger CI runners automatically.

## Prerequisites

- Rust/Cargo **1.92+** for the unpublished `xtask` crate, plus Git. The application's own MSRV is unchanged.
- For version updates, the pinned helper:

```sh
cargo install cargo-release --version 1.1.5 --locked
```

- For full preparation: reviewed Rust **1.95.0**, its `rust-docs` component, the `x86_64-unknown-linux-musl` target, C build tools, `strip`, and `readelf`.
- For terminal/media checks: Alacritty, Xvfb, xdotool, tmux, ImageMagick (`magick` and `import`), and the documentation fonts.
- For selected destinations: authenticated `gh`, authenticated `glab`, or a configured crates.io publishing credential.

Release orchestration, generators, media capture, and acceptance tests are Rust. See [Maintainer tooling](tooling.md) for the shared infrastructure. Changing the pinned compiler requires reviewing `licenses/runtime/`.

## Independent steps

| Command | Responsibility |
| --- | --- |
| `cargo xtask preflight VERSION` | Clean main, increasing version, upstream ancestry, unused tag; fetches both remotes |
| `cargo xtask version VERSION --execute` | Delegate manifest/lockfile updates to `cargo-release version` |
| `cargo xtask check` | Formatting, debug tests, Clippy for app and tooling; release tests including ignored performance test |
| `cargo xtask check --full` | Also PTY checks, bare/tmux Alacritty checks, character-width checks, dependency notices |
| `cargo xtask screenshots` | Documentation screenshots, GIF, social preview |
| `cargo publish --dry-run --locked --allow-dirty` | Verify registry package before committing generated files |
| `cargo xtask snapshot VERSION --execute` | Commit expected release files and create an annotated tag |
| `cargo xtask package VERSION --output DIR --notes FILE` | Archives, notices, offline rebuild, extracted-binary smoke test, checksummed receipt |
| `cargo xtask publish DIR --all --execute` | Verify prepared source/files, then push and publish selected destinations |

`release` coordinates these steps in order. Versioning uses `cargo-release`'s individual version step with isolated configuration; it cannot publish or push. Git commit/tag handling stays explicit because screenshots and notices must be included in the same release snapshot, and pushes target two remotes. Packaging retains the existing corresponding-source and offline-rebuild checks; adding multi-platform binary distribution with `cargo-dist` remains a separate extension.

## Review and recovery

The command prints its retained output directory and `release.log`. Use `--output /absolute/new/directory` to choose a location outside the repository. Subprocess output is captured in the log during orchestration; individual commands print it directly.

After local preparation, review the screenshots, notes, and archives, then publish:

```sh
cargo xtask publish /path/to/prepared-release --all --execute
```

- The receipt binds notes and assets to the clean tagged source commit. Edited files fail verification.
- Failures stop the workflow. There is no automatic rollback, tag replacement, or overwrite of existing hosted releases.
- Fix a failed local step and rerun that step. Do not rerun version preparation after a tag already exists. Packaging requires an output location without an existing `artifacts/` directory.
- After partial publication, inspect remote state and select only unfinished destinations. A registry installation check can fail after the immutable crate publication succeeds.
- A GitHub upload failure may leave a draft. Repair and verify its assets before publishing manually. GitLab creation refuses to update an existing release.
- Automated checks do not replace visual review or relevant manual Termux, Zed, and Linux virtual console walkthroughs.
