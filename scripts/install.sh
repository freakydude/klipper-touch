#!/bin/bash
# Installiert klipper-touch-slint auf einem Debian/Raspberry-Pi-OS/Armbian-Host als Kiosk-Dienst.
# Die App laeuft im Vollbild unter cage (minimaler Wayland-Kiosk-Compositor) auf tty1.
#
# Aufruf: ./install.sh [-b <binary>] [-m <moonraker-url>] [-u <user>]
# Ohne Angaben: ./klipper-touch-slint, http://127.0.0.1:7125, aktueller Benutzer.
# NICHT auf einem echten Geraet geprueft; Rueckmeldungen bitte als Issue.

set -euo pipefail

BINARY="./klipper-touch-slint"
MOONRAKER="http://127.0.0.1:7125"
SERVICE_USER="${SUDO_USER:-$USER}"
INSTALL_PATH="/usr/local/bin/klipper-touch-slint"
SERVICE_FILE="/etc/systemd/system/klipper-touch-slint.service"

while getopts "b:m:u:" option; do
  case $option in
    b) BINARY=$OPTARG ;;
    m) MOONRAKER=$OPTARG ;;
    u) SERVICE_USER=$OPTARG ;;
    *) echo "usage: $0 [-b binary] [-m moonraker-url] [-u user]"; exit 2 ;;
  esac
done

report() { echo -e "\n###### $1"; }

if [ "$EUID" -eq 0 ]; then
  echo "Run as normal user; sudo is used where needed."
  exit 1
fi
if [ ! -x "$BINARY" ]; then
  echo "Binary '$BINARY' not found or not executable (download the release for your architecture)."
  exit 1
fi

report "Installing runtime packages (cage, fonts, GL, input)"
sudo apt-get update
sudo apt-get install -y cage seatd fontconfig fonts-dejavu-core libxkbcommon0 libegl1 libgles2 \
  libwayland-client0 libwayland-cursor0 libwayland-egl1 libinput10

report "Installing $INSTALL_PATH"
sudo install -m 0755 "$BINARY" "$INSTALL_PATH"
sudo usermod -aG video,input,render "$SERVICE_USER" || true

report "Creating $SERVICE_FILE"
USER_ID=$(id -u "$SERVICE_USER")
sudo tee "$SERVICE_FILE" > /dev/null << EOF
[Unit]
Description=klipper-touch-slint touch UI for Klipper
After=systemd-user-sessions.service network-online.target moonraker.service
Wants=network-online.target
Conflicts=getty@tty1.service

[Service]
Type=simple
User=$SERVICE_USER
PAMName=login
TTYPath=/dev/tty1
StandardInput=tty
Environment=XDG_RUNTIME_DIR=/run/user/$USER_ID
ExecStart=/usr/bin/cage -s -- $INSTALL_PATH --fullscreen --moonraker $MOONRAKER
Restart=always
RestartSec=5

[Install]
WantedBy=graphical.target
EOF

report "Enabling service"
sudo systemctl daemon-reload
sudo systemctl disable getty@tty1.service || true
sudo systemctl enable --now klipper-touch-slint.service
echo "Done. Logs: journalctl -u klipper-touch-slint -f"
