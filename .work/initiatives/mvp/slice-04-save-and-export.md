# Slice 4 — save and export workflow

Status: Planned

Depends on: [Slice 3](slice-03-tool-panel.md)

Estimate: 2–3 developer days, including validation.

## Usable checkpoint

Create a styled document, save it, reopen it without losing formatting, and reuse generated ANSI outside the app. A failed save, open, copy, or export leaves the current document intact and gives actionable feedback.

## Deliverables

- Versioned project file containing text and style data, independent of terminal/UI state. Select and document the exact format during implementation.
- New, open, save, and save-as actions with keyboard-accessible path entry.
- Dirty-state tracking, current filename, and save/discard/cancel handling before destructive document replacement or quit.
- Validated loading: reject malformed ranges and unsupported versions without partially replacing the active document.
- Safe writes using a temporary file and atomic replacement where supported, plus deliberate handling of overwrites and write failures.
- ANSI file export containing actual escape bytes and preserved newlines, with a terminal style reset at completion.
- Escaped-text export and copying the displayed escaped ANSI representation; label these distinctly from raw ANSI export.
- Build on Slice 2’s escaped-ANSI clipboard support and success/failure feedback; add file export as a fallback when clipboard access is unsupported.
- Focus navigation, help, and shortcuts for these new workflows, integrated with the existing editor and tools.

## Stability and verification

Test project round trips for all styles/colors, Unicode, tabs, and multiline text. Cover malformed files, unsupported versions, failed writes, canceled dialogs, and dirty-state transitions across undo/redo and saves.

Verify raw exported bytes against the serializer and confirm escaped output contains readable escape notation. Verify failed persistence and clipboard operations cannot clear edits or falsely report success. Re-run prior slice walkthroughs.

## Hands-on acceptance

1. Create a styled multiline document and save it to a new project file.
2. Quit, reopen, and compare text, colors, and attributes.
3. Edit it, invoke new/open/quit, and exercise cancel and save paths.
4. Try an invalid project and an unwritable destination; confirm the working document remains available.
5. Export raw ANSI and render the generated file in the supported terminal; verify text and style transitions and that following output has normal styling.
6. Copy escaped ANSI into a plain text editor and compare it with the drawer.
7. Exercise the clipboard-unavailable path and export escaped text to a file instead.

## Deferred

Arbitrary ANSI import, autosave/recovery, recent-file menus, rich-text clipboard formats, and language code generation.

## Completion evidence

Pending implementation: launch command, checks/results, terminal/platform, clipboard mechanism, and limitations.
