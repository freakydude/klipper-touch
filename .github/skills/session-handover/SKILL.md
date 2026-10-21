---
name: session-handover
description: 'Use at the start of a new chat, at the end of a work package, when the user says merk dir / Stand / Übergabe, or when new rules or insights came up: where to look for the project state, what goes into memory vs skills vs instructions vs docs, a handover checklist, and the routine that keeps the skills current (check script, change proposals). Triggers: Übergabe, neuer Chat, Stand festhalten, merk dir, Skills aktuell, Skill-Pflege, Memory, Checkliste.'
---

# Übergabe und Skill-Pflege

## Beginn eines Chats

1. Memory ansehen: `/memories/` und `/memories/repo/` (Index: `00-index.md`), relevante Dateien lesen.
2. `.github/copilot-instructions.md` und die Skill-Beschreibungen lesen (werden automatisch angeboten); passende Skills laden, bevor Vorgehen erfunden wird.
3. Dem Nutzer in wenigen Sätzen den Stand und die offenen Punkte nennen und fragen, womit es weitergeht. Zurückgestellte Themen (z. B. P1-Inkrement) nicht vergessen.

## Wohin gehört was

| Ort                                                               | Inhalt                                                                           |
| ----------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| Memory `/memories/repo/`                                          | Projektstand, Entscheidungen, offene Punkte, Lehren, Messwerte; kurz und datiert |
| Memory `/memories/` (Nutzer)                                      | persönliche Vorlieben, Arbeitsumfeld, projektübergreifende Regeln                |
| Skill (`.github/skills/`)                                         | wiederholbare Verfahren mit Skripten, Fehlertabellen, geprüfte Fakten            |
| Instructions (`.github/instructions/`, `copilot-instructions.md`) | Regeln, die beim Bearbeiten bestimmter Dateien oder immer gelten                 |
| `docs/` und README                                                | Wissen für Menschen im Team                                                      |
| Code-Kommentar                                                    | nur Absichten, die der Code nicht zeigt                                          |

## Checkliste am Ende eines Arbeitspakets

1. Stand in der passenden Repo-Notiz aktualisieren (nicht doppelt anlegen); `00-index.md` bei neuen Dateien anpassen.
2. Neue Regel oder Erkenntnis? Als Skill- oder Instruction-Änderung vorschlagen (siehe unten).
3. Offene Punkte, Annahmen und Nicht-Geprüftes ausdrücklich benennen.
4. Temporäre Dateien, Prozesse (`klipper-touch`, `fake-moonraker`, Edge mit Debug-Port) und Testordner aufräumen.
5. Prüfen: `cargo test`/`clippy` der geänderten Crates, `npm run check`, bei UI-Änderung `webview-ui-check`.
6. `powershell -File .github\skills\session-handover\scripts\check-skills.ps1` ausführen.
7. Kurze Übergabe an den Nutzer: was ist fertig, was ungeprüft, was wird als Nächstes vorgeschlagen.

## Skills aktuell halten

- **Regelmäßig prüfen:** nach größeren Arbeitspaketen und vor einer Übergabe das Check-Skript laufen lassen ([check-skills.ps1](./scripts/check-skills.ps1)): Frontmatter, Namen, tote Pfade und Links. Danach inhaltlich stichprobenartig: laufen die genannten Befehle noch, stimmen Zahlen und Fakten, sind Skripte noch kompatibel (z. B. UI-Check nach UI-Umbau)?
- **Bei neuen Erkenntnissen:** entsteht im Chat eine Regel, ein Stolperstein oder ein wiederholbarer Ablauf, am Ende der Antwort einen kurzen "Skill-Vorschlag" machen (betroffener Skill, geplante Änderung in ein bis zwei Zeilen).
- **Kleine Korrekturen** (toter Pfad, falscher Befehl) direkt vornehmen und erwähnen. **Inhaltliche Änderungen, neue Skills, Löschungen** nur nach Zustimmung des Nutzers.
- Skills schlank halten (unter 500 Zeilen), Verweise einstufig, Beschreibung mit Auslösewörtern; Fakten mit Quelle oder dem Vermerk "Annahme".
- Überholte Skills nicht stillschweigend löschen, sondern Ablösung vorschlagen.

## Format der Übergabe

- Stand (drei bis fünf Punkte), nicht geprüft/Annahmen, offene Fragen an den Nutzer, vorgeschlagene nächste Schritte, Skill-Vorschläge.
