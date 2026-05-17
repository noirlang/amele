#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APPDIR="$ROOT_DIR/dist/WormForensicTool.AppDir"
APPIMAGE="$ROOT_DIR/dist/worm-linux-x64.AppImage"
LINUXDEPLOY="${LINUXDEPLOY:-/tmp/linuxdeploy-x86_64.AppImage}"
RUNTIME_FILE="${RUNTIME_FILE:-/tmp/runtime-x86_64}"

if [[ ! -x "$LINUXDEPLOY" ]]; then
  echo "linuxdeploy not found: $LINUXDEPLOY" >&2
  echo "Set LINUXDEPLOY=/path/to/linuxdeploy-x86_64.AppImage" >&2
  exit 1
fi

LINUXDEPLOY_RUN="$(find /tmp -maxdepth 1 -type d -name 'appimage_extracted_*' -printf '%T@ %p\n' | sort -nr | awk 'NR==1 {print $2 "/AppRun"}')"
if [[ ! -x "$LINUXDEPLOY_RUN" ]]; then
  APPIMAGE_EXTRACT_AND_RUN=1 "$LINUXDEPLOY" --help >/dev/null || true
  LINUXDEPLOY_RUN="$(find /tmp -maxdepth 1 -type d -name 'appimage_extracted_*' -printf '%T@ %p\n' | sort -nr | awk 'NR==1 {print $2 "/AppRun"}')"
fi
if [[ ! -x "$LINUXDEPLOY_RUN" ]]; then
  echo "linuxdeploy could not be extracted: $LINUXDEPLOY" >&2
  exit 1
fi
LINUXDEPLOY_ROOT="$(cd "$(dirname "$LINUXDEPLOY_RUN")" && pwd)"
APPIMAGETOOL="${APPIMAGETOOL:-$LINUXDEPLOY_ROOT/plugins/linuxdeploy-plugin-appimage/usr/bin/appimagetool}"
if [[ ! -x "$APPIMAGETOOL" ]]; then
  echo "appimagetool not found: $APPIMAGETOOL" >&2
  exit 1
fi
if [[ ! -f "$RUNTIME_FILE" ]]; then
  echo "AppImage runtime not found: $RUNTIME_FILE" >&2
  echo "Download it from https://github.com/AppImage/type2-runtime/releases/download/continuous/runtime-x86_64" >&2
  exit 1
fi

cargo build --release --locked

rm -rf "$APPDIR" "$APPIMAGE"
mkdir -p \
  "$APPDIR/usr/bin" \
  "$APPDIR/usr/share/worm/ui" \
  "$APPDIR/usr/share/applications" \
  "$APPDIR/usr/share/icons/hicolor/256x256/apps"

install -m 755 "$ROOT_DIR/target/release/worm" "$APPDIR/usr/bin/worm"
cp -a "$ROOT_DIR/ui/." "$APPDIR/usr/share/worm/ui/"
install -m 755 "$ROOT_DIR/packaging/appimage/AppRun" "$APPDIR/AppRun"
install -m 644 "$ROOT_DIR/packaging/appimage/worm.desktop" "$APPDIR/worm.desktop"
install -m 644 "$ROOT_DIR/packaging/appimage/worm.desktop" "$APPDIR/usr/share/applications/worm.desktop"
if command -v magick >/dev/null 2>&1; then
  magick "$ROOT_DIR/ui/assets/logo/icon.png" -resize 256x256 "$APPDIR/usr/share/icons/hicolor/256x256/apps/worm.png"
elif command -v convert >/dev/null 2>&1; then
  convert "$ROOT_DIR/ui/assets/logo/icon.png" -resize 256x256 "$APPDIR/usr/share/icons/hicolor/256x256/apps/worm.png"
else
  install -m 644 "$ROOT_DIR/ui/assets/logo/icon.png" "$APPDIR/usr/share/icons/hicolor/256x256/apps/worm.png"
fi
install -m 644 "$APPDIR/usr/share/icons/hicolor/256x256/apps/worm.png" "$APPDIR/worm.png"

NO_STRIP="${NO_STRIP:-1}" STRIP="${STRIP:-/usr/bin/strip}" OUTPUT="$APPIMAGE" "$LINUXDEPLOY_RUN" \
  --appdir "$APPDIR" \
  --executable "$APPDIR/usr/bin/worm" \
  --desktop-file "$APPDIR/worm.desktop" \
  --icon-file "$APPDIR/worm.png" \
  --custom-apprun "$ROOT_DIR/packaging/appimage/AppRun"
"$APPIMAGETOOL" --runtime-file "$RUNTIME_FILE" "$APPDIR" "$APPIMAGE"
chmod +x "$APPIMAGE"
echo "$APPIMAGE"
