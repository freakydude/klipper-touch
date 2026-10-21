---
name: fake-moonraker
description: 'Use when working on or with the simulated Moonraker/Klipper in fake-moonraker/: start it, choose a machine profile (basic, voron, happy-hare), add JSON-RPC methods, G-code commands or printer objects, write tests, or reproduce UI behaviour without a printer. Triggers: Fake-Moonraker, Simulator, Mock, kein Drucker, printer.objects.list, Happy Hare simulieren, neue API-Methode, Profil.'
---

# Fake-Moonraker

Eigenständiges Rust-Crate (`fake-moonraker/`, axum + tokio) mit WebSocket-JSON-RPC auf `/websocket` und HTTP-Thumbnails unter `/server/files/gcodes/...`. Lauscht nur auf `127.0.0.1`.

## Start

`cargo run --manifest-path fake-moonraker\Cargo.toml -- --port 7125 --time-scale 10 --profile voron`

- `--profile basic|voron|happy-hare` (Standard `voron`: mit `quad_gantry_level`), `--gates 1-16` (nur Happy Hare, Standard 4).
- UI dagegen: `-m http://127.0.0.1:7125/ -w ws://127.0.0.1:7125/websocket`. VS-Code-Konfiguration: "Tauri Development Debug (Fake-Moonraker)".

## Aufbau (eine Aufgabe pro Modul)

| Modul                      | Aufgabe                                                                                |
| -------------------------- | -------------------------------------------------------------------------------------- |
| `gcode.rs`                 | G-Code-Zeilen in `Command` parsen (inkl. `MmuCommand`)                                 |
| `printer.rs`               | Simulation: Heizer, Toolhead, Druckauftrag, `status()`, `object_names()`, `configfile` |
| `happy_hare.rs`            | vereinfachte Happy-Hare-Objekte (`mmu`, `mmu_machine`, ...)                            |
| `machine.rs`               | `MachineProfile`: welche optionalen Module existieren                                  |
| `files.rs`                 | Demo-Dateien, Metadaten, SVG-Thumbnails                                                |
| `subscription.rs`          | Abo einer Verbindung, Snapshot und Diff (`notify_status_update`)                       |
| `session.rs`               | JSON-RPC-Methoden pro Verbindung                                                       |
| `backend.rs` / `server.rs` | gemeinsamer Zustand, axum-Routen, Simulationstakt                                      |

## Erweitern

- **Neue RPC-Methode:** Arm in `Session::dispatch` (`session.rs`), Hilfsfunktion daneben, Fehler als `RpcError` (400 Parameter/Befehl, 404 nicht gefunden).
- **Neuer G-Code-Befehl:** `Command`-Variante und Parser in `gcode.rs`, Verhalten in `Printer::apply`, Unit-Test im Parser und im Drucker.
- **Neues Objekt/Modul:** in `Printer::status()` nur bei passendem Profil einfügen, Abschnitt in `config_sections()` ergänzen (die Objektliste kommt aus dem Status, es gibt keine zweite Quelle), Profil in `machine.rs` und `main.rs`.
- Objekte und Felder nur nach Doku benennen (Klipper Status Reference, Happy-Hare-Doku "Printer Variables"); Annahmen im Code kommentieren, nicht verifiziertes Verhalten im Skill oder README nennen.

## Tests

- `cargo test --manifest-path fake-moonraker\Cargo.toml` (Unit-Tests in den Modulen, Ende-zu-Ende in `tests/websocket.rs` mit `time_scale: 1000`).
- `cargo clippy --manifest-path fake-moonraker\Cargo.toml --all-targets -- -D warnings`.
- JSON-Vergleiche: `Value == 300.0` ist nicht gleich `Value == 300` (Fließkomma vs. Ganzzahl).
- README-Abschnitt "Test without a printer" aktuell halten.

## Grenzen

Nur die von der UI genutzten Methoden sind abgebildet; kein Upload, keine History, keine Queue. Happy-Hare-Verhalten (Selector, Unload, Fehlertexte) ist vereinfacht und nicht gegen ein echtes System geprüft.
