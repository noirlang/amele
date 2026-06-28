#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${DIST_DIR:-$ROOT_DIR/dist}"
STAGE_DIR="$DIST_DIR/linux-package-root"
PACKAGE_NAME="worm-forensic-tool"
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT_DIR/Cargo.toml" | head -n1)"

if [[ -z "$VERSION" ]]; then
  echo "Cargo.toml version could not be detected" >&2
  exit 1
fi

require_tool() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Required tool not found: $1" >&2
    exit 1
  fi
}

prepare_stage() {
  if [[ ! -x "$ROOT_DIR/target/release/worm" ]]; then
    cargo build --release --locked
  fi

  rm -rf "$STAGE_DIR"
  mkdir -p \
    "$STAGE_DIR/usr/bin" \
    "$STAGE_DIR/usr/share/worm/ui" \
    "$STAGE_DIR/usr/share/worm/tools" \
    "$STAGE_DIR/usr/share/worm/vendor" \
    "$STAGE_DIR/usr/share/applications" \
    "$STAGE_DIR/usr/share/icons/hicolor/256x256/apps"

  install -m 755 "$ROOT_DIR/target/release/worm" "$STAGE_DIR/usr/bin/worm-forensic-tool"
  ln -s worm-forensic-tool "$STAGE_DIR/usr/bin/worm"
  cp -a "$ROOT_DIR/ui/." "$STAGE_DIR/usr/share/worm/ui/"
  cp -a "$ROOT_DIR/tools/." "$STAGE_DIR/usr/share/worm/tools/"
  cp -a "$ROOT_DIR/vendor/volatility3" "$STAGE_DIR/usr/share/worm/vendor/"
  install -m 644 "$ROOT_DIR/packaging/appimage/worm.desktop" \
    "$STAGE_DIR/usr/share/applications/worm.desktop"
  install -m 644 "$ROOT_DIR/ui/assets/logo/icon.png" \
    "$STAGE_DIR/usr/share/icons/hicolor/256x256/apps/worm.png"
}

build_deb() {
  require_tool dpkg-deb

  local root="$DIST_DIR/deb-root"
  rm -rf "$root"
  mkdir -p "$root/DEBIAN"
  cp -a "$STAGE_DIR/." "$root/"

  cat >"$root/DEBIAN/control" <<EOF
Package: $PACKAGE_NAME
Version: $VERSION
Section: utils
Priority: optional
Architecture: amd64
Maintainer: favilances <favilances@proton.me>
Homepage: https://worm.noirlang.tr
Depends: libc6 (>= 2.35), libgtk-3-0, libwebkit2gtk-4.1-0, python3, curl, util-linux
Description: Desktop forensic acquisition tool
 Worm Forensic Tool collects disk, RAM, and Android evidence for authorized forensic workflows.
EOF

  dpkg-deb --root-owner-group --build "$root" "$DIST_DIR/worm-linux-x64.deb"
}

build_rpm() {
  require_tool rpmbuild

  local rpm_top="$DIST_DIR/rpmbuild"
  local spec="$rpm_top/SPECS/$PACKAGE_NAME.spec"
  rm -rf "$rpm_top"
  mkdir -p "$rpm_top/BUILD" "$rpm_top/BUILDROOT" "$rpm_top/RPMS" "$rpm_top/SOURCES" "$rpm_top/SPECS" "$rpm_top/SRPMS"

  cat >"$spec" <<'EOF'
Name: worm-forensic-tool
Version: %{_worm_version}
Release: 1%{?dist}
Summary: Desktop forensic acquisition tool
License: GPL-3.0-or-later
URL: https://worm.noirlang.tr
BuildArch: x86_64
Requires: gtk3
Requires: webkit2gtk4.1
Requires: python3
Requires: curl
Requires: util-linux

%description
Worm Forensic Tool collects disk, RAM, and Android evidence for authorized forensic workflows.

%prep

%build

%install
mkdir -p %{buildroot}
cp -a %{_worm_stagedir}/* %{buildroot}/

%files
/usr/bin/worm
/usr/bin/worm-forensic-tool
/usr/share/worm
/usr/share/applications/worm.desktop
/usr/share/icons/hicolor/256x256/apps/worm.png
EOF

  rpmbuild -bb "$spec" \
    --define "_topdir $rpm_top" \
    --define "_worm_version $VERSION" \
    --define "_worm_stagedir $STAGE_DIR"

  local built
  built="$(find "$rpm_top/RPMS" -type f -name '*.rpm' | head -n1)"
  if [[ -z "$built" ]]; then
    echo "RPM output was not produced" >&2
    exit 1
  fi
  cp "$built" "$DIST_DIR/worm-linux-x64.rpm"
}

build_arch() {
  require_tool bsdtar
  require_tool zstd

  local root="$DIST_DIR/arch-root"
  rm -rf "$root"
  mkdir -p "$root"
  cp -a "$STAGE_DIR/." "$root/"

  local installed_size
  installed_size="$(du -sb "$root" | awk '{print $1}')"

  cat >"$root/.PKGINFO" <<EOF
pkgname = $PACKAGE_NAME
pkgbase = $PACKAGE_NAME
pkgver = $VERSION-1
pkgdesc = Desktop forensic acquisition tool
url = https://worm.noirlang.tr
builddate = $(date -u +%s)
packager = favilances <favilances@proton.me>
size = $installed_size
arch = x86_64
license = GPL-3.0-or-later
depend = gtk3
depend = webkit2gtk-4.1
depend = python
depend = curl
depend = util-linux
EOF

  (
    cd "$root"
    bsdtar --format=gnutar --uid 0 --gid 0 --uname root --gname root -cf - .PKGINFO usr \
      | zstd -f -19 -T0 -o "$DIST_DIR/worm-linux-x64.pkg.tar.zst"
  )
}

write_hashes() {
  (
    cd "$DIST_DIR"
    sha256sum \
      worm-linux-x64.AppImage \
      worm-linux-x64.deb \
      worm-linux-x64.rpm \
      worm-linux-x64.pkg.tar.zst \
      >worm-linux-packages.sha256
  )
}

mkdir -p "$DIST_DIR"
prepare_stage
build_deb
build_rpm
build_arch
write_hashes

echo "Linux packages written to $DIST_DIR"
