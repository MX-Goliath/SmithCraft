#!/usr/bin/env bash
set -euo pipefail
repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo"
if [[ $(uname -s) != Linux || $(uname -m) != x86_64 ]]; then
    echo 'Build the AppImage on x86-64 Linux (Ubuntu 22.04 is used in CI).' >&2
    exit 1
fi
case ${1:-} in
    '') cargo build --release --locked --target x86_64-unknown-linux-gnu ;;
    --skip-build) ;;
    *) echo 'Usage: bash tools/package-linux.sh [--skip-build]' >&2; exit 2 ;;
esac
metadata=$(cargo metadata --no-deps --format-version 1 --locked)
version=$(python3 -c 'import json,sys; print(next(p["version"] for p in json.load(sys.stdin)["packages"] if p["name"]=="smithcraft"))' <<< "$metadata")
target_dir=$(python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])' <<< "$metadata")
binary="$target_dir/x86_64-unknown-linux-gnu/release/smithcraft"
[[ -x "$binary" ]] || { echo "Missing release executable: $binary" >&2; exit 1; }
cache="$repo/.tools/appimage"
artifacts="$repo/dist/artifacts"
mkdir -p "$cache" "$artifacts" "$repo/.tools/packages"
stage=$(mktemp -d "$repo/.tools/packages/linux-XXXXXXXX")
appdir="$stage/SmithCraft.AppDir"

# Only immutable upstream releases are used. Validate bytes before execution.
fetch() {
    local url=$1 destination=$2 digest=$3
    if [[ ! -f "$destination" ]] || ! echo "$digest  $destination" | sha256sum --check --status; then
        curl --fail --location --retry 3 --connect-timeout 20 "$url" -o "$destination.download"
        echo "$digest  $destination.download" | sha256sum --check --status
        mv -- "$destination.download" "$destination"
    fi
    chmod +x "$destination"
}
fetch 'https://github.com/linuxdeploy/linuxdeploy/releases/download/1-alpha-20251107-1/linuxdeploy-x86_64.AppImage' \
    "$cache/linuxdeploy.AppImage" c20cd71e3a4e3b80c3483cef793cda3f4e990aca14014d23c544ca3ce1270b4d
fetch 'https://github.com/AppImage/appimagetool/releases/download/1.9.1/appimagetool-x86_64.AppImage' \
    "$cache/appimagetool.AppImage" ed4ce84f0d9caff66f50bcca6ff6f35aae54ce8135408b3fa33abfc3cb384eb0
fetch 'https://github.com/AppImage/type2-runtime/releases/download/20251108/runtime-x86_64' \
    "$cache/runtime-x86_64" 2fca8b443c92510f1483a883f60061ad09b46b978b2631c807cd873a47ec260d

mkdir -p "$appdir/usr/share/doc/smithcraft" "$appdir/usr/share/smithcraft"
cp README.md BUILDING.md LICENSE "$appdir/usr/share/doc/smithcraft/"
cp -R examples help "$appdir/usr/share/smithcraft/"
# Winit loads these libraries at runtime; ldd alone does not discover them.
libraries=()
for library in libxkbcommon.so.0 libxkbcommon-x11.so.0 libwayland-client.so.0 libwayland-cursor.so.0 \
    libwayland-egl.so.1 libX11.so.6 libXi.so.6 libXcursor.so.1 libXrandr.so.2 \
    libxcb.so.1 libxcb-shape.so.0 libxcb-xfixes.so.0; do
    path=$(ldconfig -p | awk -v name="$library" '$1==name && /x86-64/ && !found {print $NF; found=1}')
    [[ -n "$path" ]] || { echo "Missing system library: $library (see BUILDING.md)" >&2; exit 1; }
    libraries+=(--library "$path")
done
export APPIMAGE_EXTRACT_AND_RUN=1
"$cache/linuxdeploy.AppImage" --appdir "$appdir" --executable "$binary" \
    --desktop-file "$repo/packaging/linux/smithcraft.desktop" \
    --icon-file "$repo/packaging/linux/smithcraft.svg" \
    --custom-apprun "$repo/packaging/linux/AppRun" "${libraries[@]}"
chmod +x "$appdir/AppRun"
artifact="$artifacts/SmithCraft-$version-linux-x86_64.AppImage"
ARCH=x86_64 "$cache/appimagetool.AppImage" --runtime-file "$cache/runtime-x86_64" "$appdir" "$artifact"
chmod +x "$artifact"
(cd "$artifacts" && sha256sum "$(basename "$artifact")" > "$(basename "$artifact").sha256")
echo "$artifact"
