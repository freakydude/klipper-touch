---
description: "Use when writing or changing Rust code in this repository (crates/core, crates/app, crates/icon-index, fake-moonraker): Clean Code Developer rules, Rust Book and Clippy conventions, error handling, tests, module boundaries."
applyTo: "**/*.rs"
---

# Rust-Code

- Clean Code Developer: sprechende Namen, kleine Funktionen mit einer Aufgabe, DRY, KISS, Trennung der Verantwortlichkeiten, Boy-Scout-Regel. Neue Logik in eigene Module statt in `main`/`lib` zu stapeln.
- Fehler als eigene Typen oder `Result` mit sprechender Meldung; kein `unwrap`/`expect` außerhalb von Tests, außer bei echten Invarianten (dann mit Begründung im `expect`).
- Kommentare nur für Absichten, die der Code nicht zeigt; eine kurze Zeile, deutsch.
- Konstanten statt Zahlen im Code, Domänenwerte mit Einheit im Namen oder Kommentar.
- Zu jedem neuen Verhalten ein Test: Unit-Test im Modul (`#[cfg(test)]`), bei Netzwerk-/Protokollverhalten Ende-zu-Ende in `tests/`.
- Vor dem Abschluss: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings` und `cargo test --workspace`; bei der App zusätzlich `cargo build`, weil Clippy nicht linkt.
- Schichten: `crates/core` kennt weder Slint noch UI-Begriffe; Drucker-Logik (G-Code, Moonraker-Methoden, Grenzen) gehört in den Kern, `crates/app` setzt nur um und formatiert (`view.rs`/`format.rs` testbar ohne Slint).
- Slint-Typen sind nicht `Send`: UI-Zugriffe nur im Ereignisloop (`slint::spawn_local`, `upgrade_in_event_loop`), Netzwerk im Tokio-Laufzeitsystem des Kerns.
- Das Repository bleibt plattformneutral: keine Windows-spezifischen Pfade oder Flags in eingecheckter Konfiguration.
- Abhängigkeiten sparsam hinzufügen (`cargo add`), TLS-Features nur wenn nötig (Moonraker ist meist `ws://`).
