# CLI reference

[Documentation index](README.md) · [Project overview](../README.md)

```text
tuxd [document.tuxd.json]
tuxd --raw INPUT
tuxd --escaped INPUT
tuxd --help
tuxd --version
```

| Argument | Behavior |
| --- | --- |
| No arguments | Open a new, empty document |
| Document path | Open a native `.tuxd.json` document |
| `--raw INPUT` | Import UTF-8 text containing actual ANSI escape bytes |
| `--escaped INPUT` | Import UTF-8 text containing escaped ANSI notation |
| `-h`, `--help` | Print usage and exit without entering the terminal UI |
| `-V`, `--version` | Print `tuxd` and its package version, then exit without entering the terminal UI |

`INPUT` is a file path, or `-` for stdin. Bare `-` is rejected: select the format explicitly. Paths beginning with `-` can be supplied using an absolute path or a `./` prefix. A `--` argument separator is not implemented.

```sh
tuxd ./greeting.tuxd.json
tuxd --raw ./output.ansi
tuxd --escaped ./output.txt
printf '\033[31mRed\033[0m\n' | tuxd --raw -
printf '%s' '\x1b[31mRed\x1b[0m\n' | tuxd --escaped -
```

Both help aliases include the package version, usage, public options, and examples. Help and version require no interactive terminal.

## Input and output

Imports must be UTF-8 and at most 16 MiB. Empty input is valid. Imported content opens unsaved. Unsupported controls require review before stripping; unrecoverable parsing errors fail startup. Original files remain unchanged.

For stdin import, tuxd reads to EOF before opening the UI, then takes keyboard input from the controlling terminal. Stdout must be a terminal and `/dev/tty` must be available. This is an interactive import workflow, not a headless conversion command. See [Files and formats](files.md) for supported escapes and SGR parameters.

Normal UI exit, help, and version return success. Invalid arguments, startup failures, and returned runtime errors exit unsuccessfully. Scripts should depend on success versus failure rather than particular nonzero codes. Help and version work with redirected stdout.

## Configuration

Settings are loaded from `$XDG_CONFIG_HOME/tuxd/`, falling back to `~/.config/tuxd/`. `theme.json` controls appearance and `shortcuts.json` controls bindings, leader, and sequence timeout. There are no command-line configuration overrides. See [Settings](settings.md).

## Developer diagnostics

`--diagnostic-error` and `--diagnostic-panic` intentionally fail after terminal initialization to exercise cleanup. They are documented in [Development](development.md).
