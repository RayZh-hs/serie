# Compatibility

## Supported terminal emulators

These image protocols are supported:

- [Inline Images Protocol (iTerm2)](https://iterm2.com/documentation-images.html)
- [Terminal graphics protocol (kitty)](https://sw.kovidgoyal.net/kitty/graphics-protocol/)
  - Supports both the existing graphics protocol mode and [the Unicode placeholder](https://sw.kovidgoyal.net/kitty/graphics-protocol/#unicode-placeholders) mode.
- Sixel graphics, selected explicitly with `--protocol sixel`.

The terminals on which each has been confirmed to work are listed below.

### Inline Images Protocol

| Terminal emulator                                                                   | Note                                                                                                                                         |
| ----------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| [iTerm2](https://iterm2.com)                                                        | But slower than other terminals                                                                                                              |
| [WezTerm](https://wezfurlong.org/wezterm/)                                          |                                                                                                                                              |
| [Rio](https://rioterm.com)                                                          |                                                                                                                                              |
| [VSCode integrated terminal](https://code.visualstudio.com/docs/terminal/basics) \* | Requires the [`terminal.integrated.enableImages` setting](https://code.visualstudio.com/docs/terminal/advanced#_image-support) to be enabled |

\*Not only the VSCode integrated terminal, but any terminal emulator using [xterm.js](https://xtermjs.org) may basically work in the same way as long as [image display feature is enabled](https://github.com/xtermjs/xterm.js/tree/master/addons/addon-image).

### Terminal graphics protocol

| Terminal emulator                         | Unicode placeholder | Note |
| ----------------------------------------- | ------------------- | ---- |
| [kitty](https://sw.kovidgoyal.net/kitty/) | ○                  |      |
| [Ghostty](https://ghostty.org)            | ○                  |      |

Rendering using Unicode Placeholder is available by explicitly specifying `kitty-unicode` as `protocol` option or config.

### Sixel graphics

This fork adds Sixel output for the existing pixel-rendered graph. Select it with
`serie --protocol sixel` or configure:

```toml
[core.option]
protocol = "sixel"
```

The terminal must report its text area dimensions in pixels, so each image can
be scaled to exactly one text row. Images are regenerated when the font cell
size changes. Plain upstream Alacritty does not implement Sixel; use
[ayosec/alacritty](https://github.com/ayosec/alacritty) with its graphics patch.

Verified on Linux/Wayland with Alacritty `0.17.0 (12d0fad)`: initial rendering,
selection changes, page scrolling, opening and closing commit details and help,
window resizing, and runtime font size changes.

Sixel does not carry partial alpha transparency. Fully transparent graph
backgrounds remain transparent, while branch edges use a color palette.

Wide graphs use a horizontal viewport capped at one third of the terminal
width, leaving the remaining space for commit text. Use `h`/`l` or Left/Right
in the commit list to pan. The viewport follows the selected commit when the
selection changes. Resizing the window adjusts the viewport automatically.

### Partially supported environments

- tmux is supported only when using the kitty Unicode placeholder protocol.
  - Requires `set -g allow-passthrough on` in tmux.conf (version 3.2+).

### Unsupported environments

- Other terminal multiplexers (screen, Zellij, etc.) other than those listed in [Partially supported environments](#partially-supported-environments) are not supported.
- Windows is not officially supported. Please refer to [the related issue](https://github.com/lusingander/serie/issues/147#issuecomment-4192875627).
