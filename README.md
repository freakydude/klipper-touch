# Klipper Touch

A svelte, vite and typescript based web ui for touch style input and control the klipper3d firmware.

## Status

This are very very early steps developing this ...

## Setup on MainsailOS (Debian 11)

Log into your klipper raspberry pi

## Clone Repository

```sh
cd ~
git clone https://github.com/freakydude/klipper-touch.git
```

### Install with Docker

Install docker [official docs here](https://docs.docker.com/engine/install/debian/)
Install docker compose [official docs here](https://docs.docker.com/compose/install/linux/#install-using-the-repository)

#### Run dev environment

```sh
# only initial or on package changes, or
# if you have nodejs/npm installed, you can skip this and run npm locally
docker-compose run klipper-touch npm install
docker-compose up # -d if you like it deferred
```

Run `npm` commands within docker for example with:

```sh
docker compose run klipper-touch npm run format
docker compose run klipper-touch npm i -D prettier prettier-plugin-sh
```

#### Run prod environment

- Adapt ENV variables in `Dockerfile`.
- (Re-)build docker klipper-touch container
- Build and start it up

```sh
docker-compose -f docker-compose.yml build
docker-compose -f docker-compose.yml up
```

### Start Process

```sh
cd ~/klipper-touch
npm run dev
```

### Configure Moonraker Updates

Add following section to moonraker.conf

```yaml
[update_manager klipper-touch]
type: git_repo
channel: dev
primary_branch: main
path: ~/klipper-touch
origin: https://github.com/freakydude/klipper-touch.git
is_system_service: True
enable_node_updates: True
install_script: scripts/install-klipper-touch.sh
```

## Finally

Find the dev version unter `http://<your-maschine>:3000`

Enjoy,
freakyDude
