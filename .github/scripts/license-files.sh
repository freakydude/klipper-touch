#!/bin/bash
# Copies LICENSE and generates THIRD-PARTY-LICENSES.html for the release binary of one target into a package folder.
# Needs cargo-about. Usage: license-files.sh <target-triple> <folder>
set -euo pipefail

target="$1"
folder="$2"
repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

mkdir -p "$folder"
cp "$repo/LICENSE" "$folder/"
# Same features as the release build (--no-default-features), so the list matches what is compiled in.
cargo about generate \
  --config "$repo/about.toml" \
  --manifest-path "$repo/crates/app/Cargo.toml" \
  --no-default-features \
  --target "$target" \
  --locked \
  --fail \
  --output-file "$folder/THIRD-PARTY-LICENSES.html" \
  "$repo/about.hbs"
