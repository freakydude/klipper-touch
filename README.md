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

### Configure

- Configure `Dockerfile.dev`, adapt `VITE_MOONRAKER_API` and `VITE_MOONRAKER_MOONRAKER` to your needs. Link either a real klipper installation or use the [virtual-klipper-printer project of the Mainsail team](https://github.com/mainsail-crew/virtual-klipper-printer)

### Install

Install depenencies in the project root folder with

```bash
npm install
```

### Run local development version

```bash
docker compose -f docker-compose.yml -f docker-compose.override.yml up
```

## Final words

Feel free to create pull requests and discussions. I can't solve anything alone.

Have fun!

If you like, buy me a coffee

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/F2F7GC8PC)

freakyDude
