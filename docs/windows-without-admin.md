# Rust on Windows without admin rights

Developing freakymate on Windows without admin rights, without WSL/Docker, with rust-analyzer in VS Code.

This document covers the Windows setup only (PowerShell scripts `scripts\*.ps1`). On WSL2 with Arch Linux, the
bash counterparts apply instead: `scripts/setup-linux.sh` checks the environment, `scripts/ui-preview.sh` and
`scripts/ui-snapshot.sh` replace the PowerShell scripts. Nothing in this document applies there.

## Setup

1. Install Rust via `rustup` (works without admin, default toolchain does not matter).
2. Run `powershell -File scripts\setup.ps1` and restart VS Code completely.
3. Check: `cargo test --workspace`; start: task "run: freakymate (fake Moonraker)" or debug configuration "debug: freakymate (fake Moonraker)".

The first build downloads and compiles Slint with all dependencies and takes a few minutes.

## How it works

The repository contains nothing machine-specific, so it also builds on Linux and in GitHub Actions (MSVC there). Everything specific to this machine is done by `scripts\setup.ps1` outside the repository.

| Part                    | Purpose                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `rust-toolchain.toml`   | Platform neutral: `stable` with Clippy, rustfmt and rust-src.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| `scripts/setup.ps1`     | Sets the rustup default host to `x86_64-pc-windows-gnu` (so `stable` resolves to GNU locally; MSVC would need `link.exe` and admin rights). Puts a portable MinGW ([w64devkit](https://github.com/skeeto/w64devkit)) into `%USERPROFILE%\tools`, adds it to the user PATH and creates an empty `libgcc_eh.a` (missing there, but required by Rust when linking). Sets `CARGO_HTTP_CHECK_REVOKE=false` (user environment variable): a proxy without certificate revocation checks otherwise makes crate downloads fail (`CRYPT_E_NO_REVOCATION_CHECK`). Enables `rust-lld` from the Rust toolchain as linker in `%USERPROFILE%\.cargo\config.toml` (see [Build times](#build-times)). |
| `.vscode/settings.json` | rust-analyzer checks with Clippy; SVG files are not formatted on save (icon rules).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |

The key is the linker: w64devkit provides `gcc`/`ld` as a plain user-directory tool. This also allows compiling build scripts and proc macros, which rust-analyzer depends on.

All downloaded tools live in `%USERPROFILE%\tools` (no admin rights, outside the repository):

| Folder                                               | Downloaded by                         | Purpose                                      |
| ---------------------------------------------------- | ------------------------------------- | -------------------------------------------- |
| `w64devkit` (version in `setup.ps1`)                 | `scripts\setup.ps1`                   | `gcc` as linker driver, `ar`                 |
| `slint-viewer-<version>` (version from `Cargo.lock`) | `scripts\ui-preview.ps1` on first use | renders the UI previews without a Rust build |

A `slint-viewer` on `PATH` takes precedence. After a Slint update the preview script downloads the matching version
next to the old one; old folders can be deleted.

## Build times

After a change to the UI most of the time goes into two steps that use only one CPU core each, so the machine looks
idle (one `rustc.exe` at about 10 %):

- type checking the crate `freakymate`, which contains the Rust code Slint generates from `ui/*.slint`
  (stable rustc checks a crate single-threaded; about 10 s, up to 50 s after larger UI changes),
- linking `freakymate.exe` and `ui-snapshot.exe` (each single-threaded).

Linking with w64devkit's `ld` took about 21 s per binary, with `rust-lld` about 11 s (incremental build 38 s → 25 s).
`scripts\setup.ps1` therefore adds to `%USERPROFILE%\.cargo\config.toml`:

```toml
[target.x86_64-pc-windows-gnu]
rustflags = ['-C', 'link-arg=-fuse-ld=lld', '-C', 'link-arg=-B<rustup>\toolchains\stable-x86_64-pc-windows-gnu\lib\rustlib\x86_64-pc-windows-gnu\bin\gcc-ld']
```

`gcc` still drives the link (start files, MinGW libraries); `-B` makes it find the `ld.lld` shipped with Rust.
Changing `rustflags` rebuilds everything once (about 5 min). Windows Defender also scans every new file in `target\`;
an exclusion for that folder helps further but needs admin rights.

`powershell -File scripts\ui-snapshot.ps1` builds `fake-moonraker` and `ui-snapshot`, so its first run after a UI
change takes as long as such a build; rendering itself takes a few seconds. It prints the PNG paths only at the end.

## Pitfalls

- `cannot find -lshlwapi` when linking: `%USERPROFILE%\.cargo\config.toml` contains `-C link-self-contained=yes`. Remove that line; w64devkit plus the empty `libgcc_eh.a` are enough.
- Skia as Slint renderer needs MSVC; the project therefore uses FemtoVG and the software renderer.
- Checking the UI without a window: `powershell -File scripts\ui-snapshot.ps1 [-OutDir <folder>] [-Size 640x400] [-Profile basic|voron|happy-hare|ercf] [-Scenes pages,network|all]` starts a fake Moonraker and renders the pages to PNG (default folder `%TEMP%\kt-ui-snapshots`).
- Linker errors after a Rust update mentioning `gcc-ld`: run `scripts\setup.ps1` again or remove the `[target.x86_64-pc-windows-gnu]` section from `%USERPROFILE%\.cargo\config.toml`.
