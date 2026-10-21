# Rust unter Windows ohne Admin-Rechte

Entwicklung von klipper-touch-slint unter Windows ohne Admin-Rechte, ohne WSL/Docker, mit rust-analyzer und Copilot in VS Code.

## Einrichtung

1. Rust über `rustup` installieren (geht ohne Admin, Standard-Toolchain egal).
2. `powershell -File scripts\setup.ps1` ausführen und VS Code komplett neu starten.
3. Prüfen: `cargo test --workspace`; starten: Task "run: klipper-touch-slint (Fake-Moonraker)" oder Debug-Konfiguration "debug: klipper-touch-slint (Fake-Moonraker)".

Der erste Build lädt und übersetzt Slint mit allen Abhängigkeiten und dauert einige Minuten.

## Wie es funktioniert

Das Repository enthält nichts Rechnerspezifisches und baut so auch unter Linux und in GitHub Actions (dort MSVC). Alles für diesen Rechner erledigt `scripts\setup.ps1` außerhalb des Repos.

| Baustein | Zweck |
| --- | --- |
| `rust-toolchain.toml` | Plattformneutral: `stable` mit Clippy, rustfmt und rust-src. |
| `scripts/setup.ps1` | Setzt den rustup-Default-Host auf `x86_64-pc-windows-gnu` (dadurch wird `stable` lokal zu GNU aufgelöst; MSVC bräuchte `link.exe`, Admin). Legt portables MinGW ([w64devkit](https://github.com/skeeto/w64devkit)) in `%USERPROFILE%\tools` ab, ergänzt den Benutzer-PATH und ein leeres `libgcc_eh.a` (fehlt dort, wird von Rust beim Linken verlangt). Setzt `CARGO_HTTP_CHECK_REVOKE=false` (Benutzer-Umgebungsvariable): Firmen-Proxy ohne Sperrlisten-Prüfung lässt sonst Crate-Downloads scheitern (`CRYPT_E_NO_REVOCATION_CHECK`). |
| `.vscode/settings.json` | rust-analyzer prüft mit Clippy; SVG-Dateien werden beim Speichern nicht formatiert (Icon-Regeln). |

Der Schlüssel ist der Linker: w64devkit liefert `gcc`/`ld` als reines Benutzerverzeichnis-Tool. Damit können auch Build-Skripte und Proc-Makros kompiliert werden, auf denen rust-analyzer aufbaut.

## Stolperfallen

- `cannot find -lshlwapi` beim Linken: In `%USERPROFILE%\.cargo\config.toml` steht `-C link-self-contained=yes`. Die Zeile entfernen; w64devkit plus leere `libgcc_eh.a` reichen.
- Skia als Slint-Renderer braucht MSVC; das Projekt nutzt deshalb FemtoVG und den Software-Renderer.
- Die UI ohne Fenster prüfen: `cargo run -p klipper-touch-slint --bin ui-snapshot` gegen einen laufenden Fake (Skill `ui-snapshot`).
