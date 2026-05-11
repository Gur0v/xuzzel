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

- closer visual parity with `fuzzel`
- better input and keybinding parity
- more complete `.desktop` and dmenu support

## License

See [LICENSE](/home/gurov/Projects/xuzzel/LICENSE:1).
