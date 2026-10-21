#!/bin/bash
# Packt die heruntergeladenen Build-Artefakte (ein Ordner je Ziel) zu Release-Zips und baut je Linux-Ziel ein .deb.
# Linux-Pakete bekommen release_info.json fuer den Moonraker-Update-Manager (type: zip).
# Aufruf im Ordner mit den Artefakt-Unterordnern: pack-artifacts.sh <release-tag>
set -euo pipefail

version="$1"

for directory in */; do
  name="${directory%/}"
  if [[ "$name" != *windows* ]]; then
    cat > "$name/release_info.json" << EOF
{
  "project_name": "klipper-touch",
  "project_owner": "freakydude",
  "version": "$version",
  "asset_name": "$name.zip"
}
EOF
    # upload-artifact verliert die Ausfuehrungsrechte; Moonraker uebernimmt sie beim Entpacken aus dem Zip.
    chmod 0755 "$name/klipper-touch" "$name/install.sh" "$name/uninstall.sh"
    bash "$(dirname "${BASH_SOURCE[0]}")/build-deb.sh" "$name" "$version"
  fi
  (cd "$name" && zip -qr "../$name.zip" .)
done
ls -l ./*.zip ./*.deb
