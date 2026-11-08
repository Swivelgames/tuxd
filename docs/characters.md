# Find and insert characters

Use **Characters** for box drawing, symbols, emoji, lettering, Braille, and Nerd Font icons.
The catalog is bundled with tuxd; browsing and search work offline.

## Browse a group

1. Place the Editor cursor, or select text to replace.
2. Open **Characters** (the lower pane's Characters tab on narrow terminals).
3. Focus **Group** with Tab and press Enter to open its dropdown.
4. Choose a group with arrows, or type its name to jump, then press Enter. Types appear below as expanded sections.
5. Choose a glyph with arrows and press Enter. Clicking a glyph also inserts it.

Press Enter or click a type heading to collapse or expand it. All types start
expanded; your collapsed sections are remembered for the session.

Dropdowns support arrows, Page Up/Down, Home/End, mouse selection, and scrollbars.
Esc or a click outside cancels without changing the choice. Left/Right on a closed
selector still steps between choices.

Characters keeps its minimum height and expands into available vertical space.
Glyphs wrap to the available width with one space between them, including
double-width characters. The grid grows with the panel; its scrollbar, mouse wheel, Page Up/Down, and Home/End
browse the remaining results. Short terminals can scroll the whole tool block.

| Group | Includes |
| --- | --- |
| Box Drawing | Light, heavy, double, rounded, dashed, and diagonal lines |
| Blocks & Shading | Block elements, shades, and legacy-computing mosaics |
| Arrows | Standard and supplemental arrows and harpoons |
| Shapes | Geometric shape blocks |
| Symbols | Technical/interface symbols, checks, dingbats, music, games, and pictographs |
| Mathematics | Operators, relations, mathematical alphabets, superscripts/subscripts |
| Typography | Punctuation and currency |
| Letters & Numbers | Latin accents, Greek, Cyrillic, enclosed labels, fractions, and Roman numerals |
| Emoji | Faces/emotions, people, nature, food, travel, activities, objects, symbols, and standalone flags |
| Braille | Visible patterns for dot-based graphics |
| Nerd Icons | Icon families from Nerd Fonts |

## Search by name, glyph, or code point

1. While a Characters control is focused, press **Ctrl+F** (configurable in Settings → Shortcuts → Tools).
2. Type or paste a query into **Find**.
3. Press Enter to focus the results. Press Enter again to insert the selected glyph.

| Query | Finds |
| --- | --- |
| `box light horizontal` | Names containing all these words |
| `arrow` | Arrow names across groups |
| `fa home` | Nerd Font names and aliases |
| `─` | The literal glyph, ranked first |
| `U+2500` or `0x2500` | An exact Unicode code point |

- Search ignores case and treats hyphens/underscores as spaces.
- **All** beside Find searches globally. Activate it to switch to **Group**.
- Search results are grouped into type sections; global search headings also identify the group.
- **x** beside Find empties Find and restores the selected group.
- The single line below the grid identifies the selected result's code point and name.
- Find supports normal field navigation, Backspace/Delete, Select All, Copy, and Paste.

## Insert without losing your place

- Repeated activation inserts repeatedly using the current document style.
- The first insertion replaces an Editor selection, if present.
- Each insertion is a separate undo step.
- Esc returns to Editor. The query and browse choices remain available for the session.

## Which characters are included?

The catalog uses **Unicode 17.0.0** and **Nerd Fonts 3.4.0** metadata. It includes
a curated set for terminal interfaces and artwork, rather than every Unicode
character. Useful design families remain broad, including technical notation,
wide punctuation, game pieces, legacy mosaics, and named Nerd Font icons.

Excluded from the picker:

- Exhaustive writing-system catalogs, including CJK ideographs, historic scripts, and specialist notation outside the design collections.
- Terminal controls, separators/whitespace, blank Braille, unassigned code points, and unnamed private-use characters.
- Standalone combining marks, invisible formatting controls, and default-ignorable characters.
- Regional indicators, skin-tone components, and composed emoji sequences.
- Characters requiring tuxd's escaped or dotted-circle display fallback, including nine symbols with confirmed tmux cell-width conflicts.

These are discovery filters, not editing restrictions: you can still type or paste
text from any writing system into your document.

**Emoji** contains standalone, fully-qualified emoji from Unicode's
[Emoji 17.0 data](https://www.unicode.org/Public/17.0.0/emoji/emoji-test.txt),
in Unicode/CLDR emoji-picker order: faces and emotions, people, nature, food,
travel, activities, objects, symbols, then flags. Related emoji stay together
within each category; they are not sorted by code point. Joined emoji, skin-tone combinations, flag pairs,
and variation-selector sequences remain deferred.

Font coverage still depends on your terminal. A missing glyph may appear as a box;
Nerd Icons need a Nerd Font. tuxd cannot inspect the font used by a remote terminal.

**Related:** [Editing](editing.md) · [Settings](settings.md) · [Catalog maintenance](character-catalog.md)
