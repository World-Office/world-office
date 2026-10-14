# word-editor/parity-contracts Specification

## Purpose
Definiert verbindliche Verhaltensverträge für den Word-Editor gegenüber dem
OnlyOffice-Blueprint: für jeden geerbten Control ist festgelegt, welcher
beobachtbare Effekt (Formatierung, Struktur, Dialog, Collabora-Op, View)
beim Auslösen eintreten MUSS — unabhängig davon, wie die UI-Präsenz aussieht.
Der Contract-Layer macht "real" im Paritäts-Ledger maschinell prüfbar.

## Requirements

### Requirement: Contract-Manifest als Single Source of Truth
Das System SHALL ein generiertes Manifest
`server/tests/parity/word-parity-contracts.json` pflegen, das JEDEZeile des
OO-Blueprints (census-oo.json: Tab, Control-Id, Label) auf genau einen
WO-Eintrag mappt mit: WO-Selektor, Effektklasse
(`format|structure|dialog|op|view|stub`), Assertion-Descriptor und
Pyramiden-Layer. Das Manifest SHALL aus Census + manuellen Overrides
(`word-parity-overrides.json`) regenerierbar sein (`npm run parity:manifest`).

#### Scenario: Manifest deckt den Blueprint vollständig ab
- **WHEN** das Manifest generiert wird
- **THEN** hat jeder Census-Eintrag aus census-oo.json genau einen
  Manifest-Eintrag (kein Missing, kein Duplikat), und der Generator bricht
  mit Exit != 0 ab, wenn Census-Einträge unmappt bleiben

#### Scenario: Effektklassen sind vollständig
- **WHEN** ein Manifest-Eintrag geprüft wird
- **THEN** trägt er genau eine Effektklasse und einen maschinell auswertbaren
  Assertion-Descriptor (kein Freitext)

### Requirement: Contract-Runner verifiziert Verhalten, nicht Präsenz
Der Contract-Runner (`server/tests/parity/contract-runner.cjs`) SHALL je
Manifest-Eintrag den WO-Control in einem echten Browser (Playwright,
Testserver-Recipe: JWT_SECRET=t, Port 18112, EDITOR_UI_DIR auf das gebaute
UI) auslösen und die behauptete Wirkung beobachten: `format` → berechneter
Style am Cursor/Selektion, `structure` → DOM-Mutation im #editor-Content,
`dialog` → sichtbares Dialog-/Popover-Element mit erwarteten Kind-Steuerelementen,
`op` → Collabora-Op im Outbox-/Ops-Endpunkt, `view` → Viewport-Zustand.

#### Scenario: Format-Contract (Beispiel Fett)
- **WHEN** der Runner den Control mit cmd `bold` auslöst
- **THEN** trägt der Text am Cursor nachfolgend `font-weight: 700` und der
  Save-Roundtrip enthält `w:b` im DOCX

#### Scenario: Dialog-Contract (Beispiel Absatz-Dialog)
- **WHEN** der Runner den Absatz-Dialog-Öffner auslöst
- **THEN** ist ein Dialog sichtbar, der OO's ParagraphSettings-Subfeature-
  liste (Einrückung, Abstand, Zeilenabstand, Ausrichtung, Kontrolle
  witwen/waisen) als aktive Steuerelemente enthält — jede fehlende
  Subfunktion ist ein Contract-Fail, kein "covered"

#### Scenario: Stub ist laut
- **WHEN** ein Control mit Effektklasse `stub` ausgelöst wird
- **THEN** zeigt der Editor einen sichtbaren "not available"-Hinweis und der
  Ledger zählt ihn als `stub` (nie still)

### Requirement: Ledger-Regel — real nur mit grünem Contract
Der Paritäts-Ledger SHALL einen Eintrag nur dann als `real` führen, wenn der
zugehörige Contract-Test grün ist; `covered` NUR mit der expliziten
Sub-Feature-Liste im Manifest; alles andere fällt auf `stub` oder
`unverified` und lässt `reconcile.py --check` rot werden.

#### Scenario: Presence bleibt nicht grün
- **WHEN** ein Button existiert (Census real) aber sein Contract-Test fehlschlägt
- **THEN** führt der Ledger den Eintrag als `unverified`, und das Gate schlägt fehl

#### Scenario: Kein stiller Rückschritt
- **WHEN** ein Commit einen Contract von grün auf rot dreht
- **THEN** schlägt das `contracts`-CI-Gate fehl und blockiert den Merge
