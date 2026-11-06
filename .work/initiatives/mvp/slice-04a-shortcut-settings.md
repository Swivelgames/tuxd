# Slice 4a — contextual shortcuts and settings

Status: Complete — refined implementation verified and accepted

Depends on: Slice 4 and the collapsible/slim layout refinement (`bd953c6`). Runs before Slice 5; existing slice numbers remain unchanged.

Estimate: 4–6 developer days including migration, settings UI, persistence, and integration verification. This is a planning range, not a deadline.

## Usable checkpoint

Open Shortcut Settings from any workspace or dialog context, jump to that context's section, capture a single key or sequence, resolve conflicts, save, and immediately use the new shortcut. Restart and retain the configuration. Canceling settings preserves the active configuration and underlying document/dialog state.

Every migrated action remains usable through its defaults. This slice is complete only when routing, configuration, and the settings UI work together; an isolated settings mockup is not a usable checkpoint.

## Agreed decisions

- Three independently unsettable slots per action: Primary, Alternate 1, Alternate 2.
- A stable action/context registry supplies dispatch, settings labels, defaults, validation, and selected inline hints.
- Keep shortcut hints restrained. The File menu shows its assigned shortcut inline. Do not introduce a pervasive shortcut-hints panel. Remove the file picker's shortcut panel.
- Replace Help with Shortcut Settings. F1, Ctrl+/, and Ctrl+, open it, are hard-coded, and cannot be assigned to another action or used as a sequence prefix. Ctrl+? is not an explicit alias. Indistinguishable terminal events cannot be distinguished by the application.
- Opening settings remembers the invoking context, jumps to its section, and preserves the underlying screen, focus, text selection, and pending dialog workflow.
- Settings navigation, capture cancellation, activation, Reset, Save, and Cancel are fixed and never read from user bindings.
- Fixed settings navigation includes arrows, h/j/k/l, gg, G, and M; Esc closes/cancels the current settings operation, Enter activates the focused binding/reset/button. Any additional settings navigation controls must also remain fixed.
- Changes are drafts until explicit Save. Save validates the entire draft, persists it, then activates it. A failed save keeps the active configuration unchanged and retains the draft for correction/retry.
- Slot Reset restores that slot's default in the draft. Reset all restores all defaults in the draft. Both require Save to take effect.
- Esc closes unchanged settings. With draft changes, offer Discard / Keep editing, initially focused on Keep editing.
- Loading missing configuration uses defaults. Invalid/unreadable configuration uses defaults and reports the problem. Never silently overwrite a broken configuration during load.
- No migration notices or breaking-change disclaimers are needed. Update actual inline hints to match saved bindings.

## Contexts and action grouping

The [settings organization and consolidation plan](shortcut-settings-refinement.md) supersedes the first implementation's flat grouping. Its ordered hierarchy is: standalone Sequence Timeout, Workspace, Editor, Code, Tools, Dialogs, File Explorer. Workspace contains Files, Panes, Field Actions, Field Navigation, Text Fields, and Range Controls; Editor separates Tools, Movement, Selection, and Editing. Toggle Editor belongs in Editor and Toggle Code belongs in Code. Both categories show Toggle wrapping at the top level.

Shared field traversal, control navigation, range adjustment, and clipboard/selection actions should be configured once across applicable controls. The plan inventories existing actions and distinguishes actual consolidation from new clipboard behavior. Editor movement and selection remain independent; Editor and Code wrapping remain independently configurable. Subsections organize presentation and do not replace runtime input scopes or which-key sequence-prefix groups.

The hierarchy, labeling direction, category navigator, collapsible subsections, and selected-action binding editor are accepted refinements. The consolidation table records the implemented shared controls. The refined navigator and binding editor are now available for manual review.

## Registry and library integration

Agreed architecture:

- The action catalogue owns stable action IDs, labels, settings sections, three default binding slots, and applicability. Dispatch returns an action ID; application code executes it.
- Concrete input contexts map to `ratatui-which-key` scopes. Explorer List, Location Field, and Filename Field are distinct scopes so explorer letter bindings never consume path input.
- Settings sections map to library categories. Category controls presentation, not applicability: Toggle Bold is listed once under Editor but is installed in Editor, Code, and Tools scopes as applicable.
- The application expands shared applicability into concrete scopes when assembling the keymap. Do not implement a competing sequence matcher or assume automatic scope inheritance.
- Library groups describe shared sequence prefixes, such as Ctrl+W followed by Left/Right. Derive prefix descriptions from effective bindings; do not permanently attach pane-focus semantics to a prefix the user may reassign.
- Use a keymap shaped as `Keymap<ShortcutKey, InputScope, ActionId, SettingsSection>`. The serializable application key adapter implements the library's `Key` trait and retains decoded code/modifier distinctions. Preserve the original event for capture feedback; define normalization explicitly and verify BackTab, case/Shift, combined modifiers, and Unicode before migrating handlers.
- Fixed settings openers are handled before configurable dispatch. Library-global bindings are not a blanket mechanism for workspace actions that must be suppressed inside dialogs.
- Movement and selection are separate actions with independent binding slots, e.g. Move Word Left and Select Word Left. They may share execution code but selection is not implicitly derived from a movement binding plus Shift.
- Input routing is: fixed controls/openers, adapter, resolved scope, library prefix/leaf lookup, action execution or text fallback. A failed pending sequence cancels according to the agreed policy rather than falling through into text.
- Saving settings builds and validates a complete replacement configuration/keymap, persists it, then swaps it into active use. Never mutate the active keymap incrementally while editing the draft.

## Shared dialog actions

| Action | Default slots |
| --- | --- |
| Next field | Tab, unset, unset |
| Previous field | Shift+Tab, unset, unset |
| Activate focused button/item | Enter; non-text aliases scoped as described in the refinement plan |
| Positive | S, Y, unset |
| Negative | D, N, unset |
| Abort | A, C, unset |
| Close | Esc, unset, unset |

The refinement separates field traversal from directional navigation, replacing the previous combined Tab/Right/Down and Shift+Tab/Left/Up actions. Shared directions provide arrow navigation in applicable button groups and menus. This is implemented through the shared Fields actions.

Semantic response mappings remain:

| Dialog | Positive | Negative | Abort / Close |
| --- | --- | --- | --- |
| Unsaved document | Save | Discard | Cancel |
| Overwrite | Overwrite | Cancel | Cancel |
| Unsupported ANSI review | Import anyway | Abort | Abort |

For a two-choice Positive+Abort or Positive+Negative dialog, Negative and Abort both select the secondary choice. For prompts, Abort and Close both dismiss without action. Menus and scrollable content apply navigation to the appropriate focused area rather than treating all arrows as button movement.

These are explicit semantic mappings, not rules based on button position or label guessing. No change notice is shown for revised aliases.

## Capture and sequence execution

- Enter on a binding captures one decoded key press.
- A number key can start capture for that many successive inputs, e.g. 2 then g g. Show the expected count, received events, and progress.
- Ctrl+W Right and Ctrl+W Left are valid example sequences for focusing Tools and Editor.
- Esc cancels capture; release/repeat events must not fill capture slots accidentally. Display what the terminal actually reports; do not invent distinguishable physical keys from identical events.
- Execute complete bindings immediately. Wait only while the received keys are a proper prefix of one or more bindings.
- Reject a complete binding that is a prefix of another binding in an overlapping context, in either assignment order. Thus Ctrl+W cannot coexist with Ctrl+W Right there.
- Default inter-key timeout: 1000 ms. The timeout is configurable in milliseconds in settings, validated as a positive supported duration, and follows the same draft/Save contract.
- Esc cancels pending sequences. Expiration or an unmatched continuation cancels the pending sequence without inserting its buffered input into text.
- Focus/context changes, dialog open/close, and terminal focus loss cancel pending sequences.
- A pending-prefix status indication may be shown while waiting; do not add persistent shortcut clutter.

## Validation

- Compare normalized decoded key events, including modifiers, across all three slots and inherited/applicable contexts.
- Reject duplicate competing bindings and ambiguous prefixes in overlapping contexts. Report both actions, contexts, and slots so users can resolve the conflict.
- Allow reuse in mutually exclusive contexts: Tab in Editor vs dialog navigation; F3 for Editor vs Code wrap.
- Block reserved settings-opening shortcuts and other assignments that would intercept the fixed escape route to settings.
- Protect editable contexts, including Editor, path fields, and RGB fields. Reject bare printable bindings or printable sequence prefixes that consume ordinary typing. Shifted printable characters are still printable. Explicit insertion actions such as Insert Tab have narrow exceptions.
- Keep text-protection and allowed defaults context/action specific. File Explorer can retain h/j/k/l, gg/G/M without preventing those characters in path fields.
- Settings' fixed keys are independent of configurable keys. Those letters may remain assignable in other contexts where ordinary text is not being edited.
- Terminal compatibility aliases such as legacy Ctrl+/ decoding must participate in reserved-key/conflict validation without pretending to represent separate physical shortcuts.

## Persistence

Use a versioned application configuration at `$XDG_CONFIG_HOME/tuxd/shortcuts.json`, falling back to `~/.config/tuxd/shortcuts.json`. Keep it independent of `.tuxd.json` documents.

Store stable action IDs, the three slots (including explicit unset values), key sequences, and the timeout. Use tab-indented JSON and atomic writes. Validate before persistence and activation; do not partially apply a malformed configuration. Unknown action IDs, unknown fields, and unsupported versions reject the whole configuration. Missing known actions inherit defaults; explicit null slots remain unset. The loader caps the configuration at 1 MiB and accepts regular files. Sequence capture supports 1–9 inputs and the timeout range is 1–60000 ms.

## Implementation approach and dependency assessment

Selected: **`ratatui-which-key` 0.14.0**, following the user decision after the [focused library assessment](shortcut-library-assessment.md). The dependency is pinned and build-checked on `feat/4a-shortcut-settings`.

Reuse its scoped/global sequence tree and prefix/leaf lookup. Supply a custom key adapter preserving decoded event/modifier distinctions, and own the configurable timeout/cancellation policy around lookup. Keep the action catalogue, overlap/text-protection validation, three-slot settings draft, and XDG persistence in the application. The which-key popup is optional and does not replace the agreed editable settings UI or introduce persistent hints. `keybinds` remains a researched alternative, not an additional production dependency.

Suggested implementation order (internal steps, not separately complete slices):

1. Establish action IDs, context applicability, three-slot defaults, normalization, and inventory parity.
2. Route existing input through the registry; implement prefix matching/cancellation, timeout, and conflict/text-protection validation.
3. Implement settings navigation, context jump, capture, reset/unset, timeout editing, and draft Save/Cancel behavior.
4. Add XDG persistence, replace Help/picker hints, update inline File-menu hints, and run integrated acceptance checks.

## Acceptance and verification

1. Open settings with every fixed opener from Editor, Code, Tools, File Explorer, and a confirmation prompt. Verify context jump and exact return to the underlying workflow.
2. Change an action's primary binding, use both alternate slots, unset a slot, cancel once, then Save and verify dispatch and inline menu hints.
3. Capture gg and Ctrl+W Right/Left. Confirm immediate single-key actions, prefix waiting, configured timeout, mismatch cancellation, and context-change cancellation.
4. Reject exact conflicts and prefixes in either order; allow context-separated Tab bindings. Reject printable Editor/path/RGB prefixes while allowing Explorer navigation and Insert Tab exceptions.
5. Rebind configurable dialog controls and verify all initial semantic mappings, including two-choice fallback behavior.
6. Exercise Reset slot, Reset all, unsaved-settings Discard/Keep editing, failed save, missing configuration, malformed configuration, and restart persistence using an isolated XDG directory.
7. Show the terminal's actual decoded key during capture. Confirm fixed settings openers and fixed settings navigation remain available regardless of draft or loaded bindings.
8. Resize settings across wide/slim/tiny layouts and ensure keyboard/mouse navigation and scrolling remain usable. Preserve document/history, pending import review, and terminal restoration.
9. Run formatting, tests, Clippy, and updated Linux PTY acceptance checks. Add focused regressions for routing, conflicts, and configuration failure boundaries.

## First implementation evidence

The following records the first pass. Passing its automated checks did not establish acceptable settings usability; the refinement above was subsequently implemented; see its implementation evidence.

Implemented on `feat/4a-shortcut-settings`, after planning/dependency commit `39f43e2` was pushed to origin.

- `src/shortcuts/registry.rs`: 130 named actions, scopes, categories, defaults, and shared applicability. `key.rs` provides the decoded key adapter; `mod.rs` provides which-key lookup, validation, cancellation, and XDG persistence.
- Workspace, Editor, Code, tool controls, RGB fields/gauges, explorer list/path fields, ANSI radios, menus, and prompts dispatch named actions. Ordinary printable input and bracketed paste remain separate.
- `src/shortcuts/settings.rs`: draft editing, all three slots, capture, reset/unset, timeout, fixed navigation, persistent Save/Reset all/Cancel controls, context jump, mouse scrolling, and discard confirmation. Wide layouts show the three slots across each row; narrower layouts stack them below the action.
- Help and the picker shortcuts panel are replaced. File-menu and workspace hints use the active configuration. Dialog response aliases follow the agreed explicit mapping.
- Automated verification: 86 Rust tests; `cargo fmt --all --check`; `cargo clippy --locked --all-targets -- -D warnings`; the Linux PTY smoke suite, including capture/Save/restart, sequence dispatch, prefix-conflict recovery, context return, resize, malformed config, and terminal restoration. Test sessions isolate XDG configuration and clipboard helpers.

## Refined implementation evidence

The [refinement record](shortcut-settings-refinement.md#implementation-evidence) documents the current 104-action catalogue, shared control behavior, version 1 configuration conversion, and redesigned settings UI. Automated verification now covers 93 Rust tests plus the Linux PTY suite, formatting, and Clippy. The refined layout was accepted after manual review. Subsections start expanded by default; users can collapse them individually. Preferred default bindings can be updated after the planned user keymap walkthrough.

Launch with `cargo run --locked`. Manual walkthrough:

1. Open F1 and use Ctrl+L to choose Code. Select Toggle Code, press Enter to focus Primary, then Enter to record F8. Cancel once, keep editing, then Save. Verify F8 toggles Code and F2 no longer does; Reset binding and Save restores F2.
2. In Workspace / Panes, capture Focus Tools as Ctrl+W Right and Focus Editor as Ctrl+W Left using count 2. Try binding Ctrl+W alone elsewhere and inspect the conflict. Edit Sequence Timeout and test expiration, mismatched continuation, and Esc cancellation.
3. Open settings from Code, a tool grid, an RGB range, an explorer list/path field, and a dirty prompt. Confirm the context jump and return to the underlying workflow.
4. Rebind Next field once and try it in Tools, file dialogs, import review, and prompts. Verify grid arrows move the highlight without applying; Enter/Space activates. Editor Tab remains Indent.
5. Try all three slots, unset/reset, Reset all, Save, and dirty-settings Discard / Keep editing. Restart and verify persistence and inline File-menu hints.
6. Resize across wide/slim layouts. Navigate categories, collapse subsections, scroll with the mouse/scrollbar, and confirm the footer stays accessible.

Limits: physical keys sharing a terminal encoding cannot be distinguished. Capture is limited to nine decoded presses; timeout is bounded to 60 seconds. Proper Vim editing and broader terminal compatibility remain future work.

## Follow-up: leader and General settings

The [shortcut leader enhancement](../../initial-concept.md#future-work-shortcut-leader-and-general-settings) records the requested optional `<leader>` prefix and capture-count behavior. A future General category will contain Leader and Sequence Timeout. This follow-up does not reopen the completed slice or change the current shortcut defaults.
