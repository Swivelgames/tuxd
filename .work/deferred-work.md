# Deferred work overview

Status: Backlog after accepted MVP, ANSI highlighting, and editor clipboard editing

This is the consolidated current backlog. Older slice documents describe what was deferred at their own checkpoints; features delivered by later slices are not outstanding work. The [initial concept](initial-concept.md) retains the detailed product requests.

## Requested product follow-ups

| Area | Remaining work | Decisions or prerequisites |
| --- | --- | --- |
| Named custom themes | Store themes under `$XDG_CONFIG_HOME/tuxd/themes/` (fallback `~/.config/tuxd/themes/`); create and name themes in Settings | Theme file schema, naming/collision behavior, discovery and selection |
| Shortcut defaults | Adopt the user's preferred defaults after their keymap walkthrough | User supplies reviewed bindings |
| Component emoji | Render composed emoji/components reliably instead of display escapes | Shared terminal/multiplexer width strategy; ZWJ, modifiers, flags, selectors, keycaps; redraw/navigation regressions |
| Character catalogs | More curated glyphs; search by glyph, name and code point | Catalog coverage, metadata and search UI |
| Style-only clipboard | Copy/paste formatting without text, targeting selection or pending typing style, with undo | Representation, mixed selection behavior and clipboard compatibility |
| Workspaces | Related files, left explorer and multiple open document tabs | Per-document undo/dirty state, switching, save/close rules and workspace metadata |
| Proper Vim mode | Modal editing, composable operators/motions, counts and coherent undo | Registers, visual selection, repeat/search and compatibility contract |
| ANSI explanations | Readable descriptions of sequences and their styling effect | Interaction/accessibility without obscuring Code output |
| Code generation | Explore languages, runtimes, frameworks and output forms, then choose initial targets | Separate [discovery initiative](initiatives/code-generation/overview.md); no target language committed |

## Recorded scope exclusions and improvement candidates

These appear in limits or scope notes. They are not automatically approved implementation slices.

| Area | Excluded or limited capability |
| --- | --- |
| Persistence reliability | Autosave, crash recovery, file watching and external-change conflict detection |
| File navigation | Recent-file menus, directory creation and remote browsing |
| Selection | Rectangular selection; dragging currently clamps at viewport edges without autoscroll |
| RGB controls | Additional visual color pickers beyond the implemented hex fields and interactive RGB gauges |
| Editing convenience | Automatic indentation beyond the current literal-tab insertion behavior |
| Large documents | Large-file engineering; current target is modest documents, synchronous file I/O, bounded history and explicit size limits |
| Extended layout | Further advanced layout refinements are unscoped; collapsible/maximized panes and the slim tab layout already exist |
| Output editing | Bidirectional editing of generated ANSI/code |
| Terminal screen operations | Arbitrary terminal-screen reconstruction/import and cursor-control authoring |
| Compatibility | Support beyond the tested Linux terminal matrix; universal glyph/style/width parity is not promised |

Application-managed font selection/installation and guaranteed terminal capabilities are platform constraints, not scheduled features. Remote clipboard querying is not provided; SSH users currently paste through their terminal. No remote-read implementation has been committed to.

## Completed work removed from the backlog

- Shared Settings with Midnight-default UI themes and an expanded preset catalog, per-role customization, live preview, independent reset, and persistence.

- Optional shortcut leader and General settings, including staged conversion/unset prompts and resolved conflict validation.

- Explicit raw/escaped CLI imports from files or stdin (`--raw -`, `--escaped -`), with control review and terminal handoff.

- ANSI syntax highlighting, including wrapped/scrolled Code output.
- Editor styled-selection copy/cut and automatic raw/escaped ANSI paste.
- Paste without formatting and Paste literal text (configurable, initially unbound).
- Clipboard control review, safe asynchronous cut/paste targeting and undo behavior.
- Raw/Escaped ANSI file import/export, project persistence and file browsing.
- Collapsible panes, slim layout, wrapping, gutters, scrollbars and interactive RGB gauges.

## Suggested next selection

Character discovery would be a self-contained product improvement. Component-emoji rendering and code generation each require more investigation before implementation scope can be estimated confidently. This is sequencing advice, not an approved order.
