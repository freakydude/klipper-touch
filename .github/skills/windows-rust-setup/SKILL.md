---
name: windows-rust-setup
description: "Use when Rust, Cargo or Slint builds fail or must be set up on the restricted Windows work machine (no admin, no WSL/Docker, corporate proxy with TLS inspection): link.exe missing, cannot find -lgcc_eh or -lshlwapi, CRYPT_E_NO_REVOCATION_CHECK, cargo cannot download crates, wrong toolchain, slow builds. Triggers: Build-Fehler, Linker, w64devkit, MinGW, rustup, GNU-Toolchain, Proxy, Zertifikat, setup.ps1."
---

# Rust unter Windows ohne Admin-Rechte

Ausgangslage: Toolchain `stable` mit GNU-Host (MSVC braucht Admin), Linker aus portablem w64devkit in `%USERPROFILE%\tools\w64devkit`. Einrichtung: `powershell -File scripts\setup.ps1`, danach VS Code komplett neu starten. Hintergrund: `docs\windows-ohne-admin.md`.

## Prinzip

Das Repository bleibt plattformneutral (CI baut Linux und Windows-MSVC). Alles Rechnerspezifische liegt außerhalb: rustup-Default-Host (`x86_64-pc-windows-gnu`), Benutzer-PATH, Benutzer-Umgebungsvariable `CARGO_HTTP_CHECK_REVOKE=false`. Niemals ein Windows-Triple in `rust-toolchain.toml` oder ein `.cargo/config.toml` mit Pfaden/Proxy-Workarounds einchecken.

## Symptome und Lösungen

| Symptom                                            | Ursache                            | Lösung                                                                                                                                                         |
| -------------------------------------------------- | ---------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `link.exe not found`, Build-Skripte scheitern      | MSVC-Toolchain aktiv               | `rustup show` prüfen; Host muss `x86_64-pc-windows-gnu` sein (`rustup set default-host ...` via `setup.ps1`)                                                   |
| `cannot find -lgcc_eh`                             | w64devkit liefert die Datei nicht  | leere `libgcc_eh.a` per `ar rcs` im gcc-Lib-Ordner (macht `setup.ps1`); NICHT `-C link-self-contained=yes` verwenden (`-lshlwapi` fehlt dann erst beim Linken) |
| `cannot find -lshlwapi`                            | `-C link-self-contained=yes` aktiv | in `%USERPROFILE%\.cargo\config.toml` (oder `RUSTFLAGS`) entfernen; danach baut alles neu                                                                      |
| Nur `cargo check`/`clippy` grün, `cargo build` rot | Check linkt nichts                 | immer auch `cargo build` prüfen                                                                                                                                |
| Skia-Renderer baut nicht                           | Skia braucht MSVC                  | nicht aktivieren; FemtoVG und Software-Renderer reichen                                                                                                        |
| `CRYPT_E_NO_REVOCATION_CHECK` beim Crate-Download  | Proxy ohne Sperrlistenprüfung      | `CARGO_HTTP_CHECK_REVOKE=false` als Benutzer-Umgebungsvariable; Downloads mit `curl.exe --ssl-no-revoke`                                                       |
| Container/WSL-Wege                                 | Nicht erlaubt/entfernt             | nicht vorschlagen; alles nativ unter Windows                                                                                                                   |
| `cargo`/`gcc` nicht gefunden in neuer Shell        | PATH nur im Benutzerprofil         | `$env:Path = [Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User')`                             |

## Befehle

- App: `cargo build -p klipper-touch-slint` (Ergebnis `target\debug\klipper-touch-slint.exe`).
- Prüfen: `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo fmt --all --check`.
- Lange Ausgaben in eine Logdatei umleiten und nur das Ende lesen (Hintergrund-Terminals liefern sonst unzuverlässige Ausgabe).

## Zeiten (gemessen)

Erster Build des Workspaces mit Slint ca. 4 min, inkrementell wenige Sekunden. `lld` und Debug-Info-Reduktion brachten früher keinen messbaren Gewinn; nicht nachrüsten.
