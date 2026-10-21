---
name: klipper-moonraker-reference
description: 'Use when designing or implementing anything that talks to Klipper, Moonraker or Happy Hare: which API method or printer object to use, capability detection (printer.objects.list, server.info, configfile), print_stats and webhooks states, Happy Hare printer.mmu variables, or where to read the official docs. Triggers: Moonraker API, JSON-RPC, objects.list, print_stats, webhooks, Happy Hare, mmu, Capability, Klipper-Objekt.'
---

# Klipper / Moonraker / Happy Hare – Referenz

Fremdsysteme (Docs, GitHub, Foren, echte Drucker) nur **lesend** ansehen. Unten steht, was wir geprüft bzw. aus Doku gelesen haben; "Annahme" heißt nicht verifiziert.

## Quellen

- Moonraker: https://moonraker.readthedocs.io/en/latest/ | Repo https://github.com/Arksine/moonraker
- Klipper: https://www.klipper3d.org/ (Status Reference, G-Codes, API Server) | Repo https://github.com/Klipper3d/klipper
- Happy Hare v4: https://moggieuk.github.io/Happy-Hare-Doc/ (Reference-Printer-Variables, Reference-Commands, Mainsail-Fluidd-Integration, Dev-Simulator) | Repo https://github.com/moggieuk/Happy-Hare
- Referenz-UIs:
  - KlipperScreen: https://klipperscreen.readthedocs.io/en/latest/ (Panels: https://klipperscreen.readthedocs.io/en/latest/Panels/) | Repo https://github.com/KlipperScreen/KlipperScreen
  - KlipperScreen Happy Hare Edition (Fork für Happy Hare): https://github.com/moggieuk/KlipperScreen-Happy-Hare-Edition (MMU-Panels: im Repo unter docs/MMU.md)
  - Mainsail: https://docs.mainsail.xyz/ | Repo https://github.com/mainsail-crew/mainsail; Happy-Hare-Forks: mainsail-happy-hare-edition, fluidd-happy-hare-edition (github.com/moggieuk)
- Tauri: https://tauri.app/start/

## Verbindung und Protokoll

- WebSocket `/websocket`, JSON-RPC 2.0; Anfragen mit `id`, Notifications ohne `id`.
- Handshake der UI: `server.connection.identify`, dann `printer.objects.subscribe` (pro Verbindung ersetzt jeder Aufruf das Abo; Antwort enthält `status` als Startstand).
- Änderungen kommen als `notify_status_update` (`params[0]` = geänderte Felder, `params[1]` = eventtime); außerdem `notify_klippy_ready`, `notify_klippy_disconnected`, `notify_klippy_error`.
- Nach Klippy-Neustart verliert Moonraker die Abos; neu abonnieren.
- Häufig genutzte Methoden: `printer.gcode.script`, `printer.print.start|pause|resume|cancel`, `printer.emergency_stop`, `printer.restart`, `printer.firmware_restart`, `server.files.list|metadata|thumbnails`, `printer.objects.list|query|subscribe`.

## Capability-Erkennung

- Klipper-Objekte: `printer.objects.list` (z. B. `quad_gantry_level`, `bed_mesh`, `probe`, `exclude_object`, `firmware_retraction`, `filament_switch_sensor <n>`, `mmu`).
- Konfiguration: Objekt `configfile` (`config`, `settings`, `save_config_pending`).
- Moonraker-Komponenten: `server.info` (u. a. `history`, `job_queue`, `power`, `spoolman`, `update_manager`, `webcam`) – Feldnamen aus der Moonraker-Doku, nicht in diesem Projekt getestet.
- Leitidee des Projekts: Workflow des Menschen zuerst; Fähigkeiten entscheiden nur, ob ein Schritt oder eine Variante angeboten wird (siehe `docs/UseCases and Dependencies.drawio`, Seite "Workflows").

## Zustände

- `print_stats.state`: `standby | printing | paused | complete | cancelled | error`.
- `webhooks.state`: `startup | ready | shutdown | error | disconnected` (UI-seitig `disconnected` zusätzlich bei WebSocket-Verlust).

## Happy Hare (v4.1.0, laut Doku)

- Objekte: `printer.mmu`, `printer.mmu_machine`, dazu `mmu_encoder <n>`, `mmu_extruder <n>`, `mmu_stepper <n>`, `mmu_leds <n>`; Dump auf dem Drucker mit `MMU_DUMP_VARS`.
- `printer.mmu` (Auszug): `enabled`, `num_gates`, `is_homed`, `print_state`, `unit`, `tool` (-1 unbekannt, -2 Bypass), `gate`, `active_filament`, `filament` (`Loaded|Unloaded|Unknown`), `filament_pos` (-1 bis 10), `action`, `reason_for_pause`, `ttg_map[]`, `gate_status[]` (-1 unbekannt, 0 leer, 1 verfügbar, 2 aus Puffer), `gate_filament_name[]`, `gate_material[]`, `gate_color[]`, `gate_color_rgb[]`, `slicer_tool_map`.
- `printer.mmu_machine`: `happy_hare_version`, `num_units`, `unit_N` (Selector-Typ, Gates, Bypass, ...).
- Befehle: `MMU_HOME`, `MMU_SELECT GATE=`, `MMU_CHANGE_TOOL TOOL=` bzw. `T<n>`, `MMU_LOAD`, `MMU_UNLOAD`, `MMU_RECOVER`, `MMU_UNLOCK`, `MMU_TTG_MAP`. Makros laufen nur auf einem echten System; Testen ohne Drucker: Happy-Hare-Simulator ("make console", Python, eigenes Terminal) oder unser Fake mit Profil `happy-hare`.
- Mainsail/Fluidd/KlipperScreen haben eigene Forks bzw. Panels für Happy Hare: Hauptpanel, Tool-to-Gate-Mapping, Gate-Map-Editor, Wartung/Recover, Extruder-Farben.
