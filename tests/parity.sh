#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
export ASAN_OPTIONS=${ASAN_OPTIONS:-detect_leaks=0}

fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }

./xuzzel --version | grep -q '^xuzzel 1\.15\.0-x11\.1$' || fail version
./xuzzel --help | grep -q -- '--dmenu' || fail help
./xuzzel --config ./xuzzel.ini --check-config

# Keep default sections complete and prevent key bindings from replacing colors.
grep -q '^\[colors\]$' xuzzel.ini || fail default-colors-section
grep -q '^# background=fdf6e3ff$' xuzzel.ini || fail default-background
grep -q '^# border=002b36ff$' xuzzel.ini || fail default-border-color
grep -q '^\[border\]$' xuzzel.ini || fail default-border-section
grep -q '^\[dmenu\]$' xuzzel.ini || fail default-dmenu-section
grep -q '^\[key-bindings\]$' xuzzel.ini || fail default-key-bindings-section
grep -q '^# cancel=Escape Control+g Control+c Control+bracketleft$' xuzzel.ini || fail default-cancel-binding
grep -q '^# custom-19=Mod1+parentleft$' xuzzel.ini || fail default-custom-binding

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT HUP INT TERM
mkdir -p "$tmp/fuzzel"
printf '[main]\nlines=7\nmatch-mode=fzf\n[colors]\nbackground=112233ff\n' > "$tmp/fuzzel/fuzzel.ini"
XDG_CONFIG_HOME="$tmp" ./xuzzel --check-config
./xuzzel --config /dev/null --check-config --icon-size=24

# Resolve and decode themed PNG/SVG icons without requiring an X server.
mkdir -p "$tmp/data/icons/parent/16x16/apps" "$tmp/data/icons/child" "$tmp/data/icons/hicolor/scalable/apps" "$tmp/data/pixmaps"
cat > "$tmp/data/icons/parent/index.theme" <<EOF
[Icon Theme]
Name=Parent
Directories=16x16/apps
[16x16/apps]
Size=16
Type=Fixed
EOF
cat > "$tmp/data/icons/child/index.theme" <<EOF
[Icon Theme]
Name=Child
Inherits=parent
Directories=
EOF
cat > "$tmp/data/icons/hicolor/index.theme" <<EOF
[Icon Theme]
Name=Hicolor
Directories=scalable/apps
[scalable/apps]
Size=48
Type=Scalable
MinSize=1
MaxSize=256
EOF
# Valid 16x16 RGBA PNG.
printf '%s' 'iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAABmJLR0QA/wD/AP+gvaeTAAAAMklEQVQ4je3MsQ0AIAACQXD/XbTU6d5Sa2o+oeQELPKmAWwrCdCInl8FChR4wAGUTNK+KN5xgmeR0lYAAAAASUVORK5CYII=' | base64 -d > "$tmp/data/icons/parent/16x16/apps/tiny.png"
cp "$tmp/data/icons/parent/16x16/apps/tiny.png" "$tmp/data/pixmaps/fallback.png"
cat > "$tmp/data/icons/hicolor/scalable/apps/vector.svg" <<EOF
<svg xmlns="http://www.w3.org/2000/svg" width="8" height="4"><rect width="8" height="4" fill="#f00"/></svg>
EOF
XDG_DATA_HOME="$tmp/data" XDG_DATA_DIRS= ./xuzzel --icon-probe child tiny 24 || fail icon-theme-inheritance-png
XDG_DATA_HOME="$tmp/data" XDG_DATA_DIRS= ./xuzzel --icon-probe child vector 24 || fail icon-hicolor-svg
XDG_DATA_HOME="$tmp/data" XDG_DATA_DIRS= ./xuzzel --icon-probe child fallback 24 || fail icon-pixmaps
./xuzzel --icon-probe child "$tmp/data/icons/parent/16x16/apps/tiny.png" 24 || fail icon-absolute-png
XDG_CONFIG_HOME="$tmp" ./xuzzel --check-config --override lines=3 --override colors.selection=aabbcc
# Quoted prompt values must parse without retaining their delimiter quotes.
printf '[main]\nprompt="> "\n[colors]\nselection=aabbcc\n[border]\nwidth=2\n' > "$tmp/sections.ini"
./xuzzel --config "$tmp/sections.ini" --check-config

# Implemented fuzzel polarity and legacy no-sort compatibility both validate.
printf '[main]\nsort-result=no\nenable-mouse=no\n' > "$tmp/behavior.ini"
./xuzzel --config "$tmp/behavior.ini" --check-config
printf '[main]\nno-sort=yes\n' > "$tmp/no-sort.ini"
./xuzzel --config "$tmp/no-sort.ini" --check-config

# --auto-select exits before opening X when filtering leaves one result.
fuzzy_one() {
  expected=$1 search=$2; shift 2
  got=$(printf '%s\n' "$@" | ./xuzzel --dmenu --auto-select --match-mode=fuzzy --search "$search")
  [ "$got" = "$expected" ] || fail "fuzzy '$search': expected '$expected', got '$got'"
}
fuzzy_none() {
  search=$1; shift
  if DISPLAY= printf '%s\n' "$@" | DISPLAY= ./xuzzel --dmenu --auto-select --match-mode=fuzzy --search "$search" >/dev/null 2>&1; then
    fail "fuzzy '$search': unexpected match"
  fi
}
fuzzy_one 'Mozilla Firefox' 'mozila' 'Mozilla Firefox' 'Chromium'
fuzzy_none 'mozlila' 'Mozilla Firefox' 'Chromium'
fuzzy_one 'Alpha Beta Tool' 'ALPA   beta' 'Alpha Beta Tool' 'Alpha Gamma Tool'
fuzzy_one 'Éclair Editor' 'éclir' 'Éclair Editor' 'Terminal'
# Distance zero leaves the exact substring as the sole auto-select candidate.
got=$(printf '%s\n' 'exact needle' 'needlx' | ./xuzzel --dmenu --auto-select \
  --match-mode=fuzzy --fuzzy-max-distance=0 --search needle)
[ "$got" = 'exact needle' ] || fail "fuzzy exact preference: got '$got'"
fuzzy_one 'abcdef' 'abqdef' 'abcdef' 'abzzef'

# All three fuzzy limits are active configuration, not compatibility no-ops.
printf '[main]\nmatch-mode=fuzzy\nfuzzy-min-length=7\nfuzzy-max-length-discrepancy=0\nfuzzy-max-distance=0\n' > "$tmp/fuzzy.ini"
./xuzzel --config "$tmp/fuzzy.ini" --check-config

# Physically unsupported or unimplemented settings must not be silently accepted.
printf '[main]\ngamma-correct-blending=no\n' > "$tmp/unsupported.ini"
if ./xuzzel --config "$tmp/unsupported.ini" --check-config >/dev/null 2>&1; then fail gamma-config; fi
printf '[main]\nmessage-mode=wrap\n' > "$tmp/unsupported.ini"
if ./xuzzel --config "$tmp/unsupported.ini" --check-config >/dev/null 2>&1; then fail message-mode-config; fi
if ./xuzzel --config /dev/null --dmenu-message-mode=wrap --check-config >/dev/null 2>&1; then fail message-mode-cli; fi

out=$(printf 'single\n' | ./xuzzel --config /dev/null --dmenu --auto-select)
[ "$out" = single ] || fail dmenu-output
out=$(printf 'zero\0one\0' | ./xuzzel --config /dev/null --dmenu0 --auto-select --search=one --index)
[ "$out" = 1 ] || fail dmenu0-index
out=$(printf 'id:Visible\n' | ./xuzzel --config /dev/null --dmenu --auto-select \
  --dmenu-nth-delimiter=: --dmenu-with-nth=2 --dmenu-accept-nth=1)
[ "$out" = id ] || fail dmenu-nth

if ./xuzzel --config "$tmp/missing" --check-config >/dev/null 2>&1; then fail missing-config; fi
printf 'bad line\n' > "$tmp/bad.ini"
if ./xuzzel --config "$tmp/bad.ini" --check-config >/dev/null 2>&1; then fail bad-config; fi

# Option parser parity smoke test: all non-display flags used by scripts must parse.
./xuzzel --check-config --config /dev/null --namespace=x --cache="$tmp/cache" \
  --output=0 --font=monospace --use-bold --dpi-aware=auto --icon-theme=hicolor \
  --no-icons --fields=name --password='*' --anchor=center --x-margin=1 --y-margin=1 \
  --select=x --select-index=0 --lines=2 --minimal-lines --hide-prompt --width=20 \
  --tabs=4 --horizontal-pad=1 --vertical-pad=1 --inner-pad=1 --background-color=000000 \
  --text-color=ffffff --prompt-color=ffffff --input-color=ffffff --match-color=ffffff \
  --selection-color=ffffff --selection-text-color=000000 --selection-match-color=000000 \
  --counter-color=ffffff --border-width=1 --border-color=ffffff \
  --match-mode=exact --no-sort --counter --fuzzy-min-length=3 \
  --fuzzy-max-length-discrepancy=2 --fuzzy-max-distance=1 --line-height=20 \
  --letter-spacing=0 --launch-prefix=env --dmenu --index --dmenu-match-nth=1 \
  --dmenu-with-nth=1 --dmenu-accept-nth=1 --dmenu-nth-delimiter=: --dmenu-only-match \
  --auto-select --dmenu-message=hello --no-mouse --search=x --log-level=none

printf 'ok\n'
