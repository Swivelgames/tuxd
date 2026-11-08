# Save, import, and export

[Documentation](README.md) · [CLI reference](cli.md)

**Choose a task:** [Save](#save-an-editable-document) · [Import](#import-existing-ansi) · [Export](#export-for-another-project) · [Browse files](#file-picker-shortcuts)

## Choose the right format

| Format | Use it for | Contains |
| --- | --- | --- |
| `.tuxd.json` | Continuing your work in tuxd | Editable text and formatting |
| Raw ANSI | Output consumed by a terminal | Actual ANSI escape bytes |
| Escaped ANSI | Copying or embedding escape notation | Literal `\x1b`, `\n`, `\t`, and `\\` |

Each native file holds **one document**. tuxd currently opens one document at a time.

## Save an editable document

1. Press **Ctrl+S**.
2. Choose a directory and enter a filename, such as `greeting.tuxd.json`.
3. Choose **Save**. Confirm overwriting if prompted.

| Task | Shortcut |
| --- | --- |
| Open a document | Ctrl+O |
| Save | Ctrl+S |
| Save under a new name | Ctrl+Shift+S |
| New document | Ctrl+N |
| Open the File menu | F4 |

- Save As adds `.tuxd.json` when you enter no extension.
- Saving preserves undo history.
- The title’s `*` disappears after saving, or after undoing back to the saved content.
- Imported content starts unsaved; saving asks for a native document destination.

## Import existing ANSI

1. Open **F4 → Import**.
2. Choose **Raw ANSI** or **Escaped ANSI**; Escaped is initially selected.
3. Choose the file and activate **Import**.
4. Review unsupported controls if prompted.
5. Save as `.tuxd.json` if you want to keep editing later.

Opening a non-`.tuxd.json` file through **Ctrl+O** also offers ANSI import.

### Review unsupported controls

| Choice | Result |
| --- | --- |
| Import anyway | Strip the listed controls and import the remaining text/styles |
| Abort | Leave the current document unchanged |

The dialog highlights erroneous sequences in its source preview and lists their reasons. Scroll the preview and error table independently. The source file is unaffected.

> Input must be **UTF-8**, at most **16 MiB**. Unterminated control strings can fail import rather than offer stripping. tuxd imports styled text, not terminal screen recordings.

Try the review with `tuxd --escaped examples/import-review.txt` from the checkout.

### Import from a pipe

```sh
# Actual escape bytes.
printf '\033[31mRed\033[0m\n' | tuxd --raw -

# Literal escaped notation.
printf '%s' '\x1b[31mRed\x1b[0m\n' | tuxd --escaped -
```

- Choose the format explicitly; bare `-` is rejected.
- Input is read to EOF before the UI opens.
- Keep stdout attached to a terminal; keyboard input needs a controlling terminal.

See [CLI reference](cli.md) for file arguments and startup behavior.

## Export for another project

1. Press **Ctrl+E**.
2. Choose **Raw ANSI** or **Escaped ANSI**.
3. Choose a destination and activate **Export**.
4. Confirm overwriting if prompted.

| Export behavior | Result |
| --- | --- |
| Raw | Preserves newlines/tabs and includes a final style reset |
| Escaped | Matches Code’s clipboard payload without display-only line breaks |
| Document state | Native path and unsaved marker stay unchanged |
| Active native document as destination | Rejected to protect the editable file |

**Export is not Save.** Save a `.tuxd.json` copy to preserve your editable work.

## File picker shortcuts

The file list starts focused. Click controls or use **Tab / Shift+Tab** to move between them.

| Task | Shortcut |
| --- | --- |
| Choose a file / enter a directory | Enter or double-click |
| Focus Location | Ctrl+L |
| Return to working directory | Ctrl+G |
| Parent directory | Backspace or Left |
| Toggle hidden files | Ctrl+H or Ctrl+. |
| Toggle document/all-file filter | Ctrl+F |
| Paste a path / select field contents | Ctrl+V / Ctrl+A |
| Choose Raw / Escaped when its group is focused | Left / Right |
| Cancel | Esc |

| File-list navigation | Keys |
| --- | --- |
| Up / down | Arrows or k / j |
| Parent / enter directory | h / l |
| First / last entry | Home / End or gg / G |
| Middle visible entry | Shift+M |
| Page | Page Up / Page Down |

Paths may be absolute, relative to the displayed directory, or start with `~/`. The picker starts beside the current document or its import source; otherwise it uses the working directory. An untitled document’s export can establish its initial directory.

## Handle unsaved changes

New, Open, import, and Quit prompt before replacing unsaved work. **Cancel is initially focused.**

| Choice | Result |
| --- | --- |
| Save | Save first, then continue |
| Discard | Continue without saving |
| Cancel | Keep editing |

There is no autosave, crash recovery, or external-change detection. Avoid concurrent edits to the same file.

**Reference:** [Native format and supported ANSI](formats.md) · [Troubleshooting](troubleshooting.md)
