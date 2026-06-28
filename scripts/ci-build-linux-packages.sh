#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LINUXDEPLOY="/tmp/linuxdeploy-x86_64.AppImage"
RUNTIME_FILE="/tmp/runtime-x86_64"
EXTRACT_DIR="/tmp/appimage_extracted_worm_ci"
GLIBC_CEILING="2.35"

# CI konteynerinde oluşan çıktıları runner kullanıcısına geri verir.
restore_output_owner() {
  if [[ -n "${HOST_UID:-}" && -n "${HOST_GID:-}" ]]; then
    chown -R "$HOST_UID:$HOST_GID" "$ROOT_DIR/dist" "$ROOT_DIR/target" 2>/dev/null || true
  fi
}
trap restore_output_owner EXIT

export DEBIAN_FRONTEND=noninteractive
apt-get update
apt-get install -y --no-install-recommends \
  build-essential \
  ca-certificates \
  curl \
  file \
  git \
  imagemagick \
  libarchive-tools \
  libgtk-3-dev \
  libwebkit2gtk-4.1-dev \
  patchelf \
  perl \
  pkg-config \
  python3 \
  rpm \
  zstd

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
  | sh -s -- -y --profile minimal --default-toolchain stable
export PATH="$HOME/.cargo/bin:$PATH"

curl -fL \
  https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage \
  -o "$LINUXDEPLOY"
curl -fL \
  https://github.com/AppImage/type2-runtime/releases/download/continuous/runtime-x86_64 \
  -o "$RUNTIME_FILE"
chmod +x "$LINUXDEPLOY"

# FUSE gerektirmeden linuxdeploy içeriğini sabit bir CI yoluna açar.
rm -rf "$EXTRACT_DIR" /tmp/squashfs-root
(
  cd /tmp
  "$LINUXDEPLOY" --appimage-extract >/dev/null
)
mv /tmp/squashfs-root "$EXTRACT_DIR"

cd "$ROOT_DIR"
./scripts/build-appimage.sh

# AppImage içindeki bütün ELF dosyalarının eski sistem uyumluluğunu korur.
highest_glibc="$({
  while IFS= read -r -d '' candidate; do
    if file -b "$candidate" | grep -q '^ELF'; then
      readelf --version-info "$candidate" 2>/dev/null \
        | grep -oE 'GLIBC_[0-9.]+' \
        | sed 's/^GLIBC_//' || true
    fi
  done < <(find "$ROOT_DIR/dist/WormForensicTool.AppDir" -type f -print0)
} | sort -Vu | tail -n1)"
if [[ -z "$highest_glibc" ]]; then
  echo "AppImage GLIBC requirement could not be detected" >&2
  exit 1
fi
if dpkg --compare-versions "$highest_glibc" gt "$GLIBC_CEILING"; then
  echo "AppImage requires GLIBC_$highest_glibc; maximum allowed is GLIBC_$GLIBC_CEILING" >&2
  exit 1
fi
echo "AppImage GLIBC requirement verified: GLIBC_$highest_glibc"

./scripts/build-linux-packages.sh
