## Context

Der Paritäts-Nachweis läuft heute über `wo-test-harness/census/`: Playwright
läuft beide Editoren ab und joint die Controls in `ledger.json`
(real/covered/stub/deferred). Die Join-Logik ist tokenbasiert (Präsenz), und
"covered" fasst OO-Dialoge mit vielen Sub-Controls zu je einem WO-Control
zusammen — Sub-Features gehen verloren (152 solcher Einträge). Verhaltens-
tests existieren nur als Ad-hoc-Probes in `/tmp` (Menü-Hit, Tab-Switch,
File-Menü, First-Open) aus Debug-Sessions. Unit-Layer (editor-common, 335
Tests) und API-Layer (wo-docserver, 49 Tests) sind stabil und bleiben
unangetastet. Das UI-Quellverzeichnis
`apps/web/apps/documenteditor-wysiwyg/` hat stabile Control-Ids; Commands
laufen zentral über `emitCommand(cmd, value)` → `runCommand`.

## Goals / Non-Goals

**Goals:**
- Effekt-Verträge je OO-Control, maschinell ausführbar in < 5 min
- Manifest aus bestehendem Census generiert (kein Doppel-Pflegen)
- Probes aus /tmp als repo-Persistenz (L4) mit explizitem Contract-Bezug
- Ledger-Status an Contract-Ergebnis gekoppelt (real = grün)

**Non-Goals:**
- Neue Test-Frameworks (Playwright + node:test reichen; kein Jest-Add)
- Sheet/Slide/PDF/Diagram-Editoren (nur Word; Muster später kopierbar)
- Visuelle Pixel-Vergleiche im Contract-Layer (bleibt L5/reconcile)
- Änderungen an editor.js-Command-Semantik

## Decisions

- **Manifest-Generierung statt Handpflege**: `parity:manifest` liest
  census-oo.json + census-wo.json, joint wie census-diff.py, erlaubt Overrides
  in `word-parity-overrides.json` (Effektklasse, Assertion, Sub-Features).
  *Alternative*: Handliste — verwirft sich bei jedem OO-Update selbst.
- **Effektklassen statt generischem DOM-Diff**: fünf Klassen mit je einem
  schmalen Beobachter (computed style am Selektionankер; MutationObserver auf
  #editor; Dialogsichtbarkeit + Kind-Selektoren; Ops-Endpunkt-Zählung;
  Viewport-Flags). *Alternative*: Screenshot-Diff — zu langsam und brittle
  für L2; visuell bleibt L5.
- **Dialog-Verträge als Sub-Feature-Listen**: ein `dialog`-Contract zählt die
  aktiven Steuerelemente (Selektor-Liste aus dem Override) — genau hier
  verliert "covered" heute Information. Sub-Feature-Liste entsteht einmalig
  pro Dialog aus dem OO-Original (manuell abgelaufen, dann stabil).
- **Contract-Ausführung im echten Browser, aber ohne Visual**: Playwright
  headless gegen den Testserver (Recipe: `JWT_SECRET=t DOCSERVER_PORT=18112
  DOCSERVER_DATA_DIR=$(mktemp -d) EDITOR_UI_DIR=<build>/word`); ein Browser
  für alle Cases, Reset je Case via Ops-Count + Undo statt Reload wo
  möglich. *Alternative*: jsdom — editor.js nutzt Canvas/execCommand, jsdom
  fliegt.
- **Ledger-Kopplung ohne Umbau von census-diff.py**: der Contract-Runner
  schreibt `contract-results.json`; census-diff.py joined dieses Ergebnis
  beim nächsten Lauf (kleiner Patch, keine Architekturänderung).
- **Probe-Persistenz**: die vier /tmp-Probes werden nach
  `server/tests/parity/surfaces/*.cjs` überführt und über ein gemeinsames
  Boot-Modul (`_boot.cjs`, Serverstart + Port-Wait) geteilt; Ausführung via
  `npm test`-Präfix in server/tests.

## Risks / Trade-offs

- [Flaky UI-Contracts (Fokus, Animationen)] → Runner setzt `prefers-reduced-motion`,
  feste Viewports, explizite Waits auf Effekt (nicht auf Zeit); Retry 1× nur
  bei nicht-deterministischen Klassen.
- [Census-Join-Divergenz (census-diff.py vs. Manifest-Generator)] → Generator
  importiert die Token-Logik als Port; reconcile --check vergleicht beide
  Outputs und schlägt bei Divergenz fehl.
- [Ops-Endpunkt als Op-Beobachter ist Implementierungsnähe] → Klasse `op`
  ist die Minderheit (Kollab-Features); Spec-Level bleibt "Collab-Op tritt auf".
- [OO-Updates ändern Blueprint laufend] → Census-Drift-Regel (Spec
  test-pyramid) trennt Warnung von Block.

## Migration Plan

1. Manifest-Generator + Overrides-Skelett (L2-Artefakte, noch ohne Gate)
2. Contract-Runner mit 5 Pilot-Contracts (bold, italic, insertTable,
   paragraph-dialog, protect) — grün zyklieren
3. Probes-Persistenz (L4) + Boot-Modul
4. census-diff.py-Kopplung + `contracts`-CI-Stage
5. Manifest-Vollausbau je Tab (11 Tabs, maschinell vorgeneriert, manuell
   klassifiziert) — der einzige langfristige Teil
Rollback: CI-Stage deaktivieren; Manifest/Probes sind additive Dateien.

## Open Questions

- (keine — Klassifikationsaufwand je Dialog wird in Task T-005 geschätzt,
  bevor die Vollausbau-Tasks freigegeben werden)
