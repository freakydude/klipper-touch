# Rust on Windows without admin rights

Developing klipper-touch on Windows without admin rights, without WSL/Docker, with rust-analyzer in VS Code.

## Setup

1. Install Rust via `rustup` (works without admin, default toolchain does not matter).
2. Run `powershell -File scripts\setup.ps1` and restart VS Code completely.
3. Check: `cargo test --workspace`; start: task "run: klipper-touch (fake Moonraker)" or debug configuration "debug: klipper-touch (fake Moonraker)".

The first build downloads and compiles Slint with all dependencies and takes a few minutes.

## How it works

The repository contains nothing machine-specific, so it also builds on Linux and in GitHub Actions (MSVC there). Everything specific to this machine is done by `scripts\setup.ps1` outside the repository.

| Part | Purpose |
| --- | --- |
| `rust-toolchain.toml` | Platform neutral: `stable` with Clippy, rustfmt and rust-src. |
| `scripts/setup.ps1` | Sets the rustup default host to `x86_64-pc-windows-gnu` (so `stable` resolves to GNU locally; MSVC would need `link.exe` and admin rights). Puts a portable MinGW ([w64devkit](https://github.com/skeeto/w64devkit)) into `%USERPROFILE%\tools`, adds it to the user PATH and creates an empty `libgcc_eh.a` (missing there, but required by Rust when linking). Sets `CARGO_HTTP_CHECK_REVOKE=false` (user environment variable): a proxy without certificate revocation checks otherwise makes crate downloads fail (`CRYPT_E_NO_REVOCATION_CHECK`). |
| `.vscode/settings.json` | rust-analyzer checks with Clippy; SVG files are not formatted on save (icon rules). |

The key is the linker: w64devkit provides `gcc`/`ld` as a plain user-directory tool. This also allows compiling build scripts and proc macros, which rust-analyzer depends on.

## Pitfalls

- `cannot find -lshlwapi` when linking: `%USERPROFILE%\.cargo\config.toml` contains `-C link-self-contained=yes`. Remove that line; w64devkit plus the empty `libgcc_eh.a` are enough.
- Skia as Slint renderer needs MSVC; the project therefore uses FemtoVG and the software renderer.
- Checking the UI without a window: `cargo run -p klipper-touch --bin ui-snapshot` against a running fake, more conveniently `powershell -File scripts\ui-snapshot.ps1`.
