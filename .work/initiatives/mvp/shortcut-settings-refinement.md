# Shortcut Settings — organization and consolidation

Status: Complete — implementation verified and accepted.

This refines [Slice 4a](slice-04a-shortcut-settings.md) following user testing of the first implementation. It supersedes that pass's flat section order and one-row-per-handler organization. The revised application has 104 actions, consolidating shared operations across concrete control scopes.

## Design contract

Settings should describe recognizable user actions. A shared operation is configured once and applied in each applicable context. Implementation-specific handlers, palette dimensions, and dialog names should not produce extra settings rows.

Keep presentation and applicability separate. An action listed under Workspace may apply inside a modal dialog; this does not make every workspace command active there. Editor actions can remain applicable while Tools or Code has focus where appropriate.

Keep the agreed three binding slots, explicit Save, fixed settings controls/openers, context jump, sequence capture, validation, and configuration failure behavior. Movement and selection remain separate actions.

## Top-level order and subsection inventory

Sequence Timeout is a standalone setting at the beginning, with a milliseconds field and its default (1000 ms). It is not a footer control or a subsection of Workspace. Context jump may open a later category; Timeout must have a direct, visible navigation destination from every category.

| Order | Category | Subsection | Contents |
| --- | --- | --- | --- |
| 0 | Sequence Timeout | None | Inter-key timeout, milliseconds |
| 1 | Workspace | None | Quit |
| 1 | Workspace | Files | Files menu, New, Open, Save, Save as, Import, Export |
| 1 | Workspace | Panes | Cycle panes, Focus Editor, Focus Code, Focus Tools; Previous/Next slim tab remain pane-specific candidates |
| 1 | Workspace | Field Actions | Activate, Submit |
| 1 | Workspace | Field Navigation | Next/Previous field; Left/Right/Up/Down; Page up/down, Beginning/End |
| 1 | Workspace | Text Fields | Backspace, Delete, Copy, Paste, Select all |
| 1 | Workspace | Range Controls | Decrease/Increase, Decrease/Increase by 10, Minimum/Maximum |
| 2 | Editor | None | Toggle Editor, Toggle wrapping |
| 2 | Editor | Tools | Toggle bold, Toggle italic, Toggle underline; other direct style actions only as implemented |
| 2 | Editor | Movement | Left, Right, Up, Down, Line beginning, Line end, Document beginning, Document end, Previous word, Next word, Page up, Page down |
| 2 | Editor | Selection | Select left/right/up/down, Select to line beginning/end, Select to document beginning/end, Select previous/next word, Select page up/down, Clear selection |
| 2 | Editor | Editing | Undo, Redo, Newline, Indent, Backspace, Delete, Delete previous word |
| 3 | Code | None | Toggle Code, Toggle wrapping |
| 3 | Code | Movement | Left, Right, Up, Down, Page up, Page down, Line beginning, Line end, Document beginning, Document end |
| 4 | Tools | RGB | Apply RGB if retained as a distinct tool command; ordinary field/range operations live in the shared Workspace subsections |
| 5 | Dialogs | General Actions | Positive action, Negative action, Abort, Close / dismiss |
| 5 | Dialogs | Content | Only genuinely review-specific commands left after shared navigation is extracted; omit if empty |
| 6 | File Explorer | Locations | Focus Location, Go to CWD, Parent directory, Enter directory |
| 6 | File Explorer | Navigation | Previous/Next entry, First/Last entry, Middle visible entry, Previous/Next page; preserve list-only Vim bindings |
| 6 | File Explorer | Display | Toggle hidden files, Toggle filter |

Workspace splits shared controls into Field Actions, Field Navigation, Text Fields, and Range Controls, in that order. Expanded headings have a blank row before their actions. Within Field Navigation, blank rows separate traversal, directions, and page/boundary navigation. Related action groups elsewhere use the same spacing; spacers cannot receive focus or mouse selection.

Do not manufacture empty tool subsections for future features. Generic character-grid navigation belongs with shared controls; a future character-search action can belong under Tools / Characters when implemented.

Copy is owned by Workspace / Text Fields and also applies to Code. Avoid a second configurable Copy row under Code. The selected action's applicability description explains that Code copies the entire escaped ANSI output.

### Naming rules

- Use subsection context to shorten labels: Movement / Left, not Movement / Move left.
- Keep Selection / Select left explicit and independently configurable.
- Use Beginning / End consistently; remove the Start / Finish wording from display labels.
- Keep Toggle for reversible on/off actions. Toggle Editor and Toggle Code mean expand/collapse their respective panes.
- Use Newline and Indent in Editor / Editing. Indent retains the existing literal-tab insertion behavior; block indentation would be an additional editing feature.
- Use Backspace and Delete consistently for the existing backward/forward deletion operations. Describe details in the selected action's explanation rather than in every row label.
- Pure label/category changes do not rename persisted action IDs.

## Shared behavior and scope boundaries

### Focus traversal

Next field / Previous field use Tab / Shift+Tab by default across tools, file forms, review views, menus, and prompt buttons. A composite radio group, tab group, or grid should be one focus stop, remembering its current item. Directional navigation operates inside it. Tab leaves the composite rather than visiting every swatch or glyph.

Editor Tab remains Indent. Code scrolling does not become field traversal. Shortcut Settings continues to use its own fixed controls.

This requires changing the current tools focus model, which treats every swatch, glyph, tab, and radio option as an independent item. Renaming ToolNext alone would not implement shared field traversal.

### Directional navigation

One shared Left / Right / Up / Down binding set applies to applicable non-document controls. The focused control supplies the behavior:

| Focused control | Directional behavior | Activation behavior |
| --- | --- | --- |
| Radio group | Select previous/next option along the group's axis | Selection changes immediately |
| Tab group | Select previous/next tab along the group's axis | Display the selected tab immediately |
| Color or character grid | Move the highlighted item spatially, preserving the column where possible | Enter/Space applies the swatch or inserts the glyph |
| Button group | Move focus between buttons | Enter/Space invokes the focused button |
| Menu | Move the highlighted item | Enter/Space invokes it |
| Scrollable review content | Scroll along the supported axis | Does not trigger Import merely by moving |
| Ordinary text field | Move the caret along supported directions | Text input and submission remain distinct |

Implemented behavior: grid navigation changes the highlighted item; it does not repeatedly insert glyphs or change document colors. Radio/tab navigation changes selection immediately. This distinguishes focus, selection, and document mutation while still sharing the bindings.

Page up/down and Beginning/End should likewise be shared across applicable controls, with viewport size and boundaries supplied by the widget. The indexed palette's seven-column/five-row geometry is not a configurable action family.

Keep Editor movement and Code scrolling independently configurable in their requested sections. File Explorer parent/enter-directory commands also retain their domain-specific identities and list-only Vim aliases. Do not propagate h/j/k/l or gg into text fields by merging their default bindings into shared field actions.

### Ranges

Increase / Decrease, Increase / Decrease by 10, Minimum, and Maximum belong under Workspace / Range Controls. RGB gauges consume them; future ranges reuse them. Preserve staging: moving an RGB range does not apply the color to the document.

Range adjustment and ordinary field navigation must have mutually exclusive applicability for the relevant directions. Left can be Decrease in a range and Left in a radio group without a conflict or priority-dependent dispatch. Rebinding Increase changes every supported range, not just one RGB channel.

### Clipboard and text fields

Copy, Paste, and Select all should each have one shared configuration wherever implemented. The focused target chooses the payload and operation. Code Copy retains its whole-output escaped ANSI behavior; text-field Copy uses the text selection. RGB Select all replaces the field's contents on subsequent typing, consistent with the path fields.

The present implementation only has clipboard Copy for Code and clipboard Paste for the file picker; it has Select all/replace for path and RGB fields. Sharing the identities is consolidation. Adding Copy to path/RGB fields or clipboard Paste to RGB is additional behavior and needs implementation and verification, not a rename.

Editor selection Copy/Cut/Paste with escaped ANSI remains the previously recorded follow-up feature. Its future implementation must consume the shared actions. Do not show Cut as functional, or claim Editor clipboard support exists, merely because an action name has been reserved. Bracketed terminal paste remains its existing separate input path.

### Activation versus submission

Share Activate across focused buttons, menus, choices, and grids. Share Submit across editable form fields where Enter confirms the field/form. Ordinary Space must continue to insert a space in text fields; do not silently install an Activate=Space alias there.

Do not preserve Raw, Escaped, and Toggle ANSI format as three dialog-specific actions. Shared radio navigation and activation should operate the selected group. RGB Apply can use form submission when a field is focused and ordinary activation when the Apply button is focused. Retain an extra Apply RGB shortcut only if it represents a useful direct command beyond these operations.

## Consolidation candidates grounded in the current registry

| Current action IDs/families | Proposed shared setting | Disposition / work required |
| --- | --- | --- |
| ToolNext/Previous, PickerNext/Previous, ReviewNext/Previous, Tab variants of DialogNext/Previous | Next field / Previous field | Consolidate; make composite controls one focus stop |
| ToolLeft/Right, ToolUp/Down, IndexedUp/Down, arrow variants of DialogNext/Previous, FormatRaw/Escaped | Left / Right / Up / Down | Consolidate where applicable; split traversal from direction before remapping defaults |
| ToolPageUp/Down, IndexedPageUp/Down, ReviewPageUp/Down | Page up / Page down | Consolidate; widget computes page size |
| ToolFirst/Last, ReviewFirst/Last | Beginning / End | Consolidate; widget supplies its boundary |
| FieldLeft/Right/Home/End | Shared field navigation | Fold into applicable directions/boundaries; preserve caret semantics |
| RgbDecrease/Increase, RgbDecrease10/Increase10, RgbMinimum/Maximum | Shared range adjustment | Move and generalize; retain six meaningful operations |
| ToolActivate, DialogActivate, FormatToggle, button/list parts of PickerActivate | Activate | Consolidate non-text targets; explicitly define radio toggling and tab/grid activation |
| Text-field parts of PickerActivate, RgbApply | Submit | Consolidate form submission; separate from Activate=Space |
| FieldDeleteBefore, RgbDelete | Backspace | Consolidate text-field deletion; RGB currently deletes only from its end |
| FieldDeleteAfter | Delete | Move under shared text-field editing; only install where implemented |
| FieldSelectAll, RgbSelect | Select all | Consolidate field selection/replacement |
| CopyCode | Copy | Promote to shared identity; keep Code payload unchanged; new text-field consumers require implementation |
| PickerPaste | Paste | Promote to shared identity; preserve asynchronous clipboard reads; add consumers deliberately |
| NextTab/PreviousTab | Next / Previous slim tab | Keep under Workspace / Panes initially: these act from anywhere in the lower pane, unlike navigation inside a focused tab group |
| EditorWrap, CodeWrap | Toggle wrapping in each section | Keep separate, allowing independent bindings as previously agreed |
| Editor movement and selection | Separate Movement and Selection subsections | Keep independent actions and slots; do not derive selection by adding Shift |
| ClearSelection | Clear selection | Audit its existing return-to-Editor side effect; preserve or explicitly separate that behavior rather than hiding it behind a shorter label |
| Code movement | Code / Movement | Keep independently configurable; shorten labels |
| ExplorerParent/Enter and list navigation | File Explorer subsections | Keep domain-specific actions and Vim defaults local to the list |
| Positive, Negative, Abort, Close | Dialogs / General Actions | Retain semantic mappings; share across applicable prompts |

The desired reduction is fewer independent decisions for users. Do not set an arbitrary target action count or merge distinct user intents merely to reduce rows.

## Registry changes needed

1. Add ordered category, optional subsection, and explicit display-order metadata. Declaration order must not dictate settings order or interleave movement and selection.
2. Replace the old Files/Panes top-level categories with Workspace; move Editor and Code toggles to their respective sections. Sequence Timeout is a separate setting, not an ActionId.
3. Give shared actions concrete applicability and context-specific execution. Dispatch still resolves an action before application code performs its focused-control behavior.
4. Introduce enough focus detail to distinguish button, radio, tab, grid, range, text field, and scrollable content. The current broad Tools/Indexed/Hex/Channel scopes cannot express all these distinctions alone.
5. Resolve effective applicability before validation and which-key map construction. A disabled or inapplicable binding must not create an artificial conflict. Two different active actions with the same binding remain an error; do not add silent last-wins precedence.
6. Keep three slots per shared action. Do not create a configurable row per consumer or add per-widget overrides as a workaround for consolidation.
7. Subsections are presentation metadata. Which-key scopes still represent input applicability, categories describe settings organization, and groups still describe sequence prefixes. Do not use which-key prefix groups for UI subsections.
8. Preserve IDs for pure presentation changes. Actual merges need an explicit configuration transition: preserve unique custom bindings when they fit and validate; report incompatible or over-three-slot combinations without silently dropping bindings or overwriting the saved file. Decide the transition format before shipping merged IDs. No user-facing migration announcement is needed.

## Agreed settings layout

The user approved the category navigator and selected-action binding editor mockups. The following layout is implemented:

- Show the requested category order in a compact navigator and one category's contents at a time. At narrow widths, use a category selector rather than stacking every category into one enormous document.
- Give Sequence Timeout its own first destination and a labeled milliseconds field. Keep Save and Cancel readily accessible; timeout no longer lives in that footer.
- Distinguish category titles, subsection headings, action labels, bindings, and selected controls with spacing and restrained emphasis. Avoid full-height background stripes across binding columns.
- Fit action labels without silently truncating them. Keep the three binding slots recognizable, but avoid repeating three equally prominent Reset buttons on every inactive row.
- Show compact binding summaries in the action list. Selecting an action reveals a detail area below it with its description, applicability, and Primary / Alternate 1 / Alternate 2 binding fields.
- Show Reset binding and Unset binding once in that detail area; they operate on the selected binding slot. Both remain keyboard- and mouse-accessible.
- Start subsections expanded in every category and allow independent collapsing. Context jump selects the relevant category and expands the related subsection.
- At wide widths, use a left category navigator and a right content pane, with the binding detail area below the action list. At narrow widths, replace the left navigator with a category selector and stack the three binding fields vertically.
- Keep Reset all, Save, and Cancel in a persistent footer. Enter on a binding records one key; number keys retain the agreed sequence-capture behavior.
- Show the selected action's applicability and concise explanation in one detail area, including differences such as Copy in Code versus Copy in a field. Do not repeat this explanation in every row.
- Context jump selects the related category/subsection. Shared operations are shown once in the applicable Workspace subsection; invoking context does not duplicate their settings.
- Preserve discoverable capture, unset, reset, validation feedback, draft Save, and discard behavior while changing presentation.

## Implementation and verification order

1. Reorganize catalogue presentation and labels, adding subsections without changing bindings or saved IDs. Verify every existing action has a destination and settings order is independent of declaration order.
2. Consolidate shared focus/navigation, activation/submission, ranges, and clipboard identities. Implement the needed control scopes and composite focus behavior. Resolve configuration transitions and verify conflicts against effective applicability.
3. Build the revised settings layout against the resulting catalogue and review wide/slim renderings.
4. Verify a single rebind propagates across consumers: Next field in tools/picker/review/prompts; directions in radios/tabs/grids; ranges across RGB channels; Copy/Paste/Select all wherever supported. Preserve editor Indent and independent selection bindings.
5. Repeat persistence, capture, failure, context-return, and Linux PTY checks. Confirm grid movement never inserts a glyph until activation, modal commands remain scoped, and no path-field letter bindings consume normal text.

## Implementation evidence

- Registry metadata orders categories and subsections independently of action declarations. Movement and Selection are separate; Code and Editor own their pane toggles.
- Shared Fields actions replace tool/picker/prompt/review duplicates. Composite groups remember their item; grid movement highlights without mutating the document. The 256 grid retains its seven-column/five-row behavior.
- Settings uses the category navigator/selector and one selected-action detail editor, with fixed controls and a persistent footer. The action list and navigator scroll independently; small layouts keep Save/Cancel available.
- Configuration version 2 retains stable IDs where possible. The version 1 adapter preserves compatible custom bindings, reports incompatible merges, and never rewrites a file during load.
- Clipboard Copy now supports selected path/RGB field text; asynchronous Paste supports RGB fields. Editor ANSI selection clipboard remains deferred.
- Verification: 93 Rust tests, formatting and Clippy checks, and the Linux PTY suite. Coverage includes shared rebindings across controls, grid activation, context jump/collapse, wide/narrow/tiny resizing, capture and conflict recovery, Save/restart, legacy conversion failures, and isolated clipboard helpers.

Manual review: run `cargo run --locked`, open F1, use Ctrl+L to switch categories, expand a subsection, and select an action. Enter focuses its bindings; Enter again records a key. Try an alternate sequence, Reset binding, Unset binding, Cancel/Keep editing, and Save. Resize below 90 columns to inspect the selector and stacked binding fields. Tab through Tools to try the shared group navigation.
