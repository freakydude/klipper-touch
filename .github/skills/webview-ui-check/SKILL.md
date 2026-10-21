---
name: webview-ui-check
description: 'Use when the Klipper-Touch UI must be verified, debugged or screenshotted: runs the Tauri app against the fake Moonraker, drives all screens via WebView2 remote debugging (CDP), collects console errors and Svelte exceptions, checks layout overflow at touch resolutions (480x320, 800x480) and saves WebView screenshots. Triggers: UI prüfen, Screenshot, Layout, Überlauf, Skalierung, Konsolenfehler, Smoke-Test, CDP, WebView2, Touch-Auflösung.'
---

# UI-Check der Tauri-WebView

## Wann verwenden

- Nach jeder Änderung an `src/**` (Svelte, TypeScript) oder am Fake-Moonraker, die sich auf die UI auswirkt.
- Bei Fragen zu Layout, Skalierung, Überlauf oder Konsolenfehlern.

## Vorbereitung (einmalig, bzw. nach Änderungen)

1. Fake bauen: `cargo build --manifest-path fake-moonraker\Cargo.toml`
2. App mit eingebettetem Frontend bauen: `npm run tauri build -- --debug --no-bundle` (Ergebnis: `src-tauri\target\debug\klipper-touch.exe`)

## Ablauf

1. `powershell -File .github\skills\webview-ui-check\scripts\run-ui-check.ps1 [-OutDir <ordner>] [-Viewports 480x320,800x480]`
2. Das Skript startet Fake (Port 7125) und App (Debug-Port 9222), ruft [ui-check.mjs](./scripts/ui-check.mjs) auf und beendet danach beide Prozesse.
3. Ausgabe lesen: `overflow ...` (Elemente außerhalb des Fensters), `problems` (Konsolenfehler, Exceptions). Exit-Code 2 bedeutet Probleme.
4. Screenshots (Standard: `%TEMP%\ui-check`) mit dem Bildbetrachter ansehen. Nur die WebView wird fotografiert.
5. Fehler beheben, Schritte 1–2 wiederholen. Die Screenshot-Ordner danach löschen.

## Was geprüft wird

- Alle Routen (`/printstate`, `/move`, `/temperature`, `/babysteps`, `/extrusion`) in den gewählten Auflösungen.
- Zustände: Aufheizen, Datei-Dialog (Bilder geladen?), Druck, Pause, Abbruch, Emergency Stop.
- Neue Szenarien als weitere Schritte in `ui-check.mjs` ergänzen (Fake per `rpc(...)` steuern, UI per `go(...)`/`clickText(...)`).

## Stolperfallen

- Debug-Builds öffnen ein DevTools-Fenster; das Skript wählt das Ziel ohne `devtools` in der URL.
- Die Skalierung der App ist `1rem = Fensterhöhe/320*16 px` (`scaleUI` in `+layout.svelte`); das Skript spiegelt das beim Viewport-Wechsel. Bei 4:3 laufen Buttons über, das ist bekannt (siehe `docs/` und Repo-Notiz zur Skalierung).
- Niemals Bildschirmfotos des ganzen Desktops machen (fremde Inhalte); immer `Page.captureScreenshot`.
- Port 9222 und 7125 müssen frei sein; das Skript beendet alte `klipper-touch`/`fake-moonraker`-Prozesse.
- Der Fake simuliert beschleunigt (`--time-scale`, Standard 10); Heizen und Druck laufen deshalb schnell.
