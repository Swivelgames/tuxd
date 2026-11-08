# Security policy

## Report privately

Use **[Report a vulnerability](https://github.com/swivelgames/tuxd/security/advisories/new)** on GitHub. Sign in, describe the problem, and submit the private report.

**Do not put exploit details or sensitive documents in a public issue or pull request.** Ordinary bugs and feature requests belong in the [issue tracker](https://github.com/swivelgames/tuxd/issues).

## What to include

- `tuxd --version`, or the commit you tested.
- Linux distribution, terminal/version, and whether tmux or SSH is involved.
- Reproduction steps and a minimal, sanitized input file or configuration.
- Expected behavior, observed behavior, and potential impact.
- Any relevant clipboard helper or terminal configuration.

A plausible report is welcome even if you cannot provide a complete exploit. Only use systems and data you are authorized to test.

## Version scope

Security work targets current `main` and the latest published release, when one exists. Older releases do not have guaranteed backports. If possible, check whether the problem reproduces on current code; still report findings from older versions with their version information.

## Handling reports

Maintainers will use the private report to investigate, ask questions, and coordinate a fix and disclosure. Response and release times depend on maintainer availability; there is no guaranteed response deadline or bounty program.

Relevant areas include untrusted document/ANSI parsing, terminal escape handling, clipboard subprocesses, configuration loading, and file publication. Include the affected workflow so the report reaches the right owner.
