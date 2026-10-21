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

So far it has only run against the included fake Moonraker and on desktops.

![Print screen](docs/screenshots/print.png)

| | | |
|---|---|---|
| ![Files](docs/screenshots/files.png) | ![Move](docs/screenshots/move.png) | ![Temperature](docs/screenshots/temperature.png) |
| ![Tune](docs/screenshots/tune.png) | ![System](docs/screenshots/system.png) | ![Klipper shut down](docs/screenshots/klippy-shutdown.png) |

## Features

- **Print**: thumbnail, progress, elapsed/remaining/ETA, layers, filament; start, pause, resume, cancel; file browser with thumbnails.
- **Move**: jog X/Y/Z with selectable steps, home (all/XY/Z), motors off, Quad Gantry Level or Z-Tilt when configured.
- **Heat**: every heater with a target (nozzle, bed, `heater_generic`, `temperature_fan`), sensors, presets, part fan.
- **Tune**: Z offset (babysteps), apply to probe/endstop, save config, speed and flow factor, pressure advance.
- **Extrude**: extrude/retract with length and speed, `LOAD_FILAMENT`/`UNLOAD_FILAMENT` macros when present, filament sensors.
- **Macros**: all user macros (helpers starting with `_` are hidden).
- **Console**: G-code history and live responses, quick commands, on-screen keyboard.
- **System**: Klipper/firmware restart, host reboot/shutdown, service restart, power devices, update manager.
- **Settings**: Moonraker address, fullscreen, temperature presets — stored as JSON.

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

   Options: `-m <moonraker url>`, `-d <printer_data folder>` (default `~/printer_data`), `-c beta` for nightly builds.
   Remove everything with `~/klipper-touch/uninstall.sh`; settings stay in `~/.config/klipper-touch`.

On a desktop (e.g. Raspberry Pi OS with desktop, Windows, Linux PC) simply start the binary; it opens as a normal window.

### Updates

After the installation klipper-touch shows up in the update list of Mainsail, Fluidd and its own System page.
Moonraker downloads the matching zip from the GitHub releases, replaces `~/klipper-touch` and restarts the service.
The section written by the installer (add it yourself if `moonraker.conf` lives elsewhere):

```ini
[update_manager klipper-touch]
type: zip
channel: stable          # beta = also nightly builds
repo: freakydude/klipper-touch
path: ~/klipper-touch
managed_services: klipper-touch
system_dependencies: system-dependencies.json
```

### Command line

```text
klipper-touch [--moonraker <url>] [--fullscreen] [--settings <file>]
```

| Option | Meaning |
|---|---|
| `-m, --moonraker` | Moonraker address for this start, e.g. `http://printer.local:7125` (otherwise the saved setting) |
| `-f, --fullscreen` | Fullscreen/kiosk mode |
| `--settings` | Settings file, default `~/.config/klipper-touch/settings.json` (Windows: `%APPDATA%`) |

Useful environment variables: `RUST_LOG=debug` (logging), `SLINT_BACKEND=winit-software` (force software rendering if the GPU
driver causes trouble), `SLINT_SCALE_FACTOR=1.5` (bigger UI on high resolution displays).

Without a desktop or compositor the optional cargo feature `linuxkms` renders directly via DRM/KMS (experimental in Slint).

## Development

Rust stable (see `rust-toolchain.toml`). On Linux install `libfontconfig1-dev libxkbcommon-dev`.
For Windows without admin rights see [docs/windows-ohne-admin.md](docs/windows-ohne-admin.md).

```bash
cargo run -p fake-moonraker -- --profile voron          # simulated printer on port 7125
cargo run -p klipper-touch -- -m http://127.0.0.1:7125
cargo clippy --workspace --all-targets
cargo test --workspace
cargo run -p klipper-touch --bin ui-snapshot -- --out shots   # render all screens to PNG (needs the fake)
cargo run -p icon-index                                  # after changing icons in assets/icons
```

Structure, data flow and conventions: [docs/architecture.md](docs/architecture.md).
Workflows and use cases: `docs/UseCases and Dependencies.drawio`.

### Contributing

`main` is protected: changes go through a pull request, all CI checks must be green and the code owner
(`.github/CODEOWNERS`) has to approve it.

### CI and releases

- Every push and pull request: format, clippy, tests and builds for all targets (`.github/workflows/ci.yml`),
  with cargo caching.
- Release assets: one zip per target. Linux zips contain `klipper-touch`, `install.sh`, `uninstall.sh`,
  `system-dependencies.json` and `release_info.json` (version and asset name for Moonraker); Windows: `klipper-touch.exe`.
- Nightly: at 02:00 UTC `main` is published as prerelease `nightly-<date>-<commit>`, replacing the previous one, only
  if `main` changed (`.github/workflows/nightly.yml`, manual run with `force` possible).
- Release: set `version` in `Cargo.toml` (`[workspace.package]`), commit via pull request, then
  `git tag -a v0.3.0 -m "v0.3.0"` and `git push origin v0.3.0`. CI builds and creates the release with generated notes;
  Moonraker offers it to every installation on the `stable` channel.

## License

GPL-3.0-or-later, see [LICENSE](LICENSE). Slint is used under its GPLv3 option.
