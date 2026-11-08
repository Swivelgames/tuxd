# Getting started

[Documentation index](README.md) · [Project overview](../README.md)

## Install

Before you start:

- **Linux** with a UTF-8 terminal.
- **Rust and Cargo** installed. The declared minimum is Rust 1.88; a current stable toolchain is recommended.

```sh
git clone https://gitlab.com/swivel/tux-design.git
cd tux-design
cargo install --path . --locked
```

Cargo normally installs the executable in `~/.cargo/bin`; add that directory to your `PATH` if necessary. A custom `CARGO_HOME` or install root changes the destination.

```sh
tuxd --version
tuxd --help
```

Desktop clipboard access needs a suitable helper: `wl-copy` and `wl-paste` on Wayland, or `xclip` or `xsel` on X11. SSH and tmux copies use forwarding instead. You can always export to a file. See [Troubleshooting](troubleshooting.md).

## Explore an example

From the checkout:

```sh
tuxd examples/welcome.tuxd.json
```

| Area | What it does |
| --- | --- |
| Editor | Edit and select styled text |
| Code | Inspect and copy escaped ANSI |
| Tools | Apply styles, colors, and characters |

| Task | Control |
| --- | --- |
| Change focus | F6 or click a pane |
| Collapse Code / Editor | F2 / F5 |
| Change lower tabs on a narrow terminal | Alt+Left / Right, or click a tab |

At **64 columns or fewer**, Code and tools share the lower pane.

## Make a message

1. Run `tuxd` for a new document.
2. Type `Hello, terminal`.
3. Select `Hello` with Shift+arrows or a mouse drag.
4. Press **Ctrl+B** or click Bold.
5. Choose a foreground color from **Color → Basic**.
6. Click Editor and press **Esc** to clear the selection.

**Result:** `Hello` is bold and colored, and Code shows the ANSI needed to produce it.

With no selection, style changes apply to subsequent typing. Undo and redo default to Ctrl+Z and Ctrl+Y. Code updates automatically as you edit.

## Copy or export

| What you want | What to do |
| --- | --- |
| Copy the complete design | Focus Code, then Ctrl+C |
| Copy selected text with formatting | Select in Editor, then Ctrl+C |
| Paste text or ANSI | Ctrl+V or your terminal’s paste command |
| Write ANSI to a file | Ctrl+E, choose Raw or Escaped, then Export |

**Escaped ANSI** contains notation such as `\x1b[31m`. **Raw ANSI** contains actual escape bytes. Language-specific strings may need additional escaping.

Export leaves the document unsaved. Continue below to save an editable copy.

## Save your work

1. Press **Ctrl+S**.
2. Choose a directory and enter `greeting.tuxd.json`.
3. Activate **Save**.
4. Reopen it later with `tuxd greeting.tuxd.json`.

**Result:** text and formatting are preserved in an editable document.

The title’s `*` marks unsaved document changes. Ctrl+Q offers Save, Discard, or Cancel; Cancel is initially focused. There is no autosave or crash recovery. Avoid modifying the same file in another program while it is open in tuxd.

## Customize

F1 opens Settings. Choose Theme → Appearance for presets and color roles, or a Shortcuts category for bindings. Changes are drafts until Save; Cancel lets you discard them. Appearance previews immediately. Settings opening and navigation keys stay fixed.

## Update or uninstall

To update a checkout installation, pull your desired revision and rerun `cargo install --path . --locked`.

```sh
cargo uninstall tuxd
```

Uninstalling the executable leaves your documents and configuration intact. Configuration normally lives in `~/.config/tuxd/`, or `$XDG_CONFIG_HOME/tuxd/`. Back it up before removing it manually.
