# Configuration reference

[Documentation](README.md) · [Settings walkthrough](settings.md)

## Locations

| File | Purpose |
| --- | --- |
| `$XDG_CONFIG_HOME/tuxd/shortcuts.json` | Bindings, leader, and sequence timeout |
| `$XDG_CONFIG_HOME/tuxd/theme.json` | Theme preset and color overrides |

If `XDG_CONFIG_HOME` is unset or empty, use `~/.config/tuxd/`. A configured `XDG_CONFIG_HOME` must be absolute.

| Situation | Behavior |
| --- | --- |
| Missing file | Use defaults |
| Invalid or unreadable file | Report the error and use defaults; leave the file untouched |
| External edit | Restart to load it |
| Save in Settings | Validate, persist, then activate both drafts |
| Save failure | Keep Settings open with the draft and previous active bindings |

Files are written individually. If saving shortcuts fails after writing the theme, tuxd attempts to restore the previous theme file; this is not a crash-atomic two-file transaction.

## Theme example

```json
{
	"version": 1,
	"preset": "midnight",
	"colors": {
		"accent": "#80c0ff",
		"border": "ansi:60",
		"surface": "default"
	}
}
```

| Value | Meaning |
| --- | --- |
| `default` | Terminal-default color |
| `ansi:0` … `ansi:255` | Indexed terminal color |
| `#RRGGBB` | RGB color |

Omitted roles inherit the preset. Unknown versions, presets, roles, or invalid colors reject the file and fall back to Midnight.

### Color roles

| Area | Roles |
| --- | --- |
| Interface | `accent`, `surface`, `text`, `border`, `muted` |
| Controls | `field`, `selected`, `selected_text`, `focus`, `inactive_indicator`, `range_text` |
| Editor and feedback | `error`, `selection_overlay` |
| RGB controls | `rgb_red`, `rgb_green`, `rgb_blue` |
| Code syntax | `syntax_sequence`, `syntax_parameter`, `syntax_escape`, `syntax_separator`, `syntax_whitespace` |

### Presets

[Preview every preset in the theme gallery](themes.md).

| Family | Variants |
| --- | --- |
| tuxd | Midnight (default), Light |
| One Dark / Dracula / Nord | One preset each |
| Gruvbox / Solarized | Dark, Light |
| Tokyo Night | Night |
| Catppuccin | Latte, Frappé, Macchiato, Mocha |

See [palette sources](../.work/initiatives/themes/presets.md) for upstream attribution.

## Shortcut files

Use Settings to record keys rather than constructing their serialized representation manually.

| Property | Contract |
| --- | --- |
| Version | `2` |
| Action identity | Stable action IDs |
| Binding slots | Primary, Alternate 1, Alternate 2; `null` means explicitly unset |
| Missing known action | Inherit its defaults |
| Invalid content | Unknown actions/fields/versions, invalid keys, or conflicts reject the whole file |
| Older version 1 | Converted in memory when compatible; explicit Save writes version 2 |
| Loading | Never rewrites the source file |

### Binding restrictions

- Conflicts are checked in overlapping contexts, including resolved leader sequences.
- An action cannot also be a prefix of another binding in the same applicable context.
- Bare printable bindings/prefixes are prohibited where they would consume editable text.
- Settings openers and its own navigation remain fixed.
- Terminals may report physically different keys as the same input.

See [recording sequences](settings.md#record-a-key-sequence) for the user-facing workflow.
