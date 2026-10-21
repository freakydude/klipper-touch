# Klipper-Touch (Rust + Slint)

Touch screen UI for [Klipper](https://www.klipper3d.org/) printers, talking to [Moonraker](https://moonraker.readthedocs.io/).
Built for small displays attached directly to the printer host (Raspberry Pi 3/4/5, BTT CB1/CB2), first target is the
BTT HDMI5 (5", 800x480, landscape). Native Rust application with a [Slint](https://slint.dev/) UI — no browser, no WebView.

> Work in progress. Tested against the included fake Moonraker and on desktop, **not yet on real printers**.
> Watch your printer, double check things. No warranty.

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

1. Download the archive for your system from the [releases](https://github.com/freakydude/klipper-touch/releases):
   `aarch64-unknown-linux-gnu` (64-bit Raspberry Pi OS/Armbian, CB1/CB2), `armv7-unknown-linux-gnueabihf` (32-bit), `x86_64-unknown-linux-gnu`.
   Builds target Debian Bookworm and run on Bookworm and Trixie.
2. Unpack and run the installer as your normal user:

   ```bash
   tar -xzf klipper-touch-slint-aarch64-unknown-linux-gnu.tar.gz
   ./install.sh -m http://127.0.0.1:7125
   ```

   The installer adds `cage` (a minimal Wayland kiosk compositor), installs the binary to `/usr/local/bin` and creates the
   systemd service `klipper-touch-slint` that runs the UI fullscreen on `tty1`. Remove it with `./uninstall.sh`.

On a desktop (e.g. Raspberry Pi OS with desktop, Windows, Linux PC) simply start the binary; it opens as a normal window.

### Command line

```text
klipper-touch-slint [--moonraker <url>] [--fullscreen] [--settings <file>]
```

| Option | Meaning |
|---|---|
| `-m, --moonraker` | Moonraker address for this start, e.g. `http://printer.local:7125` (otherwise the saved setting) |
| `-f, --fullscreen` | Fullscreen/kiosk mode |
| `--settings` | Settings file, default `~/.config/klipper-touch-slint/settings.json` (Windows: `%APPDATA%`) |

Useful environment variables: `RUST_LOG=debug` (logging), `SLINT_BACKEND=winit-software` (force software rendering if the GPU
driver causes trouble), `SLINT_SCALE_FACTOR=1.5` (bigger UI on high resolution displays).

Without a desktop or compositor the optional cargo feature `linuxkms` renders directly via DRM/KMS (experimental in Slint).

## Development

Rust stable (see `rust-toolchain.toml`). On Linux install `libfontconfig1-dev libxkbcommon-dev`.
For Windows without admin rights see [docs/windows-ohne-admin.md](docs/windows-ohne-admin.md).

```bash
cargo run -p fake-moonraker -- --profile voron          # simulated printer on port 7125
cargo run -p klipper-touch-slint -- -m http://127.0.0.1:7125
cargo clippy --workspace --all-targets
cargo test --workspace
cargo run -p klipper-touch-slint --bin ui-snapshot -- --out shots   # render all screens to PNG (needs the fake)
cargo run -p icon-index                                  # after changing icons in assets/icons
```

Structure, data flow and conventions: [docs/architecture.md](docs/architecture.md).
Workflows and use cases: `docs/UseCases and Dependencies.drawio`.

## License

GPL-3.0-or-later, see [LICENSE](LICENSE). Slint is used under its GPLv3 option.
