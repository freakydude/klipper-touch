---
name: workflow-review
description: "Use when a UI change, new screen or planned increment must be checked against the workflow diagram (docs/UseCases and Dependencies.drawio, page Workflows): does the UI cover each workflow completely, per print state, per capability (printer.objects.list), within touch constraints? Produces a coverage matrix and proposals. Triggers: Workflow prüfen, Abdeckung, Review UI, Inkrement planen, Use Case, Prioritäten P1, Druck-Screen Zustand, Capability, fehlender Schritt."
---

# Workflow-Review

Ziel: Der Workflow des Menschen steht vor der Technik. Dieser Review prüft, ob eine UI-Änderung (oder der Ist-Stand) die Workflows des Diagramms vollständig abdeckt und Fähigkeiten nur zum Ein-/Ausblenden von Schritten oder Varianten nutzt.

## 1. Soll einlesen

`[Console]::OutputEncoding = [Text.Encoding]::UTF8; node .github\skills\workflow-review\scripts\extract-workflows.mjs "docs\UseCases and Dependencies.drawio"`

- Gibt Lanes, Workflows (mit Priorität P1–P3), nummerierte Schritte, Markierung `[optional]`/`[Problem]` und Fähigkeits-Tags aus; außerdem die Tabelle "Druck-Screen je Zustand" und die offenen Fragen.
- Das Diagramm ist die Quelle der Wahrheit und wird vom Nutzer gepflegt: nie aus dem Gedächtnis arbeiten, immer neu einlesen. Rendern bei Bedarf mit dem Skill `drawio-diagrams`.

## 2. Ist erfassen

- Seiten und Bausteine: `crates/app/ui/pages/*.slint`, Rahmen in `crates/app/ui/components/chrome.slint` (Statusleiste, Navigation), Zustände/Callbacks in `crates/app/ui/state.slint`.
- Welche Moonraker-Objekte/Methoden genutzt werden: `crates/core/src/printer/subscription.rs` (Abo), `crates/core/src/commands.rs` (Befehle), Fähigkeiten in `crates/core/src/printer/capabilities.rs`.
- Verhalten je Zustand und je Profil mit dem Fake: Skill `ui-snapshot`; Profile `basic`, `voron`, `happy-hare` (Skill `fake-moonraker`).

## 3. Abdeckung bewerten

Je Schritt eines Workflows eine Zeile:

| Workflow | Schritt | Screen/Route | Status | Lücke / Vorschlag |
| -------- | ------- | ------------ | ------ | ----------------- |

Status: `vollständig`, `teilweise`, `fehlt`, `nicht nötig (begründen)`. Pro Zeile prüfen:

1. **Erreichbar:** in höchstens zwei Berührungen vom Hauptscreen; kein Scrollen oder Wischen als Standardweg.
2. **Zustandsgerecht:** passend zur Tabelle "Druck-Screen je Zustand" (Anzeigen, Anbieten, bewusst nicht anbieten).
3. **Fähigkeitsgerecht:** optionale Schritte erscheinen nur, wenn das Objekt in `printer.objects.list` bzw. die Komponente vorhanden ist; fehlende Fähigkeit führt zum Ausblenden oder zu einer Variante, nicht zu einem toten Button.
4. **Fehlerpfade:** Problem-Schritte (rot) vorhanden und verständlich; Emergency Stop auf jeder Seite; Verbindungs- und Shutdown-Zustand sichtbar.
5. **Rückmeldung:** laufende Aktionen zeigen Fortschritt oder Wartezustand.
6. **Touch:** Mindestgröße der Bedienelemente (`Theme.touch`), Lesbarkeit bei 800x480 (Skill `ui-snapshot`, optional `--size 640x400`).
7. **Priorität:** P1-Lücken zuerst; P2/P3 nur nennen.

## 4. Ergebnis

- Matrix plus höchstens fünf Vorschläge, nach Priorität sortiert, jeweils mit betroffenen Dateien.
- Widersprüche oder fehlende Workflows im Diagramm benennen und als Änderungsvorschlag an den Nutzer formulieren (Diagramm nicht ungefragt umbauen).
- Offene Fragen aus dem Diagramm nicht selbst beantworten, sondern dem Nutzer vorlegen.
- Kein neues Dokument anlegen, außer der Nutzer wünscht es; Stand in die Repo-Notiz im Memory übernehmen (Skill `session-handover`).
