# Document and ANSI format reference

[Documentation](README.md) · [Save, import, and export](files.md)

## Native document: `.tuxd.json`

| Field | Meaning |
| --- | --- |
| `version` | Format version; currently `1` |
| `text` | Original Unicode text |
| `styles.runs` | Contiguous style ranges covering the text |
| Run `start` / `end` | UTF-8 byte offsets at grapheme boundaries |
| Run `style` | Five attribute booleans and foreground/background colors |

| Color representation | Example |
| --- | --- |
| Terminal default | `"Default"` |
| Basic ANSI | `{"Ansi": 0}` |
| Indexed | `{"Indexed": 42}` |
| RGB | `{"Rgb": [128, 192, 255]}` |

See a complete [example document](../examples/showcase.tuxd.json).

- Loading rejects unknown versions/fields, invalid ranges/colors, and unsupported stored controls.
- File reads require regular files no larger than **16 MiB**.
- Writes publish through a temporary file in the destination directory and retain existing file permissions.
- Symlink and non-regular write targets are rejected.

## Raw versus escaped ANSI

| Property | Raw | Escaped |
| --- | --- | --- |
| ANSI introducer | Actual ESC byte | Literal `\x1b` |
| Newline | Actual newline | `\n` |
| Tab | Actual tab | `\t` |
| Literal backslash | Backslash | `\\` |
| Export reset | Final style reset | Encoded style reset |

Escaped import also preserves literal Unicode and actual line breaks. CRLF is normalized to LF. Code adds display line breaks for readability; copied/exported escaped output excludes them.

## Supported SGR controls

| Effect | Parameters |
| --- | --- |
| Reset all | `0` |
| Bold / dim / italic / underline / strike-through | `1` / `2` / `3` / `4` / `9` |
| Clear bold and dim | `22` |
| Clear italic / underline / strike-through | `23` / `24` / `29` |
| Basic / bright colors | ANSI foreground and background color parameters |
| Default foreground / background | `39` / `49` |
| Indexed foreground / background | `38;5;n` / `48;5;n` |
| RGB foreground / background | `38;2;r;g;b` / `48;2;r;g;b` |

If a style boundary splits a grapheme, its leading character’s style wins.

## Unsupported-control handling

| Input | Handling |
| --- | --- |
| Unsupported or malformed control with a recoverable boundary | Review, then strip or abort |
| SGR sequence containing unsupported parameters | Strip the whole sequence; preceding style stays active |
| Unterminated OSC/DCS/control string | Reject when its end cannot be determined safely |
| Terminal cursor/screen operations | Not reconstructed as a terminal screen |

Import previews are inert: imported controls are not executed by the terminal. See the [import walkthrough](files.md#import-existing-ansi).
