# Projektkontext

- Cargo-Workspace klipper-touch (Rust + Slint, GPL-3.0-or-later): `crates/core` (UI-unabhängiger Kern: Moonraker-Client, Zustand, Capabilities, Befehle, Einstellungen), `crates/app` (Slint-UI `ui/*.slint`, Binary `klipper-touch-slint`, Werkzeug `ui-snapshot`), `crates/icon-index` (Icon-Prüfung/Index), `fake-moonraker` (simulierter Moonraker für Tests und Entwicklung ohne Drucker). Aufbau: `docs/architecture.md`.
- Zielgerät: Touchscreen am Drucker-Host (Raspberry Pi 3/4/5, BTT CB1/CB2), zuerst BTT HDMI5 800x480 quer; winit-Backend, auf dem Gerät im Vollbild unter `cage`.
- Entwicklung nativ unter Windows ohne Admin-Rechte. Kein Docker, WSL oder Dev Container.
- Toolchain: `stable` (siehe `rust-toolchain.toml`), lokal per `scripts\setup.ps1` auf GNU-Host gestellt, Linker aus portablem w64devkit in `%USERPROFILE%\tools`. Kein MSVC. Das Repository bleibt plattformneutral (CI baut Linux x86_64/aarch64/armv7 in Debian Bookworm und Windows-MSVC).
- Firmen-Proxy mit TLS-Inspection: `CARGO_HTTP_CHECK_REVOKE=false` ist als Benutzer-Umgebungsvariable gesetzt, nicht im Repo. Keine Vorschläge, die Docker, WSL oder Admin-Rechte voraussetzen.
- Shell ist PowerShell, Pfade mit Backslash.
- Prüfen mit `cargo clippy --workspace --all-targets -- -D warnings` und `cargo test --workspace`; UI mit Skill `ui-snapshot`.
- Hintergrund und Stolperfallen: `docs\windows-ohne-admin.md`.

# Code-Vorgaben

- Clean Code nach [Clean Code Developer](https://clean-code-developer.de/): sprechende Namen, kleine Funktionen mit einer Aufgabe, DRY, KISS, Trennung der Verantwortlichkeiten, Boy-Scout-Regel.
- Rust nach dem [Rust Book](https://doc.rust-lang.org/book/) und Clippy; Slint nach der [Slint-Doku](https://docs.slint.dev/latest/docs/slint/).
- Fachlicher Hintergrund: Klipper, Moonraker (JSON-RPC/WebSocket, HTTP), Happy Hare. Fremdsysteme nur lesend ansehen.

# Skills und Übergabe

- Skills unter `.github/skills` zuerst laden, wenn eine Aufgabe passt (UI prüfen, Fake-Moonraker, draw.io, Windows-Build, Klipper/Moonraker-Referenz, Workflow-Review, Icons, Übergabe).
- Skills aktuell halten: bei neuen Regeln oder Erkenntnissen im Chat einen kurzen Skill-Vorschlag machen; nach größeren Arbeitspaketen `.github\skills\session-handover\scripts\check-skills.ps1` ausführen. Kleine Korrekturen direkt, inhaltliche Änderungen und neue Skills nur nach Zustimmung (Details im Skill `session-handover`).
- Neuer Chat: zuerst `/memories/repo/00-index.md` lesen.
