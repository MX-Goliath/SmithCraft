#!/usr/bin/env bash
set -euo pipefail
image=$(realpath -- "${1:?Usage: smoke-linux.sh path/to/SmithCraft.AppImage}")
version=$(cargo metadata --no-deps --format-version 1 --locked |
    python3 -c 'import json,sys; print(next(p["version"] for p in json.load(sys.stdin)["packages"] if p["name"]=="smithcraft"))')
actual=$(APPIMAGE_EXTRACT_AND_RUN=1 "$image" --version)
[[ "$actual" == "SmithCraft $version" ]] || { echo "Unexpected version: $actual" >&2; exit 1; }
state=$(mktemp -d)
export HOME="$state/home" XDG_CONFIG_HOME="$state/config" XDG_DATA_HOME="$state/data"
export XDG_RUNTIME_DIR="$state/runtime" LIBGL_ALWAYS_SOFTWARE=1 APPIMAGE_EXTRACT_AND_RUN=1
mkdir -p "$HOME" "$XDG_CONFIG_HOME" "$XDG_DATA_HOME" "$XDG_RUNTIME_DIR"
chmod 700 "$XDG_RUNTIME_DIR"
unset WAYLAND_DISPLAY
set +e
xvfb-run -a dbus-run-session -- timeout 15s "$image" --demo > "$state/startup.log" 2>&1
status=$?
set -e
cat "$state/startup.log"
[[ $status == 124 ]] || { echo "GUI exited unexpectedly: $status" >&2; exit 1; }
if grep -Ei 'panicked at|error while loading shared libraries|failed to load' "$state/startup.log"; then
    exit 1
fi
echo 'AppImage version and GUI startup passed (X11, software OpenGL).'
