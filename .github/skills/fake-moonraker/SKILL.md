---
name: fake-moonraker
description: "Use when working on or with the simulated Moonraker/Klipper in fake-moonraker/: start it, choose a machine profile (basic, voron, happy-hare), add JSON-RPC methods, G-code commands or printer objects, write tests, or reproduce UI behaviour without a printer. Triggers: Fake-Moonraker, Simulator, Mock, kein Drucker, printer.objects.list, Happy Hare simulieren, neue API-Methode, Profil."
---

# Fake-Moonraker

Rust-Crate im Workspace (`fake-moonraker/`, axum + tokio, auch als Bibliothek für Tests nutzbar) mit WebSocket-JSON-RPC auf `/websocket` und HTTP-Thumbnails (SVG) unter `/server/files/gcodes/...`. Lauscht nur auf `127.0.0.1`.

## Start

`cargo run -p fake-moonraker -- --port 7125 --time-scale 10 --profile voron` (VS-Code-Task `run: fake-moonraker`, Debug-Konfiguration "debug: fake-moonraker")

- `--profile basic|voron|happy-hare` (Standard `voron`: mit `quad_gantry_level` und `heater_generic chamber`), `--gates 1-16` (nur Happy Hare, Standard 4).
- UI dagegen: `cargo run -p klipper-touch-slint -- -m http://127.0.0.1:7125` oder Task "run: klipper-touch-slint (Fake-Moonraker)" bzw. Debug-Konfiguration "debug: klipper-touch-slint (Fake-Moonraker)" (starten den Fake per Task "run: fake-moonraker" mit).
- Im Test ohne Prozess: `fake_moonraker::server::serve(listener, ServerConfig { .. })` in einem Tokio-Task (siehe `crates/core/tests/service.rs`).

## Aufbau (eine Aufgabe pro Modul)

| Modul                      | Aufgabe                                                                                |
| -------------------------- | -------------------------------------------------------------------------------------- |
| `gcode.rs`                 | G-Code-Zeilen in `Command` parsen (inkl. `MmuCommand`)                                 |
| `printer.rs`               | Simulation: Heizer, Toolhead, Druckauftrag, `status()`, `object_names()`, `configfile` |
| `happy_hare.rs`            | vereinfachte Happy-Hare-Objekte (`mmu`, `mmu_machine`, ...)                            |
| `machine.rs`               | `MachineProfile`: welche optionalen Module existieren                                  |
| `host.rs`                  | Schaltgeräte, Update-Manager, G-Code-Verlauf (`server.gcode_store`)                    |
| `files.rs`                 | Demo-Dateien, Metadaten, SVG-Thumbnails                                                |
| `subscription.rs`          | Abo einer Verbindung, Snapshot und Diff (`notify_status_update`)                       |
| `session.rs`               | JSON-RPC-Methoden pro Verbindung                                                       |
| `backend.rs` / `server.rs` | gemeinsamer Zustand, Broadcast von Notifications, axum-Routen, Simulationstakt         |

## Abgebildet

- Methoden: `server.connection.identify`, `server.info`, `printer.info`, `printer.objects.list|query|subscribe`, `printer.gcode.script`, `printer.print.start|pause|resume|cancel`, `printer.emergency_stop`, `printer.restart`, `printer.firmware_restart`, `server.files.list|metadata|thumbnails`, `server.gcode_store`, `machine.device_power.devices|post_device`, `machine.update.status|refresh|upgrade`, `machine.services.restart`, `machine.reboot`, `machine.shutdown`.
- Notifications: `notify_status_update`, `notify_gcode_response` (Fehler als `!! ...`), `notify_klippy_shutdown` (Not-Aus), `notify_klippy_ready` (Neustart), `notify_power_changed`.
- G-Code: `G0/G1`, `G28`, `G90/G91`, `M82/M83`, `M84/M18`, `M104`, `M140`, `M106/M107`, `M220`, `M221`, `SET_GCODE_OFFSET`, `SET_PRESSURE_ADVANCE`, `SET_HEATER_TEMPERATURE`, `SAVE_CONFIG`, `QUAD_GANTRY_LEVEL`, MMU-Befehle; unbekannte Befehle (z. B. Makros) werden ignoriert.
- Objekte zusätzlich: Makros `gcode_macro PRINT_START|PRINT_END|LOAD_FILAMENT|UNLOAD_FILAMENT|PARK|_PURGE_HELPER`, `temperature_sensor raspberry_pi`.

## Erweitern

- **Neue RPC-Methode:** Arm in `Session::dispatch` (`session.rs`), Hilfsfunktion daneben, Fehler als `RpcError` (400 Parameter/Befehl, 404 nicht gefunden).
- **Neuer G-Code-Befehl:** `Command`-Variante und Parser in `gcode.rs`, Verhalten in `Printer::apply`, Unit-Test im Parser und im Drucker.
- **Neues Objekt/Modul:** in `Printer::status()` nur bei passendem Profil einfügen, Abschnitt in `config_sections()` ergänzen (die Objektliste kommt aus dem Status, es gibt keine zweite Quelle), Profil in `machine.rs` und `main.rs`.
- Objekte und Felder nur nach Doku benennen (Klipper Status Reference, Happy-Hare-Doku "Printer Variables"); Annahmen im Code kommentieren, nicht verifiziertes Verhalten im Skill oder README nennen.

## Tests

- `cargo test -p fake-moonraker` (Unit-Tests in den Modulen, Ende-zu-Ende in `tests/websocket.rs` mit `time_scale: 1000`); der Kern testet zusätzlich gegen den Fake (`cargo test -p klipper-touch-core`).
- `cargo clippy --workspace --all-targets -- -D warnings`.
- JSON-Vergleiche: `Value == 300.0` ist nicht gleich `Value == 300` (Fließkomma vs. Ganzzahl).

## Grenzen

Kein Upload, keine History, keine Queue, keine Konsolenantworten für erfolgreiche Befehle. Thumbnails sind SVG (echte Slicer liefern PNG). Happy-Hare-Verhalten (Selector, Unload, Fehlertexte) ist vereinfacht und nicht gegen ein echtes System geprüft.
