# Browse a directory

[Documentation](README.md) · [Save, import, and export](files.md)

## Open a folder

Run `tuxd` without arguments to open the current working directory. To choose another folder:

```sh
tuxd ./my-project
```

Or choose **F4 → Open directory** (**Ctrl+Shift+O**): navigate into the folder, then activate **Open directory**. You can also paste a directory path into the field and press Enter.

Opening another directory replaces the tree and keeps every open document, including unsaved edits. It does not change the process working directory.

## Use the sidebar

| Area | Action |
| --- | --- |
| Open Files | Select a document; `*` marks unsaved changes; `x` closes it |
| Directory tree | Expand a folder, then select a file to open it |
| Section headings | Collapse or expand Open Files or the directory tree |
| Scrollbar | Scroll, click the track, or drag the thumb |

Native documents open directly. Opening the same native file again selects its existing buffer. With the filter disabled, other files offer the existing Raw / Escaped ANSI import dialog.

The sidebar replaces document tabs while visible. On terminals of 64 columns or fewer it overlays the editor; opening a document or returning to Editor dismisses it. On wider terminals it takes its own column. Tools move into the lower tabs if the remaining editing area is narrow.

## Keyboard controls

Defaults are configurable in **Settings → Shortcuts**. Explorer navigation bindings are shared with file dialogs.

| Task | Default |
| --- | --- |
| Open directory | Ctrl+Shift+O |
| Show / hide sidebar | Ctrl+Shift+E, or click tuxd in the title |
| Focus sidebar | Ctrl+Alt+E |
| Cycle panes | F6 |
| Return to Editor | Esc |
| Previous / next entry | Up / Down, k / j |
| Expand folder / enter its children | Right, l |
| Collapse folder / select parent | Left, h, Backspace |
| Open file or toggle section | Enter, Space |
| Jump between sections | Tab, Shift+Tab |
| First / last entry | Home / End, gg / G |
| Middle visible entry | M |
| Page | PageUp / PageDown |
| Close selected Open Files document | Ctrl+F4 |
| Show / hide dotfiles | Ctrl+H, Ctrl+. |
| Native documents / all files | Ctrl+F |
| Refresh directory | Ctrl+R |

Dotfiles are hidden initially. The initial filter shows directories and `.tuxd.json` files. Directories load when expanded; directory symlinks and special files are omitted. Refresh discovers external changes, including newly saved files.

Directory and open-file state are session-only. Workspace persistence, file creation/rename/delete, file watching, and external-change detection are separate future work.
