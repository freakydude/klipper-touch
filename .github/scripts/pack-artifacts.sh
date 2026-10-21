#!/bin/bash
# Packt die heruntergeladenen Build-Artefakte (ein Ordner je Ziel) zu Release-Archiven:
# Windows als .zip, Linux als .tar.gz. Aufruf im Ordner mit den Artefakt-Unterordnern.
set -euo pipefail

for directory in */; do
  name="${directory%/}"
  if [[ "$name" == *windows* ]]; then
    (cd "$name" && zip -qr "../$name.zip" .)
  else
    tar -czf "$name.tar.gz" -C "$name" .
  fi
done
ls -l ./*.zip ./*.tar.gz
