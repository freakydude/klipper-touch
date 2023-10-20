# Klipper-Touch

Klipper-Touch is an alternative web-based UI for Klipper3d/Moonraker that focuses on small touch screens without keyboard or mouse.

This is currently nothing more than a personal experiment and a project to learn - even if it works.

## Caution

First of all, and as always with my other posts and projects. I am not a company. I'm doing this in my spare time and mostly because I'm really excited about making these things work and improving them.

I have tested all this stuff only with my own printer and the constellation around it. I have done my best to make it all work without problems. However, this is a work in progress. There is no guarantee. Be careful, watch your printer, double check things. Use it as is. I am not responsible for any damage or consequences of any kind.

And yes, help and improve if you find something.

## Prequisities

- Docker & Docker Compose
  see: [https://docs.docker.com/get-docker/](https://docs.docker.com/get-docker/)
- NPM Package manager (nodejs installation)
  see: [https://github.com/nodesource/distributions#installation-instructions](https://github.com/nodesource/distributions#installation-instructions)

## Development

### Install dependencies on ArchLinux / WSL2 ArchLinux

- ```bash
  pacman -S docker docker-compose docker-buildx
  ```

- CrossCompile:

  ```bash
  pacman -S qemu-user-static
  ```

- Install CrossArchs (optional):

  ```bash
  docker run --privileged --rm tonistiigi/binfmt --install arm64,arm
  ```

- Install depenencies in the project root folder with

  ```bash
  npm install && npm install --save-dev @tauri-apps/cli
  ```

### Run or Build project local

- Run:

  ```bash
  npm run tauri deb
  ```

- Build:

  ```bash
  npm run tauri build
  ```

### CrossCompile with docker

- Build debian bullseye arm32v7 package (RPi 32 bit)

  ```bash
  docker compose build klipper-touch_build_arm32v7
  docker compose run klipper-touch_build_arm32v7
  ```

- Build debian bullseye arm64v8 package (RPi 64 bit)

  ```bash
  docker compose build klipper-touch_build_arm64v8
  docker compose run klipper-touch_build_arm64v8
  ```

## Final words

Feel free to create pull requests and discussions. I can't solve anything alone.

Have fun!

If you like, buy me a coffee

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/F2F7GC8PC)

freakyDude
