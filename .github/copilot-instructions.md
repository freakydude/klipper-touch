# Projektkontext

- Wurzelordner ist das Tauri-Projekt klipper-touch (SvelteKit-Frontend in `src`, Rust in `src-tauri`). `egui-hello` ist ein eigenständiges egui-Beispiel (eframe). `fake-moonraker` ist ein simulierter Moonraker für Tests und UI-Entwicklung ohne Drucker.
- Entwicklung nativ unter Windows ohne Admin-Rechte. Kein Docker, WSL oder Dev Container.
- Toolchain: `stable` (siehe `rust-toolchain.toml`), lokal per `scripts\setup.ps1` auf GNU-Host gestellt, Linker aus portablem w64devkit in `%USERPROFILE%\tools`. Kein MSVC. Das Repository bleibt plattformneutral (CI baut Linux, macOS, Windows-MSVC).
- Firmen-Proxy mit TLS-Inspection: `CARGO_HTTP_CHECK_REVOKE=false` ist als Benutzer-Umgebungsvariable gesetzt, nicht im Repo. Keine Vorschläge, die Docker, WSL oder Admin-Rechte voraussetzen.
- Shell ist PowerShell, Pfade mit Backslash.
- Prüfen mit `cargo clippy --manifest-path <projekt>\Cargo.toml`, Frontend mit `npm run build`/`npm run dev`.
- Hintergrund und Stolperfallen: `docs\windows-ohne-admin.md`.

# Code-Vorgaben

- Clean Code nach [Clean Code Developer](https://clean-code-developer.de/): sprechende Namen, kleine Funktionen mit einer Aufgabe, DRY, KISS, Trennung der Verantwortlichkeiten, Boy-Scout-Regel.
- Rust nach dem [Rust Book](https://doc.rust-lang.org/book/) und Clippy; Svelte 5 und SvelteKit nach der offiziellen Doku ([Svelte](https://svelte.dev/docs/svelte/overview), [SvelteKit](https://svelte.dev/docs/kit/introduction)).
- Fachlicher Hintergrund: Klipper, Moonraker (JSON-RPC/WebSocket, HTTP), Happy Hare, Tauri. Fremdsysteme nur lesend ansehen.

# Skills und Übergabe

- Skills unter `.github/skills` zuerst laden, wenn eine Aufgabe passt (UI prüfen, Fake-Moonraker, draw.io, Windows-Build, Klipper/Moonraker-Referenz, Workflow-Review, Übergabe).
- Skills aktuell halten: bei neuen Regeln oder Erkenntnissen im Chat einen kurzen Skill-Vorschlag machen; nach größeren Arbeitspaketen `.github\skills\session-handover\scripts\check-skills.ps1` ausführen. Kleine Korrekturen direkt, inhaltliche Änderungen und neue Skills nur nach Zustimmung (Details im Skill `session-handover`).
- Neuer Chat: zuerst `/memories/repo/00-index.md` lesen.
