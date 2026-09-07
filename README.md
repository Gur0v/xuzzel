# xuzzel

Fast X11 application launcher and dmenu-style picker built on dmenu 5.4's Xlib/Xft foundation. It aims to provide fuzzel 1.15-compatible behavior and configuration where X11 has a sensible equivalent.

Xuzzel runs as a centered override-redirect window, supports keyboard and mouse control, and closes when clicking outside it. Launcher mode discovers freedesktop desktop entries; dmenu mode reads choices from standard input and prints the selected value.

## Highlights

- X11-native launcher with Xinerama monitor placement and Picom-friendly borders
- PNG and SVG icons resolved through freedesktop icon themes
- exact, fzf-style subsequence, and Unicode-aware fuzzy matching
- fuzzel-style INI configuration and command-line options
- dmenu-compatible newline or NUL-delimited picker operation
- launch history, keyboard editing, clipboard paste, and mouse navigation

## Build

Dependencies: C99 compiler, pkg-config, Xlib, Xft/fontconfig, Xinerama, Cairo/Xlib, and libpng. SVG icons use bundled NanoSVG.

Typical development packages are `libx11`, `libxft`, `fontconfig`, `libxinerama`, `cairo`, and `libpng`; exact names vary by distribution.

    make
    make check
    ./xuzzel
    printf 'one\ntwo\n' | ./xuzzel --dmenu

Install with `make install PREFIX=/usr/local`. This installs the binary, man pages, desktop entry, and example configuration. Use `make uninstall PREFIX=/usr/local` to remove them.

Default release build uses `-O3`, link-time optimization, dead-section removal, and strict warnings as errors. Override flags when packaging or debugging:

    make clean
    make CFLAGS='-O0 -g3' LDFLAGS=

Run AddressSanitizer and UndefinedBehaviorSanitizer when compiler runtimes are available:

    make sanitize

## Usage

Run as an application launcher:

    xuzzel

Type any shell command and press Enter when no desktop entry matches. This supports commands with arguments and shell syntax, such as `systemctl poweroff`, `ls -la`, or `cat /etc/os-release`. Shift+Enter always executes the typed input instead of the selected desktop entry.

Commands run through `/bin/sh -c` in a detached child process. Output is not displayed by xuzzel. Shell built-ins such as `cd` affect only that child process; use a terminal command when interactive output or a persistent working directory is needed.

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

Choose matching mode with `match-mode=exact`, `match-mode=fzf`, or `match-mode=fuzzy`. Fuzzy mode is case-insensitive, handles UTF-8 input, splits queries into space-separated tokens, and uses bounded Levenshtein substring distance. Tune it with `fuzzy-min-length`, `fuzzy-max-length-discrepancy`, and `fuzzy-max-distance`.

Icons are enabled by default in launcher mode. `icon-theme` selects a freedesktop theme and `icon-size` sets row icon size in pixels; size `0` derives it from row height. Theme inheritance, `hicolor`, `/usr/share/pixmaps`, absolute paths, PNG, and SVG are supported.

Scaled icon rasters are cached as PNG files under `${XDG_CACHE_HOME:-$HOME/.cache}/xuzzel/icons`. Cache keys include the resolved source path, source modification time and size, requested pixel size, and cache format version, so changed icons are regenerated automatically. Cache failures or corrupt entries are ignored and regenerated when possible. The `--cache` option controls launcher history only and does not relocate this icon cache.

## Implemented

- XDG desktop discovery, launch, terminal entries, hidden/no-display handling
- PNG/SVG launcher icons with freedesktop theme inheritance and fallbacks
- dmenu stdin/stdout, prompt/password/search/select/auto-select modes
- fzf-style subsequence, exact, and Unicode Levenshtein fuzzy matching; source-order mode
- persistent launcher history under XDG cache
- fuzzel/xuzzel INI search, validation and command-line overrides
- fuzzel 1.15 short/long option-name surface, NUL input, index output
- Xft/fontconfig text and Cairo/Xlib icon rendering, keyboard/mouse editing, mouse-disable configuration, clipboard paste
- outside-click cancellation with pointer cleanup
- typed shell-command fallback and Shift+Enter command execution
- Xinerama monitor index, anchors/margins, server/Xft DPI behavior
- colors, spacing, borders, sorted/source-order results, match counter, minimal/hide modes

## Known gaps

X11 has no exact layer-shell, namespace, compositor-output-name, Wayland IME, fractional-scale protocol, blur, gamma-correct Xft blending, or keyboard-focus-loss equivalent. Alpha, action menus, desktop actions, and message wrapping/expansion remain incomplete. Window rounding is delegated to an X11 compositor such as Picom. Compatibility-only options are accepted where documented by `xuzzel(1)`.

## License

MIT/X Consortium license. Drawing primitives and event-loop design retain dmenu attribution. Fuzzel source is not copied; its documented behavior and configuration define compatibility goals. See `LICENSE`.

## License and attribution

See `LICENSE`. Drawing primitives and event-loop design retain dmenu 5.4 lineage and notices (Copyright 2006-2024 suckless.org). Fuzzel source is not copied; 1.15.0 manuals and behavior define compatibility goals.
