# word-editor/test-pyramid Specification

## Purpose
Legt die Testpyramide des Word-Editors als verbindliche Schichten fest: was
jede Schicht zu verifizieren hat, wie schnell sie laufen muss und welche Gates
im CI hängen. Ziel: jede Paritäts-Anforderung aus dem Contract-Manifest hat
genau einen nativen Layer, Regressionen werden auf der niedrigsten möglichen
Ebene gefangen.

## Requirements

### Requirement: Fünf Layer mit festen Zuständigkeiten
Der Test-Stack des Word-Editors SHALL aus genau fünf Layern bestehen:
L1 Unit (Op-Anwendung editor-common, Rust-Kernel-Crates; < 2 min, je Commit),
L2 Contract (Manifest-Runner, Verhalten je Control; < 5 min, je Commit),
L3 Integration (wo-docserver HTTP-API: collab/ops/protect/ai; je Commit),
L4 E2E-Surface (persistierte Playwright-Probes: Menü-Hit-Tests, Tab-Switch,
File-Menü, First-Open-Render; je Commit gegen Testserver), L5 Visual-Golden
(census/reconcile gegen OO-Referenz; nächtlich + auf Request).

#### Scenario: Keine Zweitprüfung auf höherer Ebene
- **WHEN** ein Verhalten auf Layer N getestet ist
- **THEN** prüft kein höherer Layer dasselbe Verhalten erneut (nur deren
  Zusammenspiel); Verstoße sind im PR als Duplikat-Test zu entfernen

### Requirement: E2E-Probes sind Repo-Artefakte
Die Playwright-Oberflächen-Probes SHALL als Dateien unter
`server/tests/parity/surfaces/` im Repo liegen (Boot-Rezept: Testserver
18112 + EDITOR_UI_DIR), nicht als /tmp-Skripte; jede Probe benennt
explizit den Contract-Ausschnitt, den sie absichert.

#### Scenario: Menü-Hit-Test liegt im Repo
- **WHEN** `npm test -- prefix=parity` in server/tests läuft
- **THEN** werden die Tab-Hit-Tests (elementFromPoint je Ribbon-Tab),
  der File-Menü-Open-Test und der First-Open-Render-Test ausgeführt und
  sind bei Regression rot

### Requirement: CI-Gates und Laufzeiten
L1–L4 SHALL je Commit im CI laufen (Stage `contracts` + bestehende Stages),
L5 nächtlich; jedes Gate SHALL bei Fehlschlag den Merge blockieren und die
fehlernden Manifest-Einträge namentlich melden.

#### Scenario: Gate-Meldung ist aktionsfähig
- **WHEN** ein Contract-Test rot schlägt
- **THEN** nennt die CI-Ausgabe Control-Id, Tab, Effektklasse und die
  erwartete vs. beobachtete Wirkung

### Requirement: Census-Drift von Regression getrennt
Der Runner SHALL unterscheiden: Contract-Fail (Verhalten kaputt — blockiert)
und Census-Drift (OO-Blueprint hat neue Controls — warnt, erzeugt
Manifest-Entries mit Klasse `unmapped`, blockiert erst nach
Kulanzfrist-Eintrag im Ledger).

#### Scenario: Neues OO-Update bringt Controls
- **WHEN** ein frischer OO-Census Controls enthält, die im Manifest fehlen
- **THEN** läuft das CI warnend mit `unmapped`-Liste, ohne die bestehenden
  Contracts zu deaktivieren
