#!/bin/bash
# Entfernt den Kiosk-Dienst und das Programm wieder; Pakete und Einstellungen bleiben.
set -euo pipefail

sudo systemctl disable --now klipper-touch-slint.service || true
sudo rm -f /etc/systemd/system/klipper-touch-slint.service /usr/local/bin/klipper-touch-slint
sudo systemctl daemon-reload
sudo systemctl enable getty@tty1.service || true
echo "Removed. Settings remain in ~/.config/klipper-touch-slint/."
