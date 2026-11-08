# Contributing to tuxd

Small fixes, documentation improvements, terminal compatibility reports, and focused features are welcome.

## Choose a starting point

| I want to… | Start here |
| --- | --- |
| Report a bug | [Open an issue](https://github.com/swivelgames/tuxd/issues) with reproduction steps |
| Suggest a feature | Describe the user workflow in an issue before a substantial implementation |
| Improve documentation | Edit the relevant task guide under [docs/](docs/README.md) |
| Report a vulnerability | Use the private route in [SECURITY.md](SECURITY.md) |

Check existing issues and the [backlog](.work/deferred-work.md) before starting larger changes. Discuss new dependencies, file formats, and architectural changes first.

## Develop a change

1. Fork the repository and create a branch from `main`.
2. Read [AGENTS.md](AGENTS.md) and the [readability guide](docs/code-organization-and-readability.md).
3. Use the [architecture map](docs/architecture.md) to find the responsible component.
4. Make a focused change and add regression coverage where behavior warrants it.
5. Update task-oriented documentation and screenshots when the interface changes.

Build and launch from the checkout:

```sh
cargo run --locked -- examples/showcase.tuxd.json
```

Use tabs where the format permits them; otherwise use four spaces. Keep unrelated formatting changes out of the patch.

## Validate

```sh
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
```

For terminal, input, layout, or clipboard changes, run the relevant [PTY acceptance checks](docs/development.md#terminal-acceptance). Include the terminal, tmux/SSH context, and results for manual checks. Passing emulation checks does not establish visual parity on every terminal.

For dependency changes, regenerate [license notices](licenses/dependencies.txt):

```sh
cargo fetch --locked
python3 scripts/update_dependency_notices.py
```

If a package omits its license text, add a source-linked supplement as described in [third-party notices](THIRD_PARTY_NOTICES.md). Keep upstream notices when adding or adapting external material.

## Open a pull request

Include:

- The user-visible problem and resulting behavior.
- Relevant issue links and any compatibility considerations.
- Checks run and their results; identify checks you could not run.
- Screenshots or a short recording for visual changes.

Keep each pull request centered on one coherent change. Never attach secrets, private documents, or unsanitized clipboard contents.

## Licensing

Original contributions are submitted under tuxd’s [GPL-3.0-only license](LICENSE). Only submit material you have the right to contribute. Identify third-party content and preserve its license and attribution notices. Existing third-party components retain their own licenses; see [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
