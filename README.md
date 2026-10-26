# Klipper-Touch

> [!CAUTION]
> **Untested experiment. Not intended for productive use.**
> This software has not been tested on real printers. It sends commands that move motors, heat up
> nozzles and beds and restart services; a bug can damage your printer, your print or start a fire.
> Use it only if you know what you are doing, never leave the printer unattended, and keep an
> emergency stop within reach.
> The software is provided "as is", **without any warranty and without any liability** of the author
> for damage of any kind, to the extent permitted by law (see sections 15 and 16 of the [GPL](LICENSE)).

Touch screen UI for [Klipper](https://www.klipper3d.org/) printers, talking to [Moonraker](https://moonraker.readthedocs.io/).
Built for small displays attached directly to the printer host (Raspberry Pi 3/4/5, BTT CB1/CB2), first target is the
BTT HDMI5 (5", 800x480, landscape). Native Rust application with a [Slint](https://slint.dev/) UI — no browser, no WebView.

So far it has run against the included fake Moonraker, on desktops and on the BTT HDMI5.

![Print screen](docs/screenshots/print.png)

|                                      |                                        |                                                            |
| ------------------------------------ | -------------------------------------- | ---------------------------------------------------------- |
| ![Files](docs/screenshots/files.png) | ![Move](docs/screenshots/move.png)     | ![Temperature](docs/screenshots/temperature.png)           |
| ![Tune](docs/screenshots/tune.png)   | ![System](docs/screenshots/system.png) | ![Klipper shut down](docs/screenshots/klippy-shutdown.png) |
| ![Console](docs/screenshots/console.png) | ![Filament](docs/screenshots/filament.png) | ![MMU (Happy Hare)](docs/screenshots/mmu.png)       |

## Features

- **Print**: thumbnail, progress, elapsed/remaining/ETA, layers, filament; start, pause, resume, cancel; file browser with thumbnails.
- **Move**: jog X/Y/Z with selectable steps, home (all/XY/Z), motors off, Quad Gantry Level or Z-Tilt when configured.
- **Heat**: every heater with a target (nozzle, bed, `heater_generic`, `temperature_fan`), sensors, presets, part fan.
  - and − respect `min_temp`/`max_temp` from the Klipper configuration (from off, + jumps straight to `min_temp`).
- **Tune**: Z offset (babysteps), apply to probe/endstop, save config, speed and flow factor, pressure advance.
- **Filament**: extrude/retract with length and speed, `LOAD_FILAMENT`/`UNLOAD_FILAMENT` macros when present, filament sensors. With Happy Hare 4.1 or newer an MMU tab shows the gates (four at a time: status, material, colour, mapped tool), the MMU sensors, the sync-feedback buffer and FlowGuard, and loads, unloads and homes the selected gate.
- **Spoolman** (when Moonraker has the `spoolman` component): the spool in use and what is left of it on the print page and in each gate of the MMU, with a colour bar that has one, two or three parts for multi-colour filaments; a picker lists the spools (the ones in a gate first, then the most recently used) to assign a spool to a gate or, without MMU, to choose the active spool, and to correct the remaining weight by weighing; the temperature page offers a preset for the spool in use and notes quietly when the target is off from it; before a print the confirmation names what differs between the file and the spools (material, weight, colour), only as a hint.
- **Console**: G-code history and live responses, a command field (on-screen keyboard or physical keyboard), and an expandable list of macros. Macros starting with `_` and macros that rename a built-in command (`rename_existing`) are hidden; macros can be hidden in the settings. Macros with parameters open a form first: each parameter is read from the macro's template, Klipper defaults show as placeholders in brackets.
- **System**: Klipper/firmware restart, host reboot/shutdown, service restart, power devices, update manager.
- **Settings**: Moonraker address, fullscreen, temperature presets — stored as JSON.
- **Network** (Settings → Network): search and join Wi-Fi (open, WPA/WPA2, WPA3, hidden), remove saved networks,
  Wi-Fi on/off, Ethernet status and connect/disconnect. See [Network settings](#network-settings).

Functions only appear when the printer has them (capabilities from `printer.objects.list` and `server.info`).

## Install on the printer host

1. Download the zip for your system from the [releases](https://github.com/freakydude/klipper-touch/releases):
   `aarch64-unknown-linux-gnu` (64-bit Raspberry Pi OS/Armbian, CB1/CB2), `armv7-unknown-linux-gnueabihf` (32-bit), `x86_64-unknown-linux-gnu`.
   Builds target Debian Bookworm and run on Bookworm and Trixie.
2. Unpack and run the installer as your normal user (the one running Klipper and Moonraker):

   ```bash
   unzip klipper-touch-aarch64-unknown-linux-gnu.zip -d klipper-touch-setup
   cd klipper-touch-setup && ./install.sh -m http://127.0.0.1:7125
   ```

   The installer
   - installs the packages from `system-dependencies.json` (among them `cage`, a minimal Wayland kiosk compositor),
   - copies the program to `~/klipper-touch` and creates the systemd service `klipper-touch` (fullscreen on `tty1`),
   - registers it with Moonraker's update manager (`[update_manager klipper-touch]` in `moonraker.conf`, service in
     `moonraker.asvc`) and restarts Moonraker.

   Options: `-m <moonraker url>`, `-d <printer_data folder>` (default `~/printer_data`), `-c beta` for preview builds of every change on `main`.
   Settings go to `printer_data/config/klipper-touch.json`, next to the Klipper configs, as KlipperScreen does with its
   `KlipperScreen.conf`. An existing `~/.config/klipper-touch/settings.json` is copied there once.
   Remove everything with `~/klipper-touch/uninstall.sh`; the settings file stays.

### Alternative: Debian package

Each release also contains `klipper-touch_<version>_<arch>.deb` (`amd64`, `arm64`, `armhf`). apt installs the
dependencies, removes the package cleanly, but Moonraker does **not** update it — install newer `.deb` files yourself.
Use either the zip installer or the package, not both (they would compete for `tty1`).

```bash
sudo apt install ./klipper-touch_0.2.0_arm64.deb
sudo systemctl disable getty@tty1                # free tty1 for the kiosk
sudo systemctl enable --now klipper-touch@$USER  # fullscreen on tty1 as this user
# remove again (stops and disables all klipper-touch@ instances)
sudo apt remove klipper-touch && sudo systemctl enable getty@tty1
```

The Moonraker address is taken from the settings (default `http://127.0.0.1:7125`, changeable on the Settings page).

On a desktop (e.g. Raspberry Pi OS with desktop, Windows, Linux PC) simply start the binary; it opens as a normal window.

### Updates

After the installation klipper-touch shows up in the update list of Mainsail, Fluidd and its own System page.
Moonraker downloads the matching zip from the GitHub releases, replaces `~/klipper-touch` and restarts the service.
The section written by the installer (add it yourself if `moonraker.conf` lives elsewhere):

```ini
[update_manager klipper-touch]
type: zip
channel: stable          # beta = also preview builds of main
repo: freakydude/klipper-touch
path: ~/klipper-touch
managed_services: klipper-touch
system_dependencies: system-dependencies.json
```

To get the preview builds of every change on `main`, set `channel: beta` in that section and restart Moonraker
(`sudo systemctl restart moonraker`). Then the update list offers the newest prerelease. Switch back to `stable` the
same way. The installer only writes the section once: re-running `install.sh` with `-c beta` does not change an existing
`channel` line, so edit it by hand. Your edits survive later updates, because Moonraker only replaces the program folder.

### Command line

```text
klipper-touch [--moonraker <url>] [--fullscreen] [--settings <file>] [--simulate-network]
```

| Option               | Meaning                                                                                          |
| -------------------- | ------------------------------------------------------------------------------------------------ |
| `-m, --moonraker`    | Moonraker address for this start, e.g. `http://printer.local:7125` (otherwise the saved setting) |
| `-f, --fullscreen`   | Fullscreen/kiosk mode                                                                            |
| `--settings`         | Settings file, default `~/.config/klipper-touch/settings.json` (Windows: `%APPDATA%`). The installer passes `printer_data/config/klipper-touch.json` |
| `--simulate-network` | Simulated Wi-Fi/Ethernet instead of NetworkManager (development builds only, see below)          |

The supported display is the BTT HDMI5 (800x480), where it has been run; the layout is made for it. Other resolutions show the same layout
scaled up or down to fit, so a 7" 1024x600 display fills its screen and a 4.3" 480x272 display shows everything, only
smaller. These are not tested yet. A 3.5" 480x320 display is too small to operate comfortably, and displays that hang on
the SPI bus may not work with the kiosk compositor at all. Layouts of their own for the 7" and 4.3" displays are planned.

Useful environment variables: `RUST_LOG=debug` (logging), `SLINT_BACKEND=winit-software` (force software rendering if the GPU
driver causes trouble), `SLINT_SCALE_FACTOR=1.5` (sets the scale by hand and turns the fitting to the display off).

Without a desktop or compositor the optional cargo feature `linuxkms` renders directly via DRM/KMS (experimental in Slint).

### Network settings

Moonraker can only report the host's network (`machine.system_info`), it cannot change it. klipper-touch therefore talks
to NetworkManager over D-Bus, like KlipperScreen. Managing Wi-Fi and Ethernet is offered when

- klipper-touch runs on Linux with NetworkManager (default on Raspberry Pi OS Bookworm and newer), and
- Moonraker runs on the same host (`localhost`/`127.0.0.1`), so the changed network is the printer's network.

The user needs the polkit rule `50-klipper-touch.rules` (allows the NetworkManager actions for the group `netdev`).
`install.sh` installs it to `/etc/polkit-1/rules.d` and adds the user to `netdev`; the Debian package ships it in
`/usr/share/polkit-1/rules.d`, add the user yourself: `sudo usermod -aG netdev $USER` (takes effect after re-login or a
service restart). Changing the network while printing asks for confirmation, because the connection may drop.

On Windows, on Linux without NetworkManager or with a remote Moonraker the section only shows the printer host's
interfaces and addresses read from Moonraker.

## Development

Rust stable (see `rust-toolchain.toml`). On Linux install `libfontconfig1-dev libxkbcommon-dev`.
For Windows without admin rights (setup, faster linking with `rust-lld`, build times) see
[docs/windows-without-admin.md](docs/windows-without-admin.md).

```bash
cargo run -p fake-moonraker -- --profile voron          # simulated printer on port 7125
cargo run -p klipper-touch -- -m http://127.0.0.1:7125
cargo clippy --workspace --all-targets
cargo test --workspace
cargo run -p ui-snapshot -- --out shots                  # render the screens to PNG (needs the fake running)
scripts/ui-snapshot.sh [-s 1024x600] [-F] [-S pages]           # WSL2/Arch: starts the fake itself, renders all screens
scripts/ui-preview.sh                                    # WSL2/Arch: renders crates/app/ui/preview/*.slint with sample data in seconds
cargo run -p icon-index                                  # after changing icons in assets/icons
```

Scripts exist in two variants with the same behaviour:

- `scripts/*.sh` (bash) for WSL2 with Arch Linux, the main development environment. `scripts/setup-wsl.sh` checks the environment.
- `scripts/*.ps1` (PowerShell) for Windows without admin rights. `scripts\setup.ps1` prepares the GNU toolchain there, see [docs/windows-without-admin.md](docs/windows-without-admin.md).

Structure, data flow and conventions: [docs/architecture.md](docs/architecture.md).
Workflows and use cases: `docs/UseCases and Dependencies.drawio`.

### Tools

| Tool                                                                                   | Needed for                                      | How to get it                                                                                                           | Where it is looked up                                                                  |
| -------------------------------------------------------------------------------------- | ----------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| Rust (`rustup`)                                                                        | everything                                      | [rustup.rs](https://rustup.rs); `rust-toolchain.toml` selects the version                                               | `PATH`                                                                                 |
| MinGW linker ([w64devkit](https://github.com/skeeto/w64devkit))                        | building on Windows without MSVC                | `scripts\setup.ps1` downloads it                                                                                        | `%USERPROFILE%\tools\w64devkit\bin`, added to the user `PATH` by the script            |
| [`slint-viewer`](https://docs.slint.dev/latest/docs/slint/guide/tooling/slint-viewer/) | `scripts/ui-preview.sh` (WSL2), `scripts\ui-preview.ps1` (Windows) | WSL2: the script compiles the version matching `slint` in `Cargo.lock` into `~/.local/bin` (`cargo install slint-viewer --version <same>`); Windows: the script downloads it | `slint-viewer` on `PATH` first, otherwise the script's install location |
| Slint extension for VS Code (optional)                                                 | live preview of `crates/app/ui/preview/*.slint` | VS Code marketplace (`Slint.slint`)                                                                                     | –                                                                                      |

To use your own `slint-viewer`, put it on `PATH`; it should have the same version as the `slint` crate.

### Development builds and releases

Local builds of `klipper-touch` include the cargo feature `dev-mode` (default): the option `--simulate-network`
and the simulated networks in the core. Releases are built with `--no-default-features` and contain none of it.
The development tools are separate programs and never shipped: `fake-moonraker` (simulated printer) and
`ui-snapshot` (screenshots of the UI).

### Contributing

`main` is protected: changes go through a pull request, all CI checks must be green and the code owner
(`.github/CODEOWNERS`) has to approve it.

### CI, versions and releases

- Versions come from git tags (`.github/scripts/version.sh`): releases are `vX.Y.0`, every later commit on `main` adds
  one to the patch number (`0.2.1`, `0.2.2`, ...). Pull request builds are `X.Y.Z-pr.<number>.<run>`, other branches
  `X.Y.Z-dev.<commit>`, local builds `0.0.0-dev`. The version is shown on the Settings page and by `--version`.
- Every push and pull request: format, clippy, tests, builds and packages for all targets (`.github/workflows/ci.yml`),
  with cargo caching. The packages are attached to the run as artifact `packages-<version>`; pull requests get a
  comment with the download link, so an open pull request can be tried before it is merged.
- Every push to `main` is published as prerelease `vX.Y.Z`, replacing the previous prerelease. Moonraker offers it on
  the `beta` channel.
- Release: run the workflow **Release** (Actions tab) on `main` and choose `minor` or `major`. It tags `vX.Y.0`,
  builds and publishes the release with generated notes; Moonraker offers it on the `stable` channel. The patch number
  then starts again at 0.
- Assets: one zip per target and one `.deb` per Linux target (`.github/scripts/build-deb.sh`, files in
  `scripts/debian`). Linux zips contain `klipper-touch`, `install.sh`, `uninstall.sh`,
  `system-dependencies.json` and `release_info.json` (version and asset name for Moonraker); Windows: `klipper-touch.exe`.

## License

GPL-3.0-or-later, see [LICENSE](LICENSE). Slint is used under its GPLv3 option.
