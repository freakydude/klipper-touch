#!/usr/bin/env bash
# Checks the development setup in WSL2 with Arch Linux and installs slint-viewer for the UI previews.
# WSL2 counterpart of setup.ps1. Installs nothing system-wide (no sudo): missing packages are printed as pacman commands.
# Usage (repo root): scripts/setup-wsl.sh
set -uo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
missing=0

check_command() {
    if command -v "$1" > /dev/null; then
        echo "ok      $1"
    else
        echo "MISSING $1  ->  sudo pacman -S --needed $2"
        missing=$((missing + 1))
    fi
}

check_library() {
    if pkg-config --exists "$1"; then
        echo "ok      $1 (pkg-config)"
    else
        echo "MISSING $1 (pkg-config)  ->  sudo pacman -S --needed $2"
        missing=$((missing + 1))
    fi
}

check_command cargo rust
check_command rustup rustup
check_command pkg-config pkgconf
check_library fontconfig fontconfig
check_library xkbcommon libxkbcommon
check_library wayland-client wayland
check_library x11 libx11

if command -v rustup > /dev/null; then
    (cd "$root" && rustup show active-toolchain) || missing=$((missing + 1))
fi

slint=$(awk -F'"' '/^name = "slint"$/ { getline; print $2; exit }' "$root/Cargo.lock")
if command -v slint-viewer > /dev/null; then
    echo "ok      slint-viewer $(slint-viewer --version 2>/dev/null | head -1)"
else
    echo "slint-viewer $slint missing, installing into ~/.local (several minutes)..."
    if cargo install slint-viewer --version "$slint" --locked --root "$HOME/.local"; then
        echo "ok      slint-viewer $slint in ~/.local/bin"
    else
        missing=$((missing + 1))
    fi
fi

case ":$PATH:" in
    *":$HOME/.local/bin:"*) ;;
    *) echo "hint    add ~/.local/bin to PATH (e.g. in ~/.bashrc)" ;;
esac

if [[ $missing -eq 0 ]]; then
    echo "Done. Next: scripts/ui-preview.sh, scripts/ui-snapshot.sh, cargo run -p freakymate"
else
    echo "$missing item(s) missing, see above." >&2
    exit 1
fi
