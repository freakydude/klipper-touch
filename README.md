# Klipper-Touch

Klipper-Touch is an alternative web-based UI for Klipper3d/Moonraker that focuses on small touch screens without keyboard or mouse.

This is currently nothing more than a personal experiment and a project to learn - even if it works.

## Caution

First of all, and as always with my other posts and projects. I am not a company. I'm doing this in my spare time and mostly because I'm really excited about making these things work and improving them.

I have tested all this stuff only with my own printer and the constellation around it. I have done my best to make it all work without problems. However, this is a work in progress. There is no guarantee. Be careful, watch your printer, double check things. Use it as is. I am not responsible for any damage or consequences of any kind.

And yes, help and improve if you find something.

## Some early screens

![Connect Screen](docs/pics/klipper-touch-0.1.0-experimental-connect.png 'Connect Screen')

![Printing Screen](docs/pics/klipper-touch-0.1.0-experimental-printstate-printing.png 'Printing Screen')

![Print Done Screen](docs/pics/klipper-touch-0.1.0-experimental-printstate-done.png 'Print Done Screen')

![Temperature Screen](docs/pics/klipper-touch-0.1.0-experimental-temperrature.png 'Temperature Screen')

## Prepare Bigtreetech CB1

- Install Btt image with panfrost support. >= v 2.3.3

- Update to debian bookworm

- ```bash
  sudo nano /etc/apt/sources.list
  ```

  replace bullseye by bookworm, add non-free-firmware to all entries

  ```bash
  deb http://deb.debian.org/debian bookworm main contrib non-free non-free-firmware
  
  # deb-src <http://deb.debian.org/debian> bookworm main contrib non-free non-free-firmware
  
  deb < http://deb.debian.org/debian > bookworm-updates main contrib non-free non-free-firmware
  # deb-src <http://deb.debian.org/debian> bookworm-updates main contrib non-free non-free-firmware
  
  deb < http://deb.debian.org/debian > bookworm-backports main contrib non-free non-free-firmware
  # deb-src <http://deb.debian.org/debian> bookworm-backports main contrib non-free non-free-firmware
  
  deb < http://deb.debian.org/debian-security > bookworm-security main contrib non-free non-free-firmware
  # deb-src <http://deb.debian.org/debian-security> bookworm-security main contrib non-free non-free-firmware
  ```

- Upgrade system from bullseye to bookworm

  ```bash
  sudo apt update
  sudo apt dist-upgrade
  sudo apt autoremove
  ```

- Install xinit and input drivers
  sudo apt install xinit xf86-input-evdev

## Install and run Klipper-Touch

- Download it from [https://github.com/freakydude/klipper-touch/releases](https://github.com/freakydude/klipper-touch/releases)

- Install Klipper-Touch
  - Go to your manual downloaded package directory and run

    ```bash
    sudo apt install ./klipper-touch_0.1.0_arm64.deb
    ```

- Configure your `cors_domains` in `moonraker.conf` to allow Klipper-Touch to access the moonraker apis

- Run Klipper-Touch
  - exclusive: `/usr/bin/startx /usr/bin/klipper-touch -- -nocursor` or
  - in a window manager: `/usr/bin/klipper-touch`

- Klipper-Touch listen on url: `http://127.0.0.1/` and `ws://127.0.0.1/websocket` for a moonraker api connection.
  - To use another url, use the commandline parameters

    ```bash
    -m <Moonraker-HTTP-URL>  eg. `http://192.168.1.1/`
    -w <Moonraker-Websocket-URL>  eg. `ws://192.168.1.1/websocket`
    ```

  - To start in fullscreen mode

    ```bash
    -f
    ```

- Run standalone without xinit and window manager

  ```bash
  /usr/bin/startx /usr/bin/klipper-touch -f -- -nocursor
  ```

- Run standalone with wayland and cage. On rpi4, disable dma buffer renderer for webkit to avoid black screen issues. This is a workaround and should be fixed in the future.

  - install cage

    ```bash
    sudo apt install cage
    ```

  - run with wayland and cage
  
    ```bash
    export WEBKIT_DISABLE_DMABUF_RENDERER=1
    export GDK_BACKEND=wayland

    cage -- klipper-touch -f             
    ```

## Development

### Prequisities

- Common

- Docker & Docker Compose
  see: [https://docs.docker.com/get-docker/](https://docs.docker.com/get-docker/)
- NPM Package manager (Node 26 runtime)
  see: [https://github.com/nodesource/distributions#installation-instructions](https://github.com/nodesource/distributions#installation-instructions)
  and use Node 26.x for local development and builds.
- Tauri
  see: [https://tauri.app/v1/guides/getting-started/prerequisites#setting-up-linux](https://tauri.app/v1/guides/getting-started/prerequisites#setting-up-linux)

  If you are using Archlinux, you can skip the rust paragraph

### Install dependencies on ArchLinux / WSL2 ArchLinux

- Install docker and docker-compose

  ```bash
  pacman -S docker docker-compose
  ```

- Install rust (if skipped before)

  ```bash
  pacman -S rustup
  rustup default stable
  ```

### Run or Build project local

- Install dependencies in the project root folder with

  ```bash
  npm install && npm install --save-dev @tauri-apps/cli
  ```

- Run:

  ```bash
  npm run tauri dev
  ```

  or with custom moonraker urls

  ```bash
  npm run tauri dev -- -- -- -m "http://<moonraker-ip>" -w "ws://<moonraker-ip>/websocket"
  ```

  or configure your `config.json` in the local storage folder (e.g. `~/.config/klipper-touch/config.json`) with or on windows in `%APPDATA%\klipper-touch\config.json` with

  ```json
  {
    "fullscreen": false,
    "moonrakerApi": "http://<moonraker-ip>",
    "moonrakerWs": "ws://<moonraker-ip>/websocket"
  }
  ```

  and run without parameters

  ```bash
  npm run tauri dev
  ```

  or with fullscreen mode

  ```bash
  npm run tauri dev -- -- -- -f
  ```

- Build:

  ```bash
  npm run tauri build
  ```

### Test without a printer (Fake-Moonraker)

`fake-moonraker` is a small simulated Klipper/Moonraker (WebSocket JSON-RPC and thumbnails over HTTP) with heaters, homing, moves, a demo file list and print jobs. It listens on `127.0.0.1:7125` only.

- Start it (`--time-scale` speeds up heating and printing, default 10):

  ```bash
  cargo run --manifest-path fake-moonraker/Cargo.toml -- --port 7125 --time-scale 10
  ```

  `--profile` selects which modules the simulated printer has, as `printer.objects.list` and `configfile` report them: `basic`, `voron` (default, adds `quad_gantry_level`) or `happy-hare` (adds a simplified Happy Hare MMU, `--gates 1-16`, default 4; `MMU_HOME`, `MMU_SELECT`, `MMU_CHANGE_TOOL`, `T<n>`, `MMU_LOAD`, `MMU_UNLOAD`).

- Start Klipper-Touch against it:

  ```bash
  npm run tauri dev -- -- -- -m "http://127.0.0.1:7125/" -w "ws://127.0.0.1:7125/websocket"
  ```

  In VS Code the launch configuration `Tauri Development Debug (Fake-Moonraker)` starts both.

- Run its tests with `cargo test --manifest-path fake-moonraker/Cargo.toml`.

### CrossCompile with docker

- CrossCompile:

  ```bash
  pacman -S qemu-user-static docker-buildx
  ```

- Build debian bullseye arm32v7 package (RPi 32 bit)

  ```bash
  docker compose up klipper-touch_build_arm32v7
  ```

- Build debian bullseye arm64v8 package (RPi 64 bit)

  ```bash
  docker compose up klipper-touch_build_arm64v8
  ```

## Final words

Feel free to create pull requests and discussions. I can't solve anything alone.

Have fun!

If you like, buy me a coffee

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/F2F7GC8PC)

freakyDude
