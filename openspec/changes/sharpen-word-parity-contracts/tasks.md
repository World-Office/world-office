## 1. Contract-Artefakte (L2-Basis)

- [x] 1.1 Manifest-Generator `server/tests/parity/generate-manifest.cjs`: joint census-oo.json × census-wo.json (Port der census-diff.py-Token-Logik), schreibt `word-parity-contracts.json`; bricht bei ungemappten Census-Einträgen mit Exit 1 ab — Verifikation: Generator läuft grün gegen heutigen Census
- [x] 1.2 Overrides-Datei `server/tests/parity/word-parity-overrides.json` mit Schema (effektClass, assertion, subFeatures[], note) + 5 Pilot-Einträgen (bold, italic, insertTable, paragraphDialog, protectDocument) — Verifikation: Schema-Validierung im Generator schlägt bei kaputtem Override fehl
- [x] 1.3 Contract-Runner `server/tests/parity/contract-runner.cjs`: Boot-Modul (Testserver 18112 + EDITOR_UI_DIR + Port-Wait), Effekt-Beobachter für format/structure/dialog/op/view, Ausgabe `contract-results.json` mit Control-Id/Tab/Klasse/erwartet vs. beobachtet — Verifikation: 5 Pilot-Contracts grün, absichtlicher Fails röten mit aktionsfähiger Meldung
- [x] 1.4 Dialog-Sub-Feature-Listen: OO-Original-Dialoge (Paragraph, Font, Table-Properties) einmalig ablaufen und aktive Steuerelemente als Selektor-Listen in Overrides eintragen — Verifikation: Paragraph-Dialog-Contract zählt existierende Sub-Features und meldet fehlende namentlich

## 2. E2E-Persistenz (L4)

- [x] 2.1 /tmp-Probes nach `server/tests/parity/surfaces/` überführen (menu-hit.cjs, tab-switch.cjs, file-menu.cjs, first-open.cjs) über gemeinsames `_boot.cjs`; je Probe Kommentarzeile mit gesichertem Contract-Ausschnitt — Verifikation: alle vier laufen lokal grün gegen den 18112-Testserver
- [x] 2.2 `npm run parity`-Skript in server/tests/package.json (run-parity.cjs: Runner + Surfaces, ein Aufruf) — Verifikation: `npm test -- parity` führt L2+L4 aus und exitet korrekt

## 3. Ledger-Kopplung + CI-Gates

- [x] 3.1 census-diff.py: `contract-results.json` einjoinen; Status `real` nur bei grünem Contract, sonst `unverified`; `covered` nur mit Sub-Feature-Liste; reconcile --check respektiert neue Status — Verifikation: Ledger mit Pilot-Ergebnissen regeneriert, Gate-Verhalten manuell geprüft (rot bei erzwungenem Fail)
- [x] 3.2 CI-Stage `contracts` (node + playwright + 18112-Testserver, < 5 min, L2+L4) in server/.github/workflows/ci.yml; nächtlicher L5-Census bleibt unverändert — Verifikation: Workflow grün auf einem Feature-Branch, rot bei injiziertem Fail
- [x] 3.3 Census-Drift-Regel: Runner behandelt Blueprint-Neuzugänge als `unmapped`-Warnung (non-blocking) mit Ledger-Frist-Eintrag — Verifikation: manipulierter Census (fake Control) erzeugt Warnung, Gate bleibt grün, Eintrag erscheint im Ledger

## 4. Vollausbau Manifest (machinell vorgeneriert, je Tab klassifiziert)

- [ ] 4.1 Klassifikationslauf: alle 372 Census-Einträge je Tab (11) durch Effektklassen + Assertion-Deskriptoren ergänzen, Dialog-Sub-Features für File-Backstage + Rechtsklick-Kontextmenü nachtragen — Verifikation: Manifest ohne `unclassified`, Runner-Laufzeit < 5 min
- [ ] 4.2 Gap-Report: Liste aller `stub`/`unverified`/Sub-Feature-Lücken als `plan/PARITY_GAPS.md` (Ausgangspunkt für spätere Feature-Tasks, ersetzt die implizite "covered"-Bilanz) — Verifikation: Report generiert, jede Lücke hat Tab+Control+fehlende Sub-Features
