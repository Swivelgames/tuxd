# Character catalog maintenance

## Ownership

| Area | Responsibility |
| --- | --- |
| `libraries/characters` | Immutable metadata, group/subgroup catalog, stable scalar IDs, eligibility, deterministic search |
| `panel/characters` | Query field, browse selection, bounded result viewport, local input and clipboard completion |
| Shared controls | Field editing and grid movement |
| Panel | Tool layout, focus traversal, hit testing, scrollbars |
| Workspace / Editor | Apply insertion to the document, preserve styles/selection, record atomic undo |

The catalog has no App, file-dialog, clipboard, Settings, or Ratatui dependency.
Its only display dependency is the existing pure terminal-text library.
Aliases resolve to one scalar entry; they do not duplicate grid cells.
Only visible results become panel controls, even for searches covering large icon families.

## Data sources and regeneration

[Pinned sources and SHA-256 checksums](../licenses/characters/sources.json) identify
Unicode 17.0.0 UCD files, Emoji 17.0 `emoji-test.txt`, and Nerd Fonts 3.4.0 `glyphnames.json`.
The generated UTF-8 TSV is checked in and included in Cargo packages. Normal
builds and application startup never fetch data.

```sh
cargo xtask generate catalog
cargo test --locked
cargo xtask smoke characters
cargo xtask widths
```

The generator downloads only the pinned inputs and rejects checksum mismatches.
It reads Unicode names, aliases, categories, blocks, and default-ignorable
properties. Admission uses explicit design-block allowlists in the generator;
new Unicode blocks are not admitted automatically. DerivedName supplies official
scalar names. Emoji metadata supplies familiar category names, search aliases, and CLDR picker
order. An optional sixth TSV column records browse order; scalar-sorted storage
still supports lookup, while search preserves picker order after exact matches.
Nerd Font aliases are merged by scalar, with icon families as subgroups.
No font binaries or Unicode charts are redistributed.

| Filter stage | Responsibility |
| --- | --- |
| Generator | Admit curated design families and standalone fully-qualified emoji; exclude controls, marks, separators, blank Braille, unassigned scalars, default-ignorables, regional indicators and skin-tone components |
| Runtime admission | Require one or two cells and unchanged output from the existing conservative grapheme renderer |
| UI | Use only admitted catalog entries for browsing, search, and insertion |

When updating versions, review the source changes and licensing, update manifest
checksums, regenerate twice to confirm determinism, and run terminal checks.
Changes to `unicode-width` or the renderer may change the admitted set and require
review even when the data files are unchanged. The width probe compares actual
cursor reports in isolated Alacritty and tmux; it needs Alacritty, tmux, and Xvfb.
`--raw` bypasses display protection for diagnosis and can intentionally fail.

## Collection policy

- Preserve full box drawing, nonblank Braille, blocks, arrows, shapes, and useful technical families.
- Retain common lettering, enclosed labels, fractions, currency, music, games, and mathematical notation.
- Exclude exhaustive script inventories and specialized historic notation. This does not restrict document input or import.
- Merge related blocks into meaningful types instead of showing a section for every Unicode block.
- Keep Nerd Font families separate; admission cannot determine whether the user's terminal has their font.
- Emoji admits only single-scalar `fully-qualified` entries from the pinned test data. Components and composed sequences are not silently flattened.

When proposing additions, justify their terminal-design use and check rendering;
do not use assigned-code-point status alone as the admission rule.

## Dependency decision

| Candidate | Provides | Remaining work |
| --- | --- | --- |
| [unicode_names2](https://docs.rs/unicode_names2/latest/unicode_names2/) | Unicode name lookup and reverse lookup | Groups, categories, aliases/safety policy, Nerd metadata, discovery search |
| [ucd-parse](https://docs.rs/ucd-parse/latest/ucd_parse/) | Typed UCD file parsing | Generation policy, grouping, search, Nerd metadata |
| Pinned UCD + Nerd metadata | All required upstream facts in reproducible source files | Small maintainer-only Rust generator and application-specific search/grouping |

The last option avoids a runtime/build dependency for data that is generated only
when maintainers update the catalog. It reuses upstream names and properties,
existing Unicode width/grapheme libraries, and the existing field/grid controls.

## Validation and follow-ups

Checks cover unique ordered IDs, known exclusions, complete Box Drawing coverage,
name/alias/code-point search, scoped search, empty results, bounded grid layout,
keyboard navigation, repeated insertion, undo, narrow resizing, and terminal cleanup.

Future work remains separate: favorites/recents, custom catalogs, font detection,
and component/composed-emoji rendering. Group/type choices and queries are
session-local; they are not persisted in user configuration.
