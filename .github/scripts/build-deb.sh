#!/bin/bash
# Baut aus einem Linux-Build-Artefakt ein Debian-Paket klipper-touch_<version>_<arch>.deb.
# Aufruf: build-deb.sh <artefakt-ordner> <release-tag>  (im Ordner mit den Artefakten, Repo eine Ebene hoeher)
set -euo pipefail

artifact="$1"
release="$2"
repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

case "$artifact" in
  *x86_64*) arch=amd64 ;;
  *aarch64*) arch=arm64 ;;
  *armv7*) arch=armhf ;;
  *) echo "unknown architecture in '$artifact'"; exit 1 ;;
esac

# Debian-Versionen muessen mit einer Ziffer beginnen; Vorabversionen sortieren per ~ vor dem Release.
if [[ "$release" =~ ^v[0-9] ]]; then
  version=$(echo "${release#v}" | tr '-' '~')
else
  cargo_version=$(sed -n 's/^version = "\(.*\)"/\1/p' "$repo/Cargo.toml" | head -n 1)
  version="${cargo_version}~${release//-/.}"
fi

depends=$(python3 -c 'import json, sys; print(", ".join(["libc6 (>= 2.36)", "libfontconfig1"] + json.load(open(sys.argv[1]))["debian"]))' \
  "$repo/scripts/system-dependencies.json")

root=$(mktemp -d)
install -D -m 0755 "$artifact/klipper-touch" "$root/usr/bin/klipper-touch"
install -D -m 0644 "$repo/scripts/debian/klipper-touch@.service" "$root/lib/systemd/system/klipper-touch@.service"
install -D -m 0644 "$repo/LICENSE" "$root/usr/share/doc/klipper-touch/copyright"
install -d "$root/DEBIAN"
install -m 0755 "$repo/scripts/debian/postinst" "$repo/scripts/debian/prerm" "$repo/scripts/debian/postrm" "$root/DEBIAN/"

cat > "$root/DEBIAN/control" << EOF
Package: klipper-touch
Version: $version
Architecture: $arch
Maintainer: Frank Roth <developer@freakydu.de>
Depends: $depends
Section: misc
Priority: optional
Homepage: https://github.com/freakydude/klipper-touch
Description: Touch screen UI for Klipper printers via Moonraker (experimental)
 Native Rust + Slint UI for small displays on the printer host.
 UNTESTED EXPERIMENT: use at your own risk, no warranty, no liability.
 Start in kiosk mode on tty1 with: systemctl enable --now klipper-touch@<user>
EOF

dpkg-deb --root-owner-group --build "$root" "klipper-touch_${version}_${arch}.deb"
rm -rf "$root"
