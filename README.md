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

- Klipper-Touch listen on url: `ws://127.0.0.1/websocket` for a moonraker websocket api connection.
  To use another url, set environment variable `PUBLIC_KT_MOONRAKER_WS=<YOUR MOONRAKER WS URL>` (for example: `ws://192.168.1.10/websocket`)

## Development

### Prequisities

- Common

- Docker & Docker Compose
  see: [https://docs.docker.com/get-docker/](https://docs.docker.com/get-docker/)
- NPM Package manager (nodejs installation)
  see: [https://github.com/nodesource/distributions#installation-instructions](https://github.com/nodesource/distributions#installation-instructions)
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

- Build:

  ```bash
  npm run tauri build
  ```

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
