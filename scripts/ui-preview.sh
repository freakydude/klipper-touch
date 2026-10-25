#!/usr/bin/env bash
# Renders the UI previews in crates/app/ui/preview with slint-viewer, without building the app.
# WSL2/Linux counterpart of ui-preview.ps1.
# Usage (repo root): scripts/ui-preview.sh [-n print-printing,temperature] [-o <folder>]
# Uses slint-viewer from PATH; otherwise compiles the version matching Cargo.lock into ~/.local/bin (first run only).
# Exit code 0 on success; the PNG paths are printed.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
preview_dir="$root/crates/app/ui/preview"
out_dir="${TMPDIR:-/tmp}/kt-ui-preview"
names=()

while getopts "n:o:h" option; do
    case $option in
        n) IFS=, read -ra names <<< "$OPTARG" ;;
        o) out_dir=$OPTARG ;;
        *) sed -n '2,6p' "$0"; exit 2 ;;
    esac
done

slint_version() {
    awk -F'"' '/^name = "slint"$/ { getline; print $2; exit }' "$root/Cargo.lock"
}

find_viewer() {
    if command -v slint-viewer > /dev/null; then
        command -v slint-viewer
        return
    fi
    local version
    version=$(slint_version)
    echo "slint-viewer $version not found, compiling it (first run only)..." >&2
    cargo install slint-viewer --version "$version" --locked --root "$HOME/.local" >&2
    echo "$HOME/.local/bin/slint-viewer"
}

selected() {
    [[ ${#names[@]} -eq 0 ]] && return 0
    printf '%s\n' "${names[@]}" | grep -qx -- "$1"
}

viewer=$(find_viewer)
mkdir -p "$out_dir"
count=0
failed=0
for preview in "$preview_dir"/*.slint; do
    name=$(basename "$preview" .slint)
    [[ $name == sample ]] && continue
    selected "$name" || continue
    count=$((count + 1))
    png="$out_dir/$name.png"
    if "$viewer" --screenshot "$png" "$preview"; then
        echo "$png"
    else
        failed=$((failed + 1))
        echo "WARN $name.slint failed" >&2
    fi
done

if [[ $count -eq 0 ]]; then
    echo "no preview found in $preview_dir" >&2
    exit 1
fi
exit "$((failed > 0))"
