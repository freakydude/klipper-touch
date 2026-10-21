---
name: icon-set
description: 'Use when icons are added, changed, removed or used in the Klipper-Touch UI (src/lib/icons): naming scheme, style rules of the SVG set, how to add or change a single icon, the check/index script, the Icon component and state-to-icon mapping. Triggers: Icon, SVG, Symbol, Button ohne Text, neues Icon, Icon ändern, Stil, icon-names, overview.html, Icon-Set.'
---

# Icon-Set (`src/lib/icons`)

Die SVG-Dateien sind die einzige Quelle. Es gibt keinen Generator; Icons werden einzeln hinzugefügt, geändert oder entfernt.

## Ablage und Namen

- Datei: `src/lib/icons/<kategorie>/<kategorie>-<name>.svg`, kebab-case, z. B. `motion/motion-x-plus.svg`. Der Dateiname ist der Icon-Name.
- Kategorien: `ui`, `nav`, `print`, `motion`, `thermal`, `filament`, `file`, `state`, `system`, `calibration`, `accessory`, `mmu`. Neue Kategorie nur, wenn kein Bestand passt (= neuer Ordner mit gleichem Präfix).
- Benennung nach Moonraker/Klipper/Happy-Hare-Funktion, nicht nach dem aktuellen UI-Stand. Richtung/Variante als Suffix (`-plus`, `-minus`, `-xy`).

## Stilregeln

- Wurzelelement exakt: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">`, danach Elemente mit 2 Leerzeichen eingerückt, `</svg>`.
- Nur Linien, keine Füllungen; Punkte als `<path stroke-width="3" d="Mx yh.01"/>`.
- Nur `currentColor`/`none`: keine festen Farben, kein `style`, keine Skripte. Farbe kommt aus der Textfarbe des Buttons.
- Zeichnung innerhalb ca. 3..21, damit sie bei Größe 1.5-2 rem lesbar bleibt. Eigene Formen, keine Kopien aus KlipperScreen, Mainsail, Fluidd oder Icon-Bibliotheken.

## Ablauf: Icon hinzufügen oder ändern

1. SVG-Datei anlegen oder bearbeiten (Stilregeln oben); zum Entfernen Datei löschen und Verwendungen suchen (`grep` nach dem Namen).
2. Mit `node` das Skript `scripts/icons/build-icon-index.mjs` ausführen: es schreibt `icon-names.ts` (Typ `IconName`) und `overview.html` neu.
3. Dasselbe Skript mit `--check` muss 0 Fehler melden (Namensschema, Wurzelelement, Farben, Duplikate, `icon-names.ts` aktuell).
4. `overview.html` im Browser öffnen (schwarz auf weiß und weiß auf dunkel) und das Icon neben den Nachbarn beurteilen; optional per headless Edge/CDP als PNG ablegen.
5. Bei UI-Verwendung `npm run check` und Skill `webview-ui-check`.

Stiländerung für alle (z. B. Linienstärke): Wurzelelement in allen Dateien ersetzen und in `build-icon-index.mjs` die Konstante `WRAPPER_OPEN` anpassen; das Prüfskript meldet jede Abweichung.

## Verwendung in der UI

- `<Icon name="motion-x-plus" label="Right" class="h-8 w-8" />` aus `$lib/icons/Icon.svelte`. Ohne `label` ist das Icon dekorativ (`aria-hidden`); in Buttons ohne sichtbaren Text immer `label` mit der alten Beschriftung setzen.
- Standardgröße `h-6 w-6` (1.5 rem), große Touch-Buttons `h-8 w-8`; nur `rem`, keine Pixel.
- Zustände: `printStateIcon(print_stats.state)` und `klippyStateIcon(webhooks.state)` aus `$lib/icons/stateIcons`.
- Zerstörende Aktionen (Kill, Cancel) in Bestätigungsdialogen mit Icon und Text zeigen. Zahlen-Buttons (Schrittweiten) bleiben Zahlen.
- Der Bestand ist per `import.meta.glob` geladen (`icons.ts`); neue Dateien brauchen keine weitere Registrierung.

## Stolperfallen

- `icon-names.ts` nicht von Hand ändern, sonst schlägt `--check` fehl.
- Kein `fill` auf Pfaden setzen, sonst bricht der einheitliche Linienstil.
- Bei dunklem Hintergrund bleibt `currentColor` sichtbar; feste Farben würden das verhindern.
- Die MMU-, Kalibrierungs- und Zubehör-Icons sind noch ohne UI-Verwendung; vor dem Löschen prüfen, ob ein Workflow (`docs/UseCases and Dependencies.drawio`) sie braucht.
