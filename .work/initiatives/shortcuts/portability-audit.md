# Shortcut portability audit

Status: inventory and proposed direction; defaults have not changed.
Audited 2026-11-10 against `62e3a4c` on `feat/directory-workspaces`.

## Disposition after user testing

The user reports that their Alacritty setup required manual user-configuration
edits for extended keys to work. Record this as evidence about that setup, not
a claim that every Alacritty version lacks protocol support. **tuxd's default
workflows must not depend on users modifying their terminal configuration.**

The user has deferred the key diagnostic view, default-binding redesign and
related fixes until after the directory improvements ship. The inventory,
candidate alternatives and confirmed leader-prompt Shift+Tab defect below are
retained for that follow-up. They are not additional directory-slice requirements.
Existing mouse/menu access remains available; modified shortcuts are a known
portability limitation of the current defaults.

## Findings

- **Plain `o` and `O`, and `e` and `E`, remain distinct.** The adapter keeps
	character case, folding a printable Shift modifier into the character. The
	new regression test covers both explicit-Shift and uppercase-only input.
- **Ctrl+Shift+letters are unsuitable as the only default binding.** A legacy
	terminal can send the same byte for Ctrl+O and Ctrl+Shift+O. Once those bytes
	are identical, neither Crossterm nor tuxd can recover the original gesture.
- **Shift+navigation is a different family.** Modified arrows and BackTab have
	distinct encodings in many terminals. Keep useful selection shortcuts, with
	fallbacks for limited console/keymap configurations.
- **Interception differs from ambiguous encoding.** A host editor, terminal,
	multiplexer or desktop can consume a perfectly representable binding before
	tuxd receives anything.
- tuxd already requests `DISAMBIGUATE_ESCAPE_CODES` on terminal entry. Sending
	the request does not prove that the terminal/multiplexer honored it. Current
	configuration validation compares decoded keys, not legacy wire aliases.

The [kitty keyboard protocol](https://sw.kovidgoyal.net/kitty/keyboard-protocol/)
describes legacy ambiguities and opt-in enhanced reporting. The
[tmux modifier guide](https://github.com/tmux/tmux/wiki/Modifier-Keys) explains
character/control/Meta encoding; its pre-3.5 configuration advice is explicitly
outdated, so consult the [current manual](https://github.com/tmux/tmux/blob/master/tmux.1)
for extended-key configuration.

## Evidence and limits

| Evidence | What it establishes |
| --- | --- |
| `src/shortcuts/key.rs` and regression tests | Case and explicit Shift survive normalization; BackTab becomes Shift+Tab |
| Pinned Crossterm 0.29 Unix parser | `09` becomes Tab, `0d` Enter, `7f` Backspace; `08` becomes Ctrl+H; uppercase UTF-8 receives Shift |
| `src/terminal.rs` | Enhanced keyboard disambiguation is requested |
| Local tmux 3.8 server | `extended-keys on`; prefix is `C-b` |
| Existing PTY checks | Injected CSI-u/modified-arrow events dispatch; this does **not** prove a physical key produces them |
| User report | Shift combinations fail in their current workflows; exact received bytes have not yet been captured |

Do not infer a terminal capability solely from `$TERM`, the SSH hop, or an
installed version. SSH transports bytes; input encoding and interception depend
on the client and each multiplexer. No settings in the user's terminals were changed.

## Risk classes

| Class | Meaning | Policy |
| --- | --- | --- |
| Base | Ordinary text, control letter without a known alias here, or unmodified navigation | Keep, subject to host keymaps |
| Case | Uppercase printable character | Keep; `G` is distinct from `g` |
| Alias | Different gestures may produce the same legacy bytes | Provide an unambiguous alternative |
| Extended | Relies on modified-key encoding or enhanced protocol support | Optional convenience, not sole access |
| Host | Common terminal/multiplexer/desktop interception | Avoid as the only binding |
| Function | Function key, including modified function keys | Replace primary access; optionally retain alternate |
| Sequence | Multiple distinguishable inputs; current timeout applies | Useful portable expansion of key space |
| Unset | No assigned slot | Action still exists; expose through UI or deliberate assignment |

“Base” is not a universal guarantee. A user can bind any key in the host.
“Extended” is not a prohibition: it is a portability constraint.

## High-priority collisions

| Current default | Affected action | Failure mode | Proposed direction |
| --- | --- | --- | --- |
| Ctrl+Shift+S | Save as | Can become Ctrl+S → Save | Prefix, then `S` |
| Ctrl+Shift+O | Open directory | Can become Ctrl+O → Open document | Prefix, then `O` |
| Ctrl+Shift+E | Toggle sidebar | Can become Ctrl+E → Export | Prefix, then `d` |
| Ctrl+Shift+Z | Redo alternate | Can become Ctrl+Z → Undo | Keep Ctrl+Y primary; replace this alternate |
| Ctrl+I | Italic | Legacy Tab → indent or next field | Prefix, then `i` |
| Ctrl+B | Bold | Local tmux prefix consumes it | Prefix, then `b` |
| Ctrl+H | Hidden files | Physical Backspace may also send `08`; tuxd decodes it as Ctrl+H | Add a printable explorer-list binding; separate field-safe access |
| Ctrl+. | Hidden files alternate | No standard C0 encoding | Do not make it the portability fallback |
| Ctrl+, | Settings opener | No standard C0 encoding | Add a reserved sequence opener |
| Ctrl+/ | Settings opener | Common `1f` alias with Ctrl+_, Ctrl+7; mapping varies | Convenience alternate only |
| Ctrl+Backspace | Delete word | Can alias Backspace or Ctrl+H; implementation-dependent | Keep Alt+Backspace alternate; add sequence alternative |
| Ctrl+Alt+S/E | Save all / Focus sidebar | Often representable as ESC+control, but host/AltGr mappings vary | Add sequence alternatives |
| F1–F6, Ctrl+F4 | Settings, files, panes, wrapping, close | Laptop/mobile access; host interception | Sequence-based primary access |
| Ctrl+PageUp/Down | Previous/next document | Host tab navigation or unavailable modified encoding | Prefix, then `p` / `n` |

Other alias families to recognize in validation/diagnostics: Ctrl+M/Enter,
Ctrl+[/Escape, Ctrl+Space/Ctrl+@/NUL, and Ctrl+?/DEL/Backspace. Do not offer them
as independent portable bindings. Crossterm's current raw-mode handling keeps
Ctrl+J distinct from CR/Enter; do not blindly equate every linefeed with Enter.

Ctrl+C, Ctrl+Z and Ctrl+S/Q are not inherently forbidden: tuxd enters raw mode,
so ordinary tty signal/flow-control behavior is disabled there. Outer layers can
still consume them. They should not be removed merely because a shell uses them.

## Host-specific considerations

| Environment | Evidence / risk | Design response |
| --- | --- | --- |
| Alacritty | Ctrl+Shift+C/V copy/paste; Ctrl+Shift+B/F search; Ctrl+Shift+Space vi mode | Avoid those as sole app commands |
| Alacritty alternate screen | Default Shift+Home/End/PageUp/PageDown scrollback bindings use `~Alt` (not alternate screen) | Do not falsely label all selection keys intercepted in tuxd |
| tmux | Local prefix Ctrl+B; encoding depends on extended-key support along the chain | Give Bold another path; inspect actual received keys |
| Zed | Host keymap can consume shortcuts; terminal forwarding is configurable | Test focused Terminal context, not only app-level bindings |
| Termux over SSH | KeyHandler includes modified navigation and BackTab encodings | Do not classify all Shift as unsupported; test letter chords separately |
| Linux virtual console | Kernel keymap determines keyboard output; do not assume modern enhanced reporting | Baseline sequences of ordinary Ctrl letters and printable suffixes |

Sources: [Alacritty defaults](https://alacritty.org/config-alacritty-bindings.html),
[Zed forwarding](https://zed.dev/docs/key-bindings),
[Termux KeyHandler](https://github.com/termux/termux-app/blob/master/terminal-emulator/src/main/java/com/termux/terminal/KeyHandler.java).

## Proposed default strategy

Use one prefix followed by printable commands. **Ctrl+K is a candidate**, currently
unassigned in tuxd; it still needs end-to-end testing in the user's host keymaps.
This can use existing explicit sequence support without requiring the optional
leader to become mandatory. A configurable leader could mirror these bindings,
but changing its default is a separate product decision.

`Ctrl+K O` means release Ctrl+K, then type uppercase O. It does **not** mean
Ctrl+Shift+O. Lowercase suffixes should cover core actions; uppercase suffixes are
available for variants. No action may also occupy the standalone prefix.

| Family | Proposed sequences |
| --- | --- |
| Files | `Ctrl+K f` menu; `o` open; `O` directory; `s` save; `S` save as; `a` save all; `w` close; `n` next; `p` previous; `m` import; `x` export |
| Panes | `Ctrl+K e` Editor; `c` Code; `t` Tools; `d` sidebar toggle; `D` sidebar focus; `j` cycle |
| Display | `Ctrl+K E` collapse Editor; `C` collapse Code; `r` wrapping in Editor/Code |
| Styles | `Ctrl+K b` bold; `i` italic; `u` underline |
| Settings | `Ctrl+K ,` hard-coded opener alongside current convenience openers |
| Clipboard variants | `Ctrl+K v` plain; `V` literal; existing Ctrl+C/X/V remain |

All suffixes in this table include the prefix. These are discussion candidates,
not assigned defaults. They do not collide with each other; verify scopes and
user overrides through the compiler before adopting them. Ctrl+K currently has
no single-key action. Opening Settings via a reserved sequence requires an
extension: the current reserved-opener mechanism accepts single keys only.

Retain direct common keys such as Ctrl+N/O/S/Q, Ctrl+C/X/V/A, Ctrl+Z/Y where they
reach the application. Function keys can remain secondary conveniences without
being required. Avoid Ctrl+W as the universal prefix without testing: the local
Zed configuration already has pane-navigation sequences beginning with Ctrl+W.

For selection on terminals without modified navigation, consider a new **selection
anchor mode**: set anchor, navigate normally, finish/cancel. This is new behavior,
not something a default-key remapping alone can provide. For ranges, PageUp/Down
are candidates for ±10 where those keys are currently unassigned.

## Validation and implementation checkpoints

1. Add a key diagnostic view or command showing decoded code, modifiers, normalized
	binding, and detected/requested enhanced protocol state. An optional raw-byte
	probe must be separate from Crossterm's reader, not a second competing stdin reader.
2. Capture `o/O`, `e/E`, Ctrl+O/Ctrl+Shift+O, Ctrl+I/Tab, Ctrl+H/Backspace,
	Shift+Tab, Shift+arrows, Ctrl+K followed by `o/O`, and current Settings openers.
	Test Alacritty bare/tmux, Zed/tmux, Termux SSH/tmux, and Linux VC. Record actual
	results rather than declaring an entire terminal unsupported.
3. Add portability warnings to capture/validation. Distinguish exact/prefix conflicts
	(hard errors) from possible legacy aliases and host interception (warnings).
4. Agree the default prefix and suffix map, then update defaults, inline menu labels,
	docs and PTY fixtures together. Preserve user configurations unless a migration
	is explicitly designed; never silently remap their assignments.
5. Add selection-anchor fallback if required by VC evidence.

Existing constraints remain: ordinary printable prefixes are rejected in editable
contexts; reserved Settings keys cannot be assigned; Escape cancels multi-key
sequences; exact/prefix collisions are rejected in overlapping scopes; leader
references are resolved before checking conflicts. There is currently no blanket
ban on Shift, Alt or Ctrl+Shift assignments.

## Complete configurable inventory

Extracted from all seven capability-owned `ACTIONS` arrays: **116 actions**, each
with three slots, **127 assigned slots**. `—` means unset. Names in the Context
column are exact `InputScope` names, expanded from registry scope lists. Risk tags
cover every assigned slot and are assessed against the classes above.

### Workspace

| Action (ID) | Primary | Alternate 1 | Alternate 2 | Risk by assigned slot | Context |
| --- | --- | --- | --- | --- | --- |
| Save all (`SaveAll`) | `Ctrl+Alt+s` | — | — | Extended / host-dependent | Sidebar, Editor, Code, Tools, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch |
| Close document (`CloseBuffer`) | `Ctrl+F4` | — | — | Function | Sidebar, Editor, Code, Tools, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch |
| Next document (`NextBuffer`) | `Ctrl+PageDown` | — | — | Extended / host-dependent | Sidebar, Editor, Code, Tools, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch |
| Previous document (`PreviousBuffer`) | `Ctrl+PageUp` | — | — | Extended / host-dependent | Sidebar, Editor, Code, Tools, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch |
| New document (`New`) | `Ctrl+n` | — | — | Base | Sidebar, Editor, Code, Tools, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch |
| Open document (`Open`) | `Ctrl+o` | — | — | Base | Sidebar, Editor, Code, Tools, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch |
| Save (`Save`) | `Ctrl+s` | — | — | Base | Sidebar, Editor, Code, Tools, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch |
| Save as (`SaveAs`) | `Ctrl+Shift+s` | — | — | Alias | Sidebar, Editor, Code, Tools, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch |
| Import (`Import`) | — | — | — | Unset | Sidebar, Editor, Code, Tools, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch |
| Export (`Export`) | `Ctrl+e` | — | — | Base | Sidebar, Editor, Code, Tools, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch |
| Quit (`Quit`) | `Ctrl+q` | — | — | Base | Sidebar, Editor, Code, Tools, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch |
| Files menu (`FileMenu`) | `F4` | — | — | Function | Sidebar, Editor, Code, Tools, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch |
| Open directory (`OpenDirectory`) | `Ctrl+Shift+o` | — | — | Alias | Sidebar, Editor, Code, Tools, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch |
| Toggle directory sidebar (`ToggleSidebar`) | `Ctrl+Shift+e` | — | — | Alias | Sidebar, Editor, Code, Tools, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch |
| Focus directory sidebar (`FocusSidebar`) | `Ctrl+Alt+e` | — | — | Extended / host-dependent | Sidebar, Editor, Code, Tools, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch |
| Cycle panes (`CyclePanes`) | `F6` | — | — | Function | Sidebar, Editor, Code, Tools, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch |
| Focus Editor (`FocusEditor`) | — | — | — | Unset | Sidebar, Editor, Code, Tools, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch |
| Focus Code (`FocusCode`) | — | — | — | Unset | Sidebar, Editor, Code, Tools, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch |
| Focus Tools (`FocusTools`) | — | — | — | Unset | Sidebar, Editor, Code, Tools, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch |
| Previous slim tab (`PreviousTab`) | `Alt+Left` | — | — | Extended / host-dependent | Code, Tools, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch |
| Next slim tab (`NextTab`) | `Alt+Right` | — | — | Extended / host-dependent | Code, Tools, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch |
| Delete (`FieldDeleteAfter`) | `Delete` | — | — | Base | Location, Filename, CharacterSearch |
| Next field (`NextField`) | `Tab` | — | — | Base | Sidebar, Tools, CharacterPicker, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch, Explorer, Location, Filename, Format, FileButtons, Menu, Prompt, ReviewContent, ReviewButtons |
| Previous field (`PreviousField`) | `Shift+Tab` | — | — | Extended / host-dependent | Sidebar, Tools, CharacterPicker, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch, Explorer, Location, Filename, Format, FileButtons, Menu, Prompt, ReviewContent, ReviewButtons |
| Left (`FieldLeft`) | `Left` | — | — | Base | Tools, CharacterPicker, Characters, CharacterSearch, Radio, ToolTabs, Grid, Hex, Location, Filename, Format, FileButtons, Menu, Prompt, ReviewContent, ReviewButtons |
| Right (`FieldRight`) | `Right` | — | — | Base | Tools, CharacterPicker, Characters, CharacterSearch, Radio, ToolTabs, Grid, Hex, Location, Filename, Format, FileButtons, Menu, Prompt, ReviewContent, ReviewButtons |
| Up (`FieldUp`) | `Up` | — | — | Base | Tools, CharacterPicker, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch, Format, FileButtons, Menu, Prompt, ReviewContent, ReviewButtons |
| Down (`FieldDown`) | `Down` | — | — | Base | Tools, CharacterPicker, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch, Format, FileButtons, Menu, Prompt, ReviewContent, ReviewButtons |
| Page up (`FieldPageUp`) | `PageUp` | — | — | Base | Tools, CharacterPicker, Grid, Characters, ReviewContent, ReviewButtons |
| Page down (`FieldPageDown`) | `PageDown` | — | — | Base | Tools, CharacterPicker, Grid, Characters, ReviewContent, ReviewButtons |
| Beginning (`FieldHome`) | `Home` | — | — | Base | Tools, CharacterPicker, Characters, CharacterSearch, Radio, ToolTabs, Grid, Hex, Location, Filename, Menu, ReviewContent, ReviewButtons |
| End (`FieldEnd`) | `End` | — | — | Base | Tools, CharacterPicker, Characters, CharacterSearch, Radio, ToolTabs, Grid, Hex, Location, Filename, Menu, ReviewContent, ReviewButtons |
| Activate (`Activate`) | `Enter` | `Space` | — | Base; Base | Sidebar, Tools, CharacterPicker, Characters, Radio, ToolTabs, Grid, Explorer, Format, FileButtons, Menu, Prompt, ReviewButtons |
| Submit (`Submit`) | `Enter` | — | — | Base | Location, Filename, Hex, Channel, CharacterSearch |
| Decrease (`RangeDecrease`) | `Left` | — | — | Base | Channel |
| Increase (`RangeIncrease`) | `Right` | — | — | Base | Channel |
| Decrease by 10 (`RangeDecrease10`) | `Shift+Left` | — | — | Extended / host-dependent | Channel |
| Increase by 10 (`RangeIncrease10`) | `Shift+Right` | — | — | Extended / host-dependent | Channel |
| Minimum (`RangeMinimum`) | `Home` | — | — | Base | Channel |
| Maximum (`RangeMaximum`) | `End` | — | — | Base | Channel |
| Backspace (`FieldDeleteBefore`) | `Backspace` | — | — | Base | Location, Filename, Hex, Channel, CharacterSearch |
| Copy (`Copy`) | `Ctrl+c` | — | — | Base | Editor, Code, Location, Filename, Hex, Channel, CharacterSearch |
| Paste (`Paste`) | `Ctrl+v` | — | — | Base | Editor, Explorer, Location, Filename, Format, FileButtons, Hex, Channel, CharacterSearch |
| Cut (`Cut`) | `Ctrl+x` | — | — | Base | Editor |
| Paste without formatting (`PastePlain`) | — | — | — | Unset | Editor |
| Paste literal text (`PasteLiteral`) | — | — | — | Unset | Editor |
| Select all (`SelectAll`) | `Ctrl+a` | — | — | Base | Editor, Location, Filename, Hex, Channel, CharacterSearch |

### Editor

| Action (ID) | Primary | Alternate 1 | Alternate 2 | Risk by assigned slot | Context |
| --- | --- | --- | --- | --- | --- |
| Toggle Editor (`ToggleEditor`) | `F5` | — | — | Function | Sidebar, Editor, Code, Tools, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch |
| Toggle wrapping (`EditorWrap`) | `F3` | — | — | Function | Editor |
| Toggle bold (`Bold`) | `Ctrl+b` | — | — | Host (tmux) | Sidebar, Editor, Code, Tools, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch |
| Toggle italic (`Italic`) | `Ctrl+i` | — | — | Alias | Sidebar, Editor, Code, Tools, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch |
| Toggle underline (`Underline`) | `Ctrl+u` | — | — | Base | Sidebar, Editor, Code, Tools, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch |
| Undo (`Undo`) | `Ctrl+z` | — | — | Base | Sidebar, Editor, Code, Tools, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch |
| Redo (`Redo`) | `Ctrl+y` | `Ctrl+Shift+z` | — | Base; Alias | Sidebar, Editor, Code, Tools, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch |
| Left (`MoveLeft`) | `Left` | — | — | Base | Editor |
| Select left (`SelectLeft`) | `Shift+Left` | — | — | Extended / host-dependent | Editor |
| Right (`MoveRight`) | `Right` | — | — | Base | Editor |
| Select right (`SelectRight`) | `Shift+Right` | — | — | Extended / host-dependent | Editor |
| Up (`MoveUp`) | `Up` | — | — | Base | Editor |
| Select up (`SelectUp`) | `Shift+Up` | — | — | Extended / host-dependent | Editor |
| Down (`MoveDown`) | `Down` | — | — | Base | Editor |
| Select down (`SelectDown`) | `Shift+Down` | — | — | Extended / host-dependent | Editor |
| Line beginning (`MoveHome`) | `Home` | — | — | Base | Editor |
| Select line beginning (`SelectHome`) | `Shift+Home` | — | — | Extended / host-dependent | Editor |
| Line end (`MoveEnd`) | `End` | — | — | Base | Editor |
| Select line end (`SelectEnd`) | `Shift+End` | — | — | Extended / host-dependent | Editor |
| Page up (`MovePageUp`) | `PageUp` | — | — | Base | Editor |
| Select page up (`SelectPageUp`) | `Shift+PageUp` | — | — | Extended / host-dependent | Editor |
| Page down (`MovePageDown`) | `PageDown` | — | — | Base | Editor |
| Select page down (`SelectPageDown`) | `Shift+PageDown` | — | — | Extended / host-dependent | Editor |
| Document beginning (`MoveStart`) | `Ctrl+Home` | — | — | Extended / host-dependent | Editor |
| Select document beginning (`SelectStart`) | `Shift+Ctrl+Home` | — | — | Alias | Editor |
| Document end (`MoveFinish`) | `Ctrl+End` | — | — | Extended / host-dependent | Editor |
| Select document end (`SelectFinish`) | `Shift+Ctrl+End` | — | — | Alias | Editor |
| Previous word (`MoveWordLeft`) | `Ctrl+Left` | `Alt+Left` | — | Extended / host-dependent; Extended / host-dependent | Editor |
| Select previous word (`SelectWordLeft`) | `Shift+Ctrl+Left` | `Shift+Alt+Left` | — | Alias; Extended / host-dependent | Editor |
| Next word (`MoveWordRight`) | `Ctrl+Right` | `Alt+Right` | — | Extended / host-dependent; Extended / host-dependent | Editor |
| Select next word (`SelectWordRight`) | `Shift+Ctrl+Right` | `Shift+Alt+Right` | — | Alias; Extended / host-dependent | Editor |
| Backspace (`DeleteBefore`) | `Backspace` | — | — | Base | Editor |
| Delete (`DeleteAfter`) | `Delete` | — | — | Base | Editor |
| Delete previous word (`DeleteWord`) | `Ctrl+Backspace` | `Alt+Backspace` | — | Alias; Extended / host-dependent | Editor |
| Newline (`InsertNewline`) | `Enter` | — | — | Base | Editor |
| Indent (`InsertTab`) | `Tab` | — | — | Base | Editor |
| Clear selection (`ClearSelection`) | `Esc` | — | — | Base | Sidebar, Editor, Code, Tools, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch |

### Code

| Action (ID) | Primary | Alternate 1 | Alternate 2 | Risk by assigned slot | Context |
| --- | --- | --- | --- | --- | --- |
| Toggle Code (`ToggleCode`) | `F2` | — | — | Function | Sidebar, Editor, Code, Tools, Radio, ToolTabs, Grid, Hex, Channel, Characters, CharacterSearch |
| Toggle wrapping (`CodeWrap`) | `F3` | — | — | Function | Code |
| Up (`CodeUp`) | `Up` | — | — | Base | Code |
| Down (`CodeDown`) | `Down` | — | — | Base | Code |
| Left (`CodeLeft`) | `Left` | — | — | Base | Code |
| Right (`CodeRight`) | `Right` | — | — | Base | Code |
| Page up (`CodePageUp`) | `PageUp` | — | — | Base | Code |
| Page down (`CodePageDown`) | `PageDown` | — | — | Base | Code |
| Line beginning (`CodeHome`) | `Home` | — | — | Base | Code |
| Line end (`CodeEnd`) | `End` | — | — | Base | Code |
| Document beginning (`CodeStart`) | `Ctrl+Home` | — | — | Extended / host-dependent | Code |
| Document end (`CodeFinish`) | `Ctrl+End` | — | — | Extended / host-dependent | Code |

### Tools

| Action (ID) | Primary | Alternate 1 | Alternate 2 | Risk by assigned slot | Context |
| --- | --- | --- | --- | --- | --- |
| Apply RGB (`RgbApply`) | — | — | — | Unset | Hex, Channel |
| Search characters (`CharacterSearch`) | `Ctrl+f` | — | — | Base | Characters, CharacterSearch |

### Dialogs

| Action (ID) | Primary | Alternate 1 | Alternate 2 | Risk by assigned slot | Context |
| --- | --- | --- | --- | --- | --- |
| Positive action (`Positive`) | `s` | `y` | — | Base; Base | Prompt, ReviewContent, ReviewButtons, FileButtons |
| Negative action (`Negative`) | `d` | `n` | — | Base; Base | Prompt, ReviewContent, ReviewButtons, FileButtons |
| Abort action (`Abort`) | `a` | `c` | — | Base; Base | Prompt, ReviewContent, ReviewButtons, FileButtons |
| Close / dismiss (`Close`) | `Esc` | — | — | Base | Menu, CharacterPicker, Prompt, ReviewContent, ReviewButtons, Explorer, Location, Filename, Format, FileButtons |

### Explorer

| Action (ID) | Primary | Alternate 1 | Alternate 2 | Risk by assigned slot | Context |
| --- | --- | --- | --- | --- | --- |
| Refresh directory (`RefreshDirectory`) | `Ctrl+r` | — | — | Base | Sidebar |
| Focus Location (`PickerLocation`) | `Ctrl+l` | — | — | Base | Explorer, Location, Filename, Format, FileButtons |
| Go to CWD (`PickerCwd`) | `Ctrl+g` | — | — | Base | Explorer, Location, Filename, Format, FileButtons |
| Toggle hidden files (`PickerHidden`) | `Ctrl+h` | `Ctrl+.` | — | Alias; Extended | Explorer, Location, Filename, Format, FileButtons, Sidebar |
| Toggle filter (`PickerFilter`) | `Ctrl+f` | — | — | Base | Explorer, Location, Filename, Format, FileButtons, Sidebar |
| Parent directory (`ExplorerParent`) | `Left` | `Backspace` | `h` | Base; Base; Base | Explorer, Sidebar |
| Enter directory (`ExplorerEnter`) | `Right` | `l` | — | Base; Base | Explorer, Sidebar |
| Previous entry (`ExplorerUp`) | `Up` | `k` | — | Base; Base | Explorer, Sidebar |
| Next entry (`ExplorerDown`) | `Down` | `j` | — | Base; Base | Explorer, Sidebar |
| First entry (`ExplorerFirst`) | `Home` | `g g` | — | Base; Sequence | Explorer, Sidebar |
| Last entry (`ExplorerLast`) | `End` | `G` | — | Base; Case | Explorer, Sidebar |
| Middle visible entry (`ExplorerMiddle`) | `M` | — | — | Case | Explorer, Sidebar |
| Previous page (`ExplorerPageUp`) | `PageUp` | — | — | Base | Explorer, Sidebar |
| Next page (`ExplorerPageDown`) | `PageDown` | — | — | Base | Explorer, Sidebar |

## Hard-coded Settings controls

These sit outside the 116-action registry and intentionally cannot be rebound.
Source: `src/settings/{input,navigation,bindings,general,appearance}.rs` and
`src/shortcuts/key.rs`.

| Context | Keys | Action | Portability |
| --- | --- | --- | --- |
| Open Settings | F1 / Ctrl+/ / Ctrl+, | Open at related category | Function / alias / extended; no universally portable opener yet |
| Browse | Esc | Close or leave narrow category selector | Base |
| Browse | Enter | Activate field, section, binding capture or button | Base |
| Browse | Tab / Shift+Tab | Next / previous focus | Base / BackTab encoding |
| Browse | Arrows / h j k l | Navigate lists, categories, fields, buttons | Base; depends on focused control |
| Browse | Home / g g | First row/category | Base / sequence |
| Browse | End / G | Last row/category | Base / case |
| Browse | M | Middle row/category | Case |
| Browse | PageUp / PageDown | Page through lists; category navigation treats these as up/down | Base |
| Browse | Ctrl+S | Save settings | Base in raw mode |
| Browse | Ctrl+R | Reset all | Base |
| Browse | Ctrl+T | Edit sequence timeout | Base |
| Browse | Ctrl+L | Focus categories | Base; host may use it |
| Binding/action row | Enter / 1–9 | Capture one / N inputs | Base |
| Binding or Unset focus | Delete / Backspace | Unset selected binding | Base; Backspace encoding varies |
| Leader focus | Delete | Request unsetting leader | Base |
| Appearance color row | Delete | Remove override | Base |
| Capture | Esc | Cancel capture | Base; Esc cannot be recorded interactively |
| Capture | Other press events | Record decoded key; first configured leader extends capture | Subject to input path; repeat/release ignored |
| Unsaved-settings prompt | Arrows / Tab / h j k l | Switch choice | Base; no directional Shift+Tab distinction |
| Unsaved-settings prompt | Enter / Esc | Activate choice / return to settings | Base |
| Leader-impact prompt | Left / h; Right / l / Tab | Previous / next choice | Base; see Shift+Tab bug below |
| Leader-impact prompt | Up / k; Down / j | Scroll affected bindings | Base |
| Leader-impact prompt | Enter; Esc / c | Choose; cancel | Base |
| Leader-impact prompt | y | Accept new leader | Base |
| Unset-leader prompt | p / r | Preserve literal bindings / remove leader bindings | Base |
| Timeout editor | Digits, Backspace, Ctrl+A, Enter, Esc | Edit, select all, apply, cancel | Base; Backspace encoding varies |
| Color editor | Printable text, Backspace, Ctrl+A, Enter, Esc | Edit, select all, apply, cancel | Base; bracketed paste is also handled |
| Preset dropdown | Up/k, Down/j, Home/g, End/G, PageUp/Down | Navigate presets (single g here) | Base / case |
| Preset dropdown | Enter / Esc | Select / cancel | Base |

Some Settings handlers match `KeyCode` without requiring modifiers, so extra
modified gestures may incidentally activate the same operation. These are not
separate intentional shortcuts and should be made consistent during the follow-up.

### Confirmed local Shift handling defect

`leader_choice_key` normalizes BackTab to Tab+Shift, then tests for BackTab and
handles every Tab as forward. Consequently **Shift+Tab moves forward in this
specific confirmation prompt**. Main Settings browsing correctly checks the
normalized Shift modifier. Fix this narrowly with a regression test; it does not
explain Ctrl+Shift+O or imply that uppercase letters are lost globally.

## Follow-up scope

This audit adds one case-preservation regression test, but changes no production
bindings or terminal configuration. The focused shortcut suite passes (18 tests).
The preceding directory/drag checkpoint passed the full check suite before commit.
