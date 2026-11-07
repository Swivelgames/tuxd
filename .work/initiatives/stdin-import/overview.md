# CLI stdin import

Implemented on `feat/stdin-import`; awaiting user review.

- `--raw INPUT` and `--escaped INPUT` select the format explicitly; input is a file path or `-`.
- Piped/redirected stdin is consumed to EOF before terminal setup. Keyboard input resumes through `/dev/tty`.
- UTF-8 and 16 MiB limits match file import. Empty EOF is valid.
- Imports are unsaved; file origins guide subsequent file dialogs, while stdin uses CWD.
- Unsupported controls reuse the import review, including strip/continue and abort. Unrecoverable parse failures remain errors.
- No implicit stdin detection, streaming editing, or terminal-screen reconstruction.

Validation: unit tests, formatting, clippy and PTY acceptance checks. The dedicated `scripts/stdin_import_smoke.py` covers real pipes, raw/escaped file imports, review accept/abort, saving imported content/styles, post-EOF keyboard input, terminal restoration, invalid UTF-8, oversized input, invalid arguments, interactive stdin misuse and absent controlling terminal.
