---
description: "Use when writing or changing the Slint UI in crates/app/ui (*.slint): globals, pages, components, touch rules, icons, translations, how to verify with ui-snapshot."
applyTo: "**/*.slint"
---

# Slint-Oberfläche

- Nach der [Slint-Doku](https://docs.slint.dev/latest/docs/slint/). Veraltete Properties meldet der Compiler als Warnung; sofort ersetzen (z. B. `content-y` statt `viewport-y`).
- Daten nur über Globals aus `state.slint`: `Printer` (von Rust geschrieben, in der UI nur lesen), `UiSettings`, `Navigation`. Bedienungen nur über `Actions`-Callbacks oder `Confirm.ask(...)`; keine Drucker-Logik (G-Code, Grenzwerte) in `.slint`.
- Neue Globals/Structs in `app.slint` exportieren, sonst sieht Rust sie nicht.
- Farben, Abstände, Schriftgrößen nur aus `Theme`; Bedienflächen mindestens `Theme.touch` (56 px). Layout für 800x480 quer, muss bis 640x400 bedienbar bleiben; kein Scrollen als einziger Weg (Flickable plus Auf/Ab-Knöpfe).
- Gefährliche oder irreversible Aktionen (Abbruch, Not-Aus, Host, Updates, Makros) mit `Confirm.ask`.
- Texte englisch mit `@tr("...")`; Platzhalter mit `{}`.
- Icons nur aus `Icons` (`icons.slint`, generiert) mit `colorize`; neue Icons nach Skill `icon-set`.
- Software-Renderer kann keine Rotation/Skalierung und `drop-shadow` nur eingeschränkt; flach gestalten.
- Prüfen: `cargo build -p klipper-touch-slint`, dann Skill `ui-snapshot` und die PNGs ansehen (Überlauf, abgeschnittene Texte, Lesbarkeit).
