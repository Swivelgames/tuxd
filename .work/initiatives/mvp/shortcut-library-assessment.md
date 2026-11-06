# Shortcut library assessment

Assessment date: 2026-11-06. Supersedes the initial, incomplete recommendation to start with an application-owned dispatcher.

## Scope and evidence

Surveyed eight relevant libraries plus Ratatui's configuration template. Read primary documentation, then inspected published source for the strongest sequence candidates and several representation libraries. Compiled an isolated probe against `ratatui-which-key` 0.14.0 and `keybinds` 0.2.0 with Crossterm 0.29: all eight checks pass, including assertions that reproduce limitations. This is not an exhaustive ecosystem census or an end-to-end application integration.

The repository currently uses Ratatui 0.30 and Crossterm 0.29. `ratatui-which-key` 0.14.0 has subsequently been selected and added as a pinned production dependency. Runtime shortcut behavior has not changed.

## Comparison

| Candidate | Reusable capabilities | Evidence and fit |
| --- | --- | --- |
| `ratatui-which-key` 0.14.0 | Scoped/global actions, categories, sequence tree, prefix/leaf lookup, pending-state inspection, optional popup | Strong candidate. Published dependencies match Ratatui 0.30 / Crossterm 0.29. Probe confirms sequences, scope isolation, global bindings, and custom key representation. Default formatter/parser cannot faithfully represent all required modifier combinations. Generic `Key` trait allows an adapter. License metadata: LGPL-3.0-or-later. |
| `keybinds` 0.2.0 (rhysd/keybinds-rs) | Sequence parsing/dispatch, configurable timeout, reset, pending inputs, runtime binding construction, Serde | Strong alternative. Uses Crossterm 0.29; MIT. Probe confirms sequences, timeout/reset, BackTab conversion, Unicode and JSON round trips. Character input normalization removes Shift, including on lowercase Ctrl-modified characters; preserving captured distinctions needs an adapter/original-event model. Scope composition is application-owned. |
| `crokey` 1.5.1 | Key parsing, formatting, Serde, Crossterm conversions | Useful building block, not a temporal sequence dispatcher. Its multi-key combinations/Combiner refer to simultaneously held keys, not the requested successive-input sequences. Uses Crossterm 0.29; MIT. Source/docs reviewed, not compile-probed. |
| `crossterm-keybind` / core 0.4.4 | Configurable action bindings, multiple alternatives, serialization and display helpers | Core source supports Crossterm 0.29 and checks individual code/modifier pairs. A useful alternative for configuration plumbing; inspected `KeyBindings::match_any` is single-event matching, not our sequence engine. No evidence from this review that it replaces the scoped sequence dispatcher. |
| `ratatui-kit-keymap` 0.1.0 | Per-scope semantic actions, defaults/overrides, conflict warnings, effective-binding descriptions | Documentation explicitly rejects multi-key chords and uses crokey. Useful reference for effective binding metadata and config validation; does not supply required temporal sequence dispatch. Framework integration is additional scope. Documentation review only. |
| `ratatui-input-manager` 0.4.0 | Declarative method dispatch, key metadata, Help/HelpBar widgets | Uses Crossterm 0.29. Reviewed API centers on macro-defined method bindings, not runtime recording/editing of temporal sequences. No basis to select it as the main settings/sequence engine without further evidence. Documentation review only. |
| `keymap` 0.4.1 | Config-to-key parsing and Serde | Published source uses Crossterm 0.28. The GitHub main README advertises newer sequence/derive capabilities that are not established by the published 0.4.1 package. Earlier assessment conflated those surfaces; do not rely on repository-main features as released APIs. |
| `kbd` + `kbd-crossterm` 0.2.1 | Dispatcher, layers and sequences through a physical-key-style adapter | Documentation reports unsupported non-ASCII characters/BackTab and ignored Hyper/Meta modifiers. The adapter is a poor fit for faithful terminal-event capture. Documentation review only. |

Ratatui's component template also demonstrates per-mode key configuration and parsing. It is a reference implementation, rather than a packaged engine, and does not remove the need for our settings workflow.

## Verified integration details

### ratatui-which-key

`Keymap::navigate` exposes Branch/Leaf/no-match results directly. We can reuse its tree without rendering the which-key popup, so adopting it does not require shortcut-hint clutter. Shared keys can resolve differently by scope, and true global bindings are supported.

The generic `Key` trait lets an application wrapper preserve exact decoded key/modifier distinctions. The probe binds and resolves a combined-modifier key and Unicode using such a wrapper; it also reproduces the stock formatter's loss of Shift on a Ctrl-character.

`WhichKeyState::set_scope` does not clear pending input; `dismiss` does. Its state handler treats Backspace as sequence backtracking. There is no built-in elapsed-time policy in the inspected state implementation. A host timer and explicit cancellation can wrap `navigate`, or the state API can be adapted. Do not mistake these integration points for a reason to rewrite the tree.

### keybinds

Public APIs include `set_timeout`, `reset`, `is_ongoing`, `ongoing_inputs`, sequence parsing and Serde. Timeout handling runs during dispatch; the host still needs idle-time status cleanup. It accepts ambiguous prefix assignments, so our overlap validator must reject them before installing bindings.

The failed first probe exposed `ShiftUnavailable(Char('z'))` for the text `Ctrl+Shift+z`. The published model encodes shifted character case instead and removes Shift when constructing character inputs. A successful revised test records this limitation explicitly; it is not evidence that raw Ctrl+Shift+z capture already round-trips faithfully. BackTab is handled by converting the full KeyEvent to Shift+Tab.

## Selection

User decision: use **`ratatui-which-key` 0.14.0**. Reuse its scoped sequence engine with a faithful key adapter; application code supplies the agreed timeout, cancellation, validation, settings editor, and persistence.

`ratatui-which-key` is the leading scoped-engine candidate: it already models the action/scope/category structure we need and permits exact key wrappers. Use its lookup engine independently of the popup; add the agreed timeout/cancellation policy around it. Check license fit as part of dependency selection; no project license decision is recorded here.

`keybinds` is a credible alternative with built-in timeout and Serde, especially if we prefer composing one active binding set per context. Before selection, demonstrate preservation of our existing Ctrl+Shift+Z behavior and captured-key representation through a normalization adapter.

The action catalogue, three binding slots, context-overlap/text-protection rules, draft Save/Cancel/Reset, and XDG persistence remain application-specific with either library. That does not imply reimplementing sequence parsing or matching.

The dependency choice is settled. The comparison and probes remain as the decision evidence; neither crate is claimed to supply the entire interactive settings editor.

## Reproducible probe

Run:

```sh
cargo test --manifest-path .work/initiatives/mvp/research/shortcut-library-probe/Cargo.toml
```

Eight checks cover scoped sequence lookup/global actions, scope cancellation behavior, stock formatting limitations, a custom faithful key wrapper, sequence dispatch/pending state, timeout/reset/prefix acceptance, BackTab/Unicode/JSON, and Shift normalization. All passed on the local Rust toolchain. They do not yet verify the application's full context graph, capture UI, or persistence transaction.

## Primary sources

- [ratatui-which-key documentation](https://docs.rs/ratatui-which-key/0.14.0/ratatui_which_key/) and [published source](https://docs.rs/crate/ratatui-which-key/0.14.0/source/)
- [keybinds API](https://docs.rs/keybinds/0.2.0/keybinds/struct.Keybinds.html) and [published source](https://docs.rs/crate/keybinds/0.2.0/source/)
- [crokey documentation](https://docs.rs/crokey/1.5.1/crokey/)
- [crossterm-keybind documentation](https://docs.rs/crossterm-keybind/0.4.4/crossterm_keybind/) and [core source](https://docs.rs/crate/crossterm-keybind-core/0.4.4/source/)
- [ratatui-kit-keymap documentation](https://docs.rs/ratatui-kit-keymap/0.1.0/ratatui_kit_keymap/)
- [ratatui-input-manager documentation](https://docs.rs/ratatui-input-manager/0.4.0/ratatui_input_manager/)
- [keymap published documentation](https://docs.rs/keymap/0.4.1/keymap/) and [repository main](https://github.com/rezigned/keymap-rs)
- [kbd-crossterm mappings](https://docs.rs/kbd-crossterm/0.2.1/kbd_crossterm/)
- [Ratatui configuration template](https://ratatui.rs/templates/component/config-rs/)
