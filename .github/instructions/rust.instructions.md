---
description: 'Use when writing or changing Rust code in this repository (src-tauri, fake-moonraker, egui-hello): Clean Code Developer rules, Rust Book and Clippy conventions, error handling, tests.'
applyTo: '**/*.rs'
---

# Rust-Code

- Clean Code Developer: sprechende Namen, kleine Funktionen mit einer Aufgabe, DRY, KISS, Trennung der Verantwortlichkeiten, Boy-Scout-Regel. Neue Logik in eigene Module statt in `main`/`lib` zu stapeln.
- Fehler als eigene Typen oder `Result` mit sprechender Meldung; kein `unwrap`/`expect` außerhalb von Tests, außer bei echten Invarianten (dann mit Begründung im `expect`).
- Kommentare nur für Absichten, die der Code nicht zeigt; eine kurze Zeile. Im Bestand sind Kommentare deutsch und ASCII (ae/oe/ue).
- Konstanten statt Zahlen im Code, Domänenwerte mit Einheit im Namen oder Kommentar.
- Zu jedem neuen Verhalten ein Test: Unit-Test im Modul (`#[cfg(test)]`), bei Netzwerk-/Protokollverhalten Ende-zu-Ende in `tests/`.
- Vor dem Abschluss: `cargo test` und `cargo clippy --all-targets -- -D warnings` im betroffenen Projekt (`--manifest-path <projekt>\Cargo.toml`); bei Tauri zusätzlich `cargo build`, weil Clippy nicht linkt.
- Das Repository bleibt plattformneutral: keine Windows-spezifischen Pfade oder Flags in eingecheckter Konfiguration.
- Abhängigkeiten sparsam hinzufügen (`cargo add`), TLS-Features nur wenn nötig (Moonraker ist meist `ws://`).
