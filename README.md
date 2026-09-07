# xuzzel

Fast X11 application launcher and dmenu-style picker built on dmenu 5.4's Xlib/Xft foundation. It aims to provide fuzzel 1.15-compatible behavior and configuration where X11 has a sensible equivalent.

Xuzzel runs as a centered override-redirect window, supports keyboard and mouse control, and closes when clicking outside it. Launcher mode discovers freedesktop desktop entries; dmenu mode reads choices from standard input and prints the selected value.

## Build

Dependencies: C99 compiler, pkg-config, Xlib, Xft/fontconfig, Xinerama.

    make
    make check
    ./xuzzel
    printf 'one\ntwo\n' | ./xuzzel --dmenu

Install with `make install PREFIX=/usr/local`. This installs the binary, man pages, desktop entry, and example configuration. Use `make uninstall PREFIX=/usr/local` to remove them.

## Usage

Run as an application launcher:

    xuzzel

Use it as a picker:

    printf 'shutdown\nreboot\ncancel\n' | xuzzel --dmenu --prompt='Power: '

Useful references:

    man xuzzel
    man xuzzel.ini
    xuzzel --help

Escape or an outside click cancels with status 1. Enter accepts the selected entry. Arrow keys, Page Up/Down, mouse wheel, and pointer selection navigate results.

## Configuration

`xuzzel.ini` mirrors fuzzel 1.15.0 defaults with Wayland-only settings removed. Copy it to `~/.config/xuzzel/xuzzel.ini` to customize it. Installation puts an untouched example in `share/doc/xuzzel/xuzzel.ini` rather than overriding user configuration.

Configuration lookup prefers `fuzzel/fuzzel.ini`, then `xuzzel/xuzzel.ini`, under `$XDG_CONFIG_HOME`, `~/.config`, and `$XDG_CONFIG_DIRS`. Command-line options override file values. Validate configuration without opening a window using:

    xuzzel --check-config

## Implemented

- XDG desktop discovery, launch, terminal entries, hidden/no-display handling
- dmenu stdin/stdout, prompt/password/search/select/auto-select modes
- fzf-style subsequence and exact matching, source-order mode
- persistent launcher history under XDG cache
- fuzzel/xuzzel INI search, validation and command-line overrides
- fuzzel 1.15 short/long option-name surface, NUL input, index output
- Xft/fontconfig rendering, keyboard/mouse editing, mouse-disable configuration, clipboard paste
- outside-click cancellation with pointer cleanup
- Xinerama monitor index, anchors/margins, server/Xft DPI behavior
- colors, spacing, borders, sorted/source-order results, match counter, minimal/hide modes

## Known gaps

X11 has no exact layer-shell, namespace, compositor-output-name, Wayland IME, fractional-scale protocol, blur, gamma-correct Xft blending, or keyboard-focus-loss equivalent. Alpha, rounded clipping, action menus, icon/SVG drawing, desktop actions, message wrapping/expansion, and full Damerau-Levenshtein fuzzy mode remain incomplete. Compatibility-only options are accepted where documented by `xuzzel(1)`.

## License

MIT/X Consortium license. Drawing primitives and event-loop design retain dmenu attribution. Fuzzel source is not copied; its documented behavior and configuration define compatibility goals. See `LICENSE`.

## License and attribution

See `LICENSE`. Drawing primitives and event-loop design retain dmenu 5.4 lineage and notices (Copyright 2006-2024 suckless.org). Fuzzel source is not copied; 1.15.0 manuals and behavior define compatibility goals.
