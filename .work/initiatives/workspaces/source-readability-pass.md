# Remaining-source readability pass

Checkpoint: `2ceec37`, committed and pushed to GitLab and GitHub before this pass.

## Review coverage

This pass follows the buffer/workspace review and examines the remaining application
owners: startup and terminal lifecycle, Settings, shortcut compilation/resolution,
ANSI parsing, editing/navigation, clipboard transport and transactions, text
presentation, character discovery, file picking/import review, configuration,
native persistence, and the maintainer-tool entry point.

The review targets handwritten logic and its boundaries. Generated character data,
fixed shortcut fixtures, and declarative catalogs are not rewritten for style.

## Changes

| Owner | Improvement | Preserved contract |
| --- | --- | --- |
| Shortcut compiler | Name resolution and validation of one binding slot | Validation order/messages, leader expansion, prefix conflicts, editable scopes |
| ANSI parser | Extended-color consumption owns mode, channels, validation, and cursor advance | Whole-SGR atomicity and exact errors |
| Clipboard processes | Copy and paste share child completion, timeout, and cleanup | Same deadline, polling interval, pipe ordering, and failure behavior |
| Settings | One operation chooses a category's initial content focus | Enter, Right, and footer Up retain the same destinations |
| Import review | Preview and error table share inert character notation | Printable text, source offsets, marks, and escaped controls |
| Editing/Code | Compute vertical distance once; use a local viewport borrow | Wrapped/unwrapped movement and page/endpoint semantics |
| Configuration and declarations | Keep module assembly together and separate methods visually | No interface or serialization changes |

## Boundaries retained

- Terminal guard construction/restoration remains explicit and ordered.
- Storage byte publication remains separate from document/configuration policy.
- Clipboard copy/paste keep different pipe directions and result validation.
- Review display notation stays separate from the ANSI export encoder.
- Settings appearance/shortcut navigation retains its intentional differences.
- Character catalog/search and its flow-grid presentation keep separate owners.
- Startup and maintainer entry points remain orchestration; no generic dispatcher
	or additional crate is introduced.

## Validation

- `cargo xtask check`: passed formatting, debug/release tests including ignored
	checks, and warnings-denied application/xtask clippy.
- `cargo xtask smoke all`: passed Editor, buffers/files, shortcuts/leader,
	Characters, clipboard/TMUX, stdin, and theme/Settings acceptance.
- Final source review and whitespace checks completed. No new device or graphical
	walkthrough is claimed; there are no intended layout changes.
