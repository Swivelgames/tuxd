# Slice 5 — compatibility and hardening

Status: Complete — automated checks and target-terminal acceptance passed

Depends on: [Slice 4a — contextual shortcuts and settings](slice-04a-shortcut-settings.md)

Estimate: 3–5 developer days, including validation.

## Usable checkpoint

The complete MVP workflow is verified across a declared, small support matrix. Previously working features remain stable through longer editing sessions, resizing, and combined operations.

This slice broadens confidence and fixes newly discovered edge cases. Basic editing, styling, persistence, and terminal cleanup must already have passed their own slice acceptance checks.

## Target environments

User-confirmed matrix: Alacritty (primary), Linux virtual console, Termux Android via SSH, and Zed’s integrated terminal. See the [validation record](slice-05-validation.md) for environment-specific checks and verification status.

## Deliverables

- Recorded support matrix naming terminal versions/platforms and tested attribute, color, clipboard, and glyph limitations. Include the primary environment and at least one additional terminal where available; mark untested environments honestly.
- Expanded Unicode cases covering combining sequences, wide characters, emoji sequences, tabs, and style boundaries.
- Integration coverage for scroll/selection/tool/drawer interactions and narrow-window focus behavior.
- Longer edit/undo/redo/save/reopen sequences that exercise document invariants.
- Verification of terminal restoration on failure paths and useful error handling throughout the workflow.
- A documented modest-document performance target and a representative test fixture; optimize only if measurements expose a usability issue.
- Updated usage/help and clear known limitations for MVP release.

## Stability and verification

Use focused regression tests for every discovered defect. Add generated edit-sequence tests where they provide useful coverage of range validity, text preservation, and undo/redo invariants. Check output determinism and project round trips after combined operations.

Do not assert universal glyph-width or attribute parity. Record discrepancies, define supported behavior, and ensure unsupported rendering does not corrupt document data.

## Hands-on acceptance

1. Run the initiative's full MVP acceptance scenario in the primary environment.
2. Repeat core typing, selection, formatting, and export in the additional tested terminal.
3. Edit the representative fixture while scrolling, switching tools, and opening/closing the drawer.
4. Resize repeatedly during selection and tool navigation, including a very small window.
5. Exercise a longer mixed text/style edit history, undo and redo it, then save and reopen.
6. Confirm exported styles do not leak and terminal state is restored on exit/failure.
7. Record observed limitations and confirm no blocker remains in the declared supported workflows.

## Deferred

Large-file engineering, universal terminal compatibility, advanced layout features, and slice 6 code-output enhancements.

## Completion evidence

See [compatibility and performance evidence](slice-05-validation.md). Automated Unicode/history/storage checks and the performance fixture are implemented. The user verified tmux, direct SSH, and local desktop clipboard copying; the SSH clipboard defect is resolved. Alacritty rendering, keyboard input, styling, saving and resizing were verified in an isolated Xvfb session with inspected screenshots. The user confirmed successful Linux virtual console testing and full Termux/Zed walkthroughs. Installed terminals alone are not counted as verified.

User testing exposed composed-emoji rendering corruption in bare Alacritty and tmux inside Alacritty/Zed. A user-approved display-only Unicode escape fallback now passes direct and tmux graphical regression checks, including pixel equality after newline insertion/deletion. Component-emoji rendering is recorded as a near-term follow-up. See the validation record for exact scope, passing manual acceptance and documented limitations.
