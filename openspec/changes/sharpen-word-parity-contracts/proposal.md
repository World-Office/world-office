## Why

Der OnlyOffice-Paritäts-Census misst **Präsenz, nicht Verhalten**: ein Button
gilt als "real", wenn sein Token in `editor.js` auftaucht, und OO-Dialoge mit
je 20+ Controls werden zu einem WO-Control zusammengefasst ("covered") —
verbatim 152 Einträge. Das Ergebnis (Ledger: stubs 0, deferred 0) widerspricht
der gelebten Erfahrung: Funktionen sind trotzdem nicht alle abgebildet. Es
gibt keine verbindliche Ebene, die garantiert, dass ein Command auch das
richtige Dokument-Ereignis erzeugt, und die E2E-Probes leben ad-hoc in /tmp
statt als reproduzierbare Gates im Repo. Gefordert ist das Dreieck
**Spec (Was) → Contract (Wie verbindlich) → Testpyramide (Wo verifiziert)**.

## What Changes

- **Spec**: Capability-Matrix aus dem OO-Blueprint (`census-oo.json`, 11 Tabs,
  372 Controls) als verhaltensbasierte Requirements — je Capability ein
  Szenario "Command X erzeugt beobachtbaren Effekt Y" statt Button-Präsenz.
- **Contract**: Maschinenlesbares Manifest `word-parity-contracts.json`
  (generiert aus census-oo.json × census-wo.json), je Eintrag: OO-Control,
  WO-Selektor, Effektklasse (`format` / `structure` / `dialog` / `op` /
  `view`), Assertion. Dazu ein Contract-Runner (Playwright, headless, schnell
  — kein Visual Diff), der das Manifest ausführt.
- **Testpyramide**: Fünf Layer als dokumentierte, im CI laufende Gates —
  L1 Unit (editor-common/Rust, existiert), L2 Contract (neu), L3 Integration
  (wo-docserver API, existiert), L4 E2E-Surface (Probes aus /tmp
  persistiert: Menü-Hit-Tests, Tab-Switch, File-Menü, First-Open), L5 Visual
  (census/reconcile-Goldens, existiert).
- **Ledger-Regel verschärft**: Status `real` nur noch, wenn der zugehörige
  Contract-Test grün ist; `covered` nur mit expliziter Sub-Feature-Liste im
  Manifest. Presence-only bleibt nicht mehr grün.

## Capabilities

### New Capabilities
- `word-editor/parity-contracts`: Verhaltensverträge für alle Word-Ribbon-
  Commands — Manifest-Format, Runner-Semantik, Effektklassen, Gate-Regeln.
- `word-editor/test-pyramid`: Die fünf Test-Layer als Requirements — was jeder
  Layer verifizieren muss, welche Gates im CI hängen, wie Layer ausfallen
  (was zählt als Regression, was als Census-Drift).

### Modified Capabilities
- (keine — `openspec/specs/` ist leer; dies ist der erste Spec-Durchstoß)

## Impact

- `server/tests/` — neuer Contract-Runner + persistierte E2E-Probes
  (Playwright, global installiert; Testserver-Recipe wie in AGENTS-Notizen).
- `wo-test-harness/census/` — census-diff.py liest künftig das Manifest für
  die verschärften Status-Regeln (Ledger-Regenerierung).
- `apps/web/apps/documenteditor-wysiwyg/` — keine Funktionsänderung; nur
  Selektor-Stabilität (IDs sind bereits stabil).
- CI — neues Job-Stage `contracts` (Node + Playwright + Testserver, < 5 min);
  keine Produktions-Dependencies.
