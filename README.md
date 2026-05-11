# xuzzel

`xuzzel` is a X11 application launcher inspired by
[`fuzzel`](https://codeberg.org/dnkl/fuzzel).

Current features:

- `.desktop` launcher discovery
- fuzzy matching
- launch history
- `dmenu` mode
- X11 popup UI
- desktop icons
- TOML config

## Status

Completed:

- X11 popup window with floating/above-window hints
- keyboard-driven filtering and selection
- mouse selection and scroll wheel navigation
- rounded window radius via the X Shape extension
- `.desktop` launcher discovery from XDG application directories
- launch history cache
- freedesktop icon lookup and rendering
- TOML-based config loading
- Fontconfig-style font selection through Pango/Cairo
- `dmenu` mode with newline or NUL-separated input
- `dmenu` options like `--index`, `--password`, `--minimal-lines`,
  `--with-nth`, `--accept-nth`, and `--match-nth`
- configurable colors, sizing, padding, prompt, placeholder, and counter

Partially complete:

- visual parity with `fuzzel`
- keybinding parity with `fuzzel`
- `.desktop` execution fidelity
- `dmenu` compatibility
- window manager focus behavior

Missing or planned:

- exact `fuzzel` layout and font metrics
- exact border rendering to match rounded corners
- message wrapping and more exact text layout behavior
- full `fuzzel.ini` keybinding coverage
- richer text editing behavior
- desktop actions support
- filtering based on `OnlyShowIn` and `NotShowIn`
- localized `.desktop` strings
- startup notification support
- launch prefix support
- listing executables from `$PATH`
- execute-input behavior outside current partial support
- Rofi icon protocol support in `dmenu` mode
- optional large icon preview for small result sets
- broader UTF-8, emoji, IME, and clipboard/paste parity
- multi-monitor placement logic closer to `fuzzel`
- more aggressive performance work for huge lists

## Build

```sh
cargo build --release
```

`xuzzel` links against `libX11`, which must be installed on the target system.

## Run

```sh
./target/release/xuzzel
```

To use stdin mode:

```sh
printf 'firefox\nfoot\nthunderbird\n' | ./target/release/xuzzel --dmenu
```

## Config

`xuzzel` looks for config in:

- `$XDG_CONFIG_HOME/xuzzel/xuzzel.toml`
- `~/.config/xuzzel/xuzzel.toml`

`font` now uses a Fontconfig-style font description, for example
`monospace`, `Iosevka 11`, or `JetBrains Mono 10`.

## Roadmap

- finish visual parity with `fuzzel`
- finish feature parity with `fuzzel`
- keep the X11/Rust implementation fast and maintainable

## License

See [LICENSE](/home/gurov/Projects/xuzzel/LICENSE:1).
