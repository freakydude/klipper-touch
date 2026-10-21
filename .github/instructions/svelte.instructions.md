---
description: 'Use when writing or changing the SvelteKit frontend (Svelte 5 runes, TypeScript, Tailwind) in src/: reactivity rules, Moonraker stores, touch UI conventions.'
applyTo: 'src/**/*.svelte, src/**/*.ts'
---

# Svelte 5 / SvelteKit Frontend

- Nach Svelte-5-Doku (Runes). Keine State-Mutation in Templates oder `$derived`: kein `array.sort()` auf `$state` im `{#each}`; stattdessen `$derived([...liste].sort(...))`.
- Abgeleitete Werte mit `$derived`, nicht mit `$effect` plus `$state`. `$effect` nur für Nebenwirkungen; asynchrone Arbeit in eine benannte Funktion auslagern und mit `void` aufrufen.
- Mehrere Datensätze erst vollständig laden, dann einmal zuweisen (kein indexweises Füllen eines `$state`-Arrays aus parallelen Promises).
- Zeiten: Moonraker liefert Unix-Sekunden (`modified * 1000` für `Date`); vor Datumsformatierung auf `NaN`/Division durch 0 absichern.
- Moonraker-Zustand kommt aus den Stores in `MoonrakerClient`/`Values`; Seiten abonnieren nur, was sie brauchen (`definePrinterObjects`) und melden im `onDestroy` ab.
- Touch-Bedienung: große Buttons, kein Scrollen/Wischen als Standardweg, Größen in `rem` (Skalierung über `scaleUI` in `+layout.svelte`), keine festen Pixelmaße für Layout.
- Verfügbarkeit von Funktionen aus Fähigkeiten ableiten (`printer.objects.list`), nicht aus Annahmen; der Workflow des Menschen bestimmt die Struktur (siehe `docs/UseCases and Dependencies.drawio`).
- Nach Änderungen: `npm run check` (0 Fehler) und, wenn die UI betroffen ist, den Skill `webview-ui-check` ausführen. Prettier meldet im Bestand auch bei unveränderten Dateien Abweichungen (Zeilenenden); nicht ganze Dateien umformatieren.
