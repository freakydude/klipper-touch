---
name: ui-snapshot
description: "Use when the klipper-touch Slint UI must be verified, debugged or screenshotted: renders every page headless with the Slint software renderer against the fake Moonraker (standby, files, printing, confirm dialog, cancelled, Klipper shutdown) and writes PNGs to look at. Triggers: UI prüfen, Screenshot, Layout, Überlauf, abgeschnitten, Lesbarkeit, Touch-Auflösung, 800x480, Seite ansehen, Smoke-Test."
---

# UI-Snapshot (Slint, ohne Fenster)

Das Werkzeug `ui-snapshot` (`crates/app/src/bin/ui-snapshot.rs`) setzt eine eigene Slint-Plattform mit `MinimalSoftwareWindow`, verbindet sich mit einem Moonraker, spielt einen festen Ablauf durch und speichert jede Ansicht als PNG. Es nutzt dieselben Bausteine wie die App (`Presenter`, `SettingsController`, `actions::bind`), nur ohne Ereignisloop.

## Ablauf

1. `powershell -File .github\skills\ui-snapshot\scripts\run-ui-snapshot.ps1 [-OutDir <ordner>] [-Profile basic|voron|happy-hare] [-Size 800x480]` (Standard: `%TEMP%\kt-ui-snapshots`, `voron`, `800x480`). Das Skript baut, startet den Fake auf Port 7125, rendert und beendet den Fake.
2. Die PNGs mit dem Bildbetrachter ansehen: abgeschnittene Texte (`…`), Überlauf am Rand, Lesbarkeit, deaktivierte Knöpfe, Zustandsfarben.
3. Fehler beheben, erneut laufen lassen. Ordner danach löschen. Gute Bilder für die Doku nach `docs/screenshots/` kopieren.

## Gerenderte Ansichten

Verbinden, Druck (Standby), Dateiliste, Bewegen, Heizen, Tune, Extruder, Makros, Konsole, System, Einstellungen, Druck läuft, Heizen während Druck, Abbruch-Dialog, abgebrochen, Klipper-Shutdown (nach Not-Aus). Neue Szenarien in `run()` von `ui-snapshot.rs` ergänzen: Befehl über `session.service.send(...)`, warten mit `session.wait_until(...)`, Seite mit `session.show_page(...)`.

## Stolperfallen

- Der Software-Renderer kann keine Rotation/Skalierung und Schatten nur eingeschränkt; was hier gut aussieht, sieht mit FemtoVG gleich oder besser aus.
- Asynchrone Effekte (`slint::spawn_local`) laufen im Werkzeug nicht; Snapshots und Ereignisse werden in `Session::pump` direkt übertragen. `init`-Callbacks der Seiten (Konsole, System) werden dagegen ausgeführt.
- Port 7125 muss frei sein; das Skript beendet alte `fake-moonraker`/`ui-snapshot`-Prozesse.
- Der Fake simuliert beschleunigt (`--time-scale 20`); Heizen und Druck laufen deshalb schnell.
- Keine Screenshots des ganzen Desktops (fremde Inhalte); immer dieses Werkzeug oder nur das App-Fenster.
