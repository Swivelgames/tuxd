# Edit, style, and copy text

[Documentation](README.md) · [Getting started](getting-started.md)

**Choose a task:** [Style text](#style-a-selection) · [Pick a color](#pick-a-color) · [Insert a character](#insert-a-character) · [Copy and paste](#clipboard-and-ansi-output) · [Arrange panes](#arrange-the-panes)

All shortcuts below are defaults. Press **F1** to inspect or change them.

## Style a selection

1. Select text with a mouse drag or **Shift+arrows**.
2. Press **Ctrl+B**, **Ctrl+I**, or **Ctrl+U**, or choose a Style button.
3. Press **Esc** to clear the selection and inspect the result.
4. Use **Ctrl+Z** to undo or **Ctrl+Y** to redo.

| Situation | Result |
| --- | --- |
| Text is selected | Formatting changes the selection |
| Nothing is selected | Formatting applies to subsequent typing |
| Selection has mixed formatting (`~`) | Toggling an attribute enables it throughout |
| You choose Clear | Attributes and both color channels return to defaults |
| You click a tool | Your selection stays available |

Moving the cursor adopts the preceding character’s style. Replacing a selection inherits its first character’s style.

## Pick a color

1. Select text, or leave the cursor where you want to type.
2. Choose **Foreground** or **Background** in Color.
3. Choose a color mode:

| Mode | How to use it |
| --- | --- |
| Basic | Choose one of 16 terminal colors |
| 256 | Choose an indexed color; scroll to reach more swatches |
| RGB | Enter `#RRGGBB` or adjust the R/G/B values, then choose Apply |

**Reset** clears the selected color channel. Switching modes or Foreground/Background does not change text until you apply a color.

### Adjust an RGB value

| Input | Effect |
| --- | --- |
| Left / Right | Decrease / increase by 1 |
| Shift+Left / Shift+Right | Decrease / increase by 10 |
| Home / End | Set 0 / 255 |
| Click, drag, or wheel on a gauge | Adjust the value |
| Enter or Apply | Apply the staged color |

The preview swatch shows the candidate color. Incomplete or invalid values leave the document unchanged.

## Insert a character

1. Place the cursor, or select text to replace.
2. Open **Characters** and choose a **Group** and **Type**, or use **Ctrl+F** to search.
3. Click a glyph, or highlight it with arrows and press **Enter**.
4. Repeat to insert more; press **Esc** to return to Editor.

The details identify the highlighted glyph. Each insertion can be undone separately. Nerd glyphs need a suitable font. See [Character discovery](characters.md) for groups, search, and navigation.

## Clipboard and ANSI output

| Task | Action | What you get |
| --- | --- | --- |
| Copy selected text | Ctrl+C in Editor | Escaped ANSI preserving formatting |
| Cut selected text | Ctrl+X in Editor | Copy, followed by deletion after successful forwarding |
| Copy the whole design | Ctrl+C in Code | Complete escaped ANSI, regardless of scrolling or wrapping |
| Paste | Ctrl+V or terminal paste | Plain text or automatically detected raw/escaped ANSI |
| Select everything | Ctrl+A in Editor | Entire document selected |

Unsupported controls open a **Strip and paste / Abort** review. Cut and paste are undoable. Failed or stale clipboard operations preserve the document.

### Paste without styles or escape interpretation

These actions start **unbound**:

1. Open **Settings → Shortcuts → Workspace → Text Fields**.
2. Assign the action you need:

| Action | Use when… |
| --- | --- |
| Paste without formatting | You want ANSI text decoded using your current typing style |
| Paste literal text | You want notation such as `\x1b[31m` to remain literal text |

Over SSH, use your terminal’s paste command. For clipboard setup and forwarding limits, see [Troubleshooting](troubleshooting.md).

## Arrange the panes

| Task | Default control |
| --- | --- |
| Focus another pane | F6 or click it |
| Collapse / expand Code | F2 or its title indicator |
| Collapse / expand Editor | F5 or its title indicator |
| Toggle wrapping in the focused text pane | F3 or its Wrap button |
| Scroll | Wheel, scrollbar track, or draggable thumb |
| Change lower tabs on a narrow terminal | Alt+Left / Alt+Right, or click a tab |

- Collapsing a pane gives its space to the other; at least one stays expanded.
- At **64 columns or fewer**, Code and the tools share tabs in the lower pane.
- Wrapping and scrolling affect presentation only. Copied ANSI has no gutters or display wrap breaks.
- Code is read-only. Newlines appear as `\n`, tabs as `\t`, and literal backslashes are doubled.

## Movement reference

| Task | Shortcut |
| --- | --- |
| Move | Arrows |
| Move by word | Ctrl/Alt+Left / Right |
| Beginning / end of line | Home / End |
| Beginning / end of document | Ctrl+Home / Ctrl+End |
| Move a viewport | Page Up / Page Down |
| Extend selection | Add Shift to movement |
| Delete previous word | Ctrl/Alt+Backspace |
| Newline / indent | Enter / Tab |

**Editing notes:** tabs display at four-column stops; automatic indentation is not enabled. Multiline paste is one undo step. Consecutive typing/deletions are grouped until navigation, another operation, or a 750 ms pause. Up to 200 undo transactions are retained.

**Next:** [Save or export your design](files.md) · [Customize shortcuts](settings.md)
