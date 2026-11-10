# Shared workspace frame

The buffers/tabs checkpoint was committed as `ac6fd17` and pushed to GitLab and
GitHub before this iteration. Template grammar drafts were committed as `76c7619`.

## Implemented layout

- One rounded outer frame contains Editor, Code, and the tools sidebar.
- The top border holds tuxd and document tabs; Editor no longer has a redundant title box.
- Editor position and application notices sit directly above the Code divider.
- Code and the tool sections use connected horizontal dividers instead of cards.
- Sidebar scrolling and nested tool scrollbars occupy separate tracks inside the frame.
- Editor and Code scrollbars stay inside their bounds without covering the status line.
- Slim layout retains Code/Style/Color/Characters tabs in the lower divider.
- F2/F5 collapse behavior and F3 wrapping remain available. The collapsed Editor
	label expands it by mouse; Code retains its disclosure and Wrap controls.

`workspace/layout.rs` owns geometry, Workspace composes the shared frame, and
Panel paints its section dividers. No project explorer, workspace metadata, or
Tokens placeholder is introduced. Those mockups guide future work.

## Verification

- Rust tests, formatting, and clippy passed through `cargo xtask check`.
- Additional wide/slim regression coverage verifies the outer frame, editor-local
	status, and non-overlapping scrollbar tracks.
- Isolated Alacritty Basic passed, including small/large resize captures:
	`/tmp/tuxd-alacritty-evidence-LJRmLK`.
- All PTY suites passed across the full run and focused follow-ups: Editor, Files
	(including buffers), Shortcuts, Characters, Clipboard, stdin, and Themes.
- Final Rust tests and clippy passed again after adjusting nested scrollbar hit
	bounds. The frame regression passed at 40, 64, 65, and 100 columns.
