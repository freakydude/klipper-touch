# Rust unter Windows ohne Admin-Rechte

Rust (egui, Tauri) unter Windows, ohne Admin-Rechte, ohne WSL/Docker, mit rust-analyzer und Copilot in VS Code.

## Einrichtung

1. Rust über `rustup` installieren (geht ohne Admin, Standard-Toolchain egal).
2. `powershell -File scripts\setup.ps1` ausführen und VS Code komplett neu starten.
3. Tauri-App: `npm ci`, dann `npm run tauri dev`. egui-Beispiel: `cargo run --manifest-path egui-hello\Cargo.toml`

## Wie es funktioniert

Das Repository enthält nichts Rechnerspezifisches und baut so auch unter Linux, macOS und in GitHub Actions (dort MSVC). Alles für diesen Rechner erledigt `scripts\setup.ps1` außerhalb des Repos.

| Baustein | Zweck |
| --- | --- |
| `rust-toolchain.toml` | Plattformneutral: `stable` mit Clippy und rust-src. |
| `scripts/setup.ps1` | Setzt den rustup-Default-Host auf `x86_64-pc-windows-gnu` (dadurch wird `stable` lokal zu GNU aufgelöst; MSVC bräuchte `link.exe`, Admin). Legt portables MinGW ([w64devkit](https://github.com/skeeto/w64devkit)) in `%USERPROFILE%\tools` ab, ergänzt den Benutzer-PATH und ein leeres `libgcc_eh.a` (fehlt dort, wird von Rust beim Linken verlangt). Setzt `CARGO_HTTP_CHECK_REVOKE=false` (Benutzer-Umgebungsvariable): Firmen-Proxy ohne Sperrlisten-Prüfung lässt sonst Crate-Downloads scheitern (`CRYPT_E_NO_REVOCATION_CHECK`). |
| `src-tauri/Cargo.toml` | `crate-type = ["rlib"]`: Die GNU-cdylib überschreitet unter Windows das Export-Limit (65535). `staticlib`/`cdylib` braucht nur Mobile. |
| `.vscode/settings.json` | rust-analyzer prüft mit Clippy und kennt beide Cargo-Projekte (`src-tauri`, `egui-hello`). |

Der Schlüssel ist der Linker: w64devkit liefert `gcc`/`ld` als reines Benutzerverzeichnis-Tool. Damit können auch Build-Skripte und Proc-Makros kompiliert werden, auf denen rust-analyzer aufbaut.
