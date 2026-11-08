# Documentation media

These are real tuxd captures from an isolated Alacritty window on Xvfb, with a temporary configuration directory. They contain no personal documents or desktop content.

- `social-preview.png`: 1280×640 GitHub/social sharing card composed from the real overview capture.
- `overview.png`: Editor, live Code output, and styling tools.
- `settings.png`: contextual shortcut categories and binding controls.
- `themes/*.png`: every built-in preset using the same document and layout.
- `demo.gif`: selection, underline formatting, Code collapse/expand, and Settings.

The document is [showcase.tuxd.json](../../examples/showcase.tuxd.json). Open it with `cargo run --locked -- examples/showcase.tuxd.json` to explore the same content. Captures use a 116-column, 48-row Alacritty window, DejaVu Sans Mono at 13 points, and the default application settings. The GIF uses captured frames with a 1.4-second display delay; playback timing is condensed for demonstration.

Regenerate the full set with `python3 scripts/capture_docs.py` (requires Alacritty,
Xvfb, xdotool, and ImageMagick). It uses a temporary copy of the showcase document
and temporary configuration; personal settings are not changed.

Regenerate these assets when the interface changes. Capture the terminal window only, use an isolated configuration, and inspect both the original images and the animation before updating the documentation.
