---
name: drawio-diagrams
description: 'Use when creating, extending or reviewing draw.io diagrams in docs/ (e.g. UseCases and Dependencies.drawio): add a page, generate cells by script, avoid layout overlaps, validate the XML and render a page to a PNG with the real draw.io viewer. Triggers: drawio, draw.io, Diagramm, Workflows, Seite hinzufügen, Legende überlappt, Use Cases, mxGraph.'
---

# draw.io-Dateien bearbeiten und prüfen

## Format der Dateien

- `.drawio` ist unkomprimiertes XML (`<mxfile pages="N">` mit je `<diagram id name>` und `<mxGraphModel>`), CRLF-Zeilenenden, 4 Leerzeichen Einrückung, kein BOM.
- Neue Seiten vor `</mxfile>` einfügen und `pages` erhöhen; bestehende Seiten niemals verändern (nach dem Einfügen prüfen, dass die Datei mit dem alten Inhalt beginnt).
- Zellen-IDs pro Seite eindeutig und mit festem Präfix (z. B. `wf-`).
- Container als `swimlane` mit Kind-Zellen (`parent=<container>`, relative Koordinaten), damit Verschieben in draw.io zusammenhält. Kanten bekommen den Container als `parent`.
- Text mit Umbrüchen: `html=1` und `<br>` als `&lt;br&gt;` im Attribut; alle Sonderzeichen escapen.

## Vorgehen für eine neue Seite

1. Inhalt als Datenstruktur (Lanes, Workflows, Schritte, Tags) in einem kurzen Node-Skript halten und daraus die XML-Zellen erzeugen; Layout aus Konstanten berechnen (Breite, Schritthöhe, Abstände).
2. Vor dem Schreiben eine Sicherung ziehen, danach als XML laden (`[xml]` in PowerShell), Seitenanzahl und doppelte IDs prüfen.
3. Rendern und ansehen (siehe unten). Das Skript nach Gebrauch löschen oder bewusst ins Repo legen.

## Layout-Regeln aus Erfahrung

- Die Legende beansprucht Höhe: erste Lane erst unterhalb der Legende beginnen lassen (Legende mit n Einträgen belegt ca. 42 + n·24 px).
- Tags und Schritttexte kurz halten (Tag-Breite ca. 190 px bei Schrift 10), sonst brechen sie um.
- Alle Lane-Köpfe gleich breit, Tabellen als Raster aus Rechtecken.

## Rendern mit dem echten Viewer

1. Viewer-Skript einmalig nach `%TEMP%` laden (nicht ins Repo, 4,5 MB): `curl.exe -sS --ssl-no-revoke -o $env:TEMP\viewer-static.min.js https://viewer.diagrams.net/js/viewer-static.min.js`
2. `node .github\skills\drawio-diagrams\scripts\render-drawio.mjs "<datei.drawio>" <seitenindex ab 0> "$env:TEMP\viewer-static.min.js" "$env:TEMP\seite.png"`
3. PNG ansehen. Das Skript startet einen eigenen headless Edge (eigenes Profil, Debug-Port 9333) und beendet ihn wieder. Die Tauri-App wird dafür nicht gebraucht.
4. Temporäre Dateien löschen.

## Inhaltliche Konventionen im Projektdiagramm

- Seite "Workflows": Workflow zuerst, Fähigkeiten (Tags) nur als Bedingung; Prioritätsvorschlag P1–P3 pro Workflow; Legende und offene Fragen auf der Seite.
- Fähigkeitsquellen: Klipper-Objekte (`printer.objects.list`), Moonraker-Komponenten (`server.info`), Konfiguration (`configfile.settings`).
