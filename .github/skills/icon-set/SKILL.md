---
name: icon-set
description: "Use when icons are added, changed, removed or used in the klipper-touch Slint UI (assets/icons, crates/app/ui/icons.slint): naming scheme, style rules of the SVG set, how to add or change a single icon, the icon-index check/generator, colorize in Slint and state-to-icon mapping. Triggers: Icon, SVG, Symbol, Button ohne Text, neues Icon, Icon ändern, Stil, icons.slint, overview.html, Icon-Set."
---

# Icon-Set (`assets/icons`)

Die SVG-Dateien sind die einzige Quelle. Es gibt keinen Generator für die Zeichnungen; Icons werden einzeln hinzugefügt, geändert oder entfernt. `crates/icon-index` prüft sie und erzeugt nur die Liste für Slint.

## Ablage und Namen

- Datei: `assets/icons/<kategorie>/<kategorie>-<name>.svg`, kebab-case, z. B. `motion/motion-x-plus.svg`. Der Dateiname ist der Icon-Name und zugleich die Property in `Icons` (`Icons.motion-x-plus`).
- Kategorien: `ui`, `nav`, `print`, `motion`, `thermal`, `filament`, `file`, `state`, `system`, `calibration`, `accessory`, `mmu`. Neue Kategorie nur, wenn kein Bestand passt (= neuer Ordner mit gleichem Präfix).
- Benennung nach Moonraker/Klipper/Happy-Hare-Funktion, nicht nach dem aktuellen UI-Stand. Richtung/Variante als Suffix (`-plus`, `-minus`, `-xy`).

## Stilregeln

- Wurzelelement exakt: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">`, in einer Zeile, danach Elemente mit 2 Leerzeichen eingerückt, `</svg>`.
- Nur Linien, keine Füllungen; Punkte als `<path stroke-width="3" d="Mx yh.01"/>`.
- Nur `currentColor`/`none`: keine festen Farben, kein `style`, keine Skripte.
- Zeichnung innerhalb ca. 3..21; Buchstaben (X/Y/Z) gleich groß (4 breit, 6 hoch). Eigene Formen, keine Kopien aus KlipperScreen, Mainsail, Fluidd oder Icon-Bibliotheken.

## Ablauf: Icon hinzufügen oder ändern

1. SVG-Datei anlegen oder bearbeiten (Stilregeln oben); zum Entfernen Datei löschen und Verwendungen in `crates/app/ui` suchen (`Icons.<name>`).
2. `cargo run -p icon-index` schreibt `crates/app/ui/icons.slint` und `assets/icons/overview.html` neu.
3. `cargo test -p icon-index` (oder `cargo run -p icon-index -- --check`) muss 0 Fehler melden: Namensschema, Wurzelelement, Farben, Duplikate, `icons.slint` aktuell.
4. `overview.html` im Browser öffnen (schwarz auf weiß und weiß auf dunkel; in VS Code per "Simple Browser: Show" mit `file:///`-URL) und das Icon neben den Nachbarn beurteilen.
5. Bei UI-Verwendung `cargo build -p klipper-touch-slint` und Skill `ui-snapshot`.

Stiländerung für alle (z. B. Linienstärke): Wurzelelement in allen Dateien ersetzen und in `crates/icon-index/src/lib.rs` die Konstante `WRAPPER_OPEN` anpassen; die Prüfung meldet jede Abweichung.

## Verwendung in Slint

- `Image { source: Icons.motion-x-plus; colorize: Theme.foreground; width: 28px; height: 28px; }`; in Knöpfen `IconButton { icon: Icons.motion-x-plus; text: "X+"; }`.
- Ohne sichtbaren Text `accessible-label` setzen (macht `IconButton` über `text`).
- Zustände: `StateIcons.print-state(state)` und `StateIcons.klippy-state(connection, state)` in `crates/app/ui/state.slint`.
- Zerstörende Aktionen (Not-Aus, Abbruch) im Bestätigungsdialog mit Icon und Text. Zahlen-Buttons (Schrittweiten) bleiben Zahlen.

## Stolperfallen

- `icons.slint` nicht von Hand ändern, sonst schlägt der Test fehl.
- Formatter dürfen SVGs nicht umbrechen (Wurzelelement muss einzeilig bleiben); `.vscode/settings.json` schaltet Format-on-Save für SVG ab.
- Kein `fill` auf Pfaden setzen, sonst bricht der einheitliche Linienstil.
- Die MMU-, Kalibrierungs- und Zubehör-Icons sind teils noch ohne UI-Verwendung; vor dem Löschen prüfen, ob ein Workflow (`docs/UseCases and Dependencies.drawio`) sie braucht.
