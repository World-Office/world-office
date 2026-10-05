# 04 — Solution Strategy

> arc42 §4 · The fundamental decisions that shape the system, and the rationale.
> **Revision:** 2026-09-05

World-Office solves "edit office documents through OpenCloud" with one
deliberately boring, auditable architecture. The strategy rests on seven
pillars; each is a chosen alternative with a recorded alternative.

## 4.1 Pillar 1 — Stoic Python monolith, not a cathedral

**Chosen:** a single Python 3.12 + FastAPI process (~3,000 LOC, ~9 runtime
deps) replacing the Rust+TypeScript subsystem.
**Why:** the cathedral (26 crates, 13 TS packages, WASM, nightly Rust,
45-min builds) maximized complexity without user value. Boring wins: the
best editor is the one nobody has to patch at 3 AM.
**Rejected:** incremental rescue of the old stack; porting the WASM canvas
editor; a polyglot microservice fleet.
**Evidence:** `plan/RETHINK_WORLD_OFFICE.md` (metrics in §8).

## 4.2 Pillar 2 — DOCX/ODT ↔ HTML round-trip, not canvas rendering

**Chosen:** the server converts Office binary/zip formats into **editable
HTML** (`python-docx` for DOCX, `odfpy` for ODT); the browser edits that
HTML (contenteditable); on save the HTML is re-encoded server-side back to
the Office format.
**Why:** canvas/pagination rendering needs complex font layout, line
breaking and pagination engines. HTML gives editing "for free" and works in
every browser. The editor is a collaborative web page, not a print preview —
a deliberate trade for 90% of editing use cases.
**Rejected:** custom OOXML/WASM renderer (the cathedral's approach), or
LibreOffice-as-a-service (heavy, stateful).
**Consequence (owned):** conversion is *lossy for content outside the
mapped subset*; documented per converter (`src/editor/converter.py`,
`src/editor/odt_converter.py`). Round-trip fidelity is continuously measured
by differential/golden tests.

## 4.3 Pillar 3 — Vanilla-JS editor, not a framework

**Chosen:** one `editor.js`, `index.html`, `style.css`, `i18n.js` — zero npm
dependencies, zero build step, auditable.
**Why:** a contenteditable editor + toolbar needs no framework; 1 file
beats 15,000 files in `node_modules`. PWA (service worker + manifest) gives
offline caching without a bundler.
**Rejected:** React/Vue/Svelte shells, TipTap-style editor toolkit as the
*base* (kept as an evolution candidate in OpenSpec `cloud-editor-complete`).
**Consequence:** formatting fidelity is bounded by what contenteditable +
browser execCommand-like DOM ops deliver; the OpenSpec `editor-format-parity`
spec tracks the gap.

## 4.4 Pillar 4 — WOPI as the only integration contract

**Chosen:** speak the WOPI protocol in **both roles**: *host* (local
store, dev/test) and *client* (OCIS-launched, production). One code path,
two modes toggled by presence of a `wopi_host`/token.
**Why:** WOPI is the sanctioned way to edit files through OpenCloud; the
dual role makes the whole loop testable with zero external services and
keeps production path simple (forward bytes + token).
**Rejected:** proprietary APIs, direct database access to OCIS, editing on
the OCIS side.

## 4.5 Pillar 5 — CRDT, not OT or lock-only editing

**Chosen:** a **tombstone RGA sequence CRDT** (character-level) with a
per-document hub: Lamport-clock item ids, tombstoned deletes, deterministic
order, idempotent commuting ops, op-log replay for late joiners, live
delivery over **SSE** (+ polling fallback).
**Why:** any two concurrent edits commute → convergence proof without an
OT server; agents and humans share the *same* op stream; offline-friendly
(ops are just data).
**Rejected:** Operational Transform (server-order dependent, harder to
prove), full-document locking (no collaboration), third-party collab cloud.
**Evidence:** `src/editor/collab.py`; property/concurrency tests
(`test_collab_*`, `test_crdt_concurrent_edges.py`).

## 4.6 Pillar 6 — AI agents as attributable, revertible collaborators

**Chosen:** agent tool calls become **CRDT ops authored by a named agent
site**, so human edits, agent edits and agent rejections share one history.
An `AgentRunner` (model-agnostic callable) enforces step/op budgets; a
review surface lists agent ops with per-op **accept/reject** implemented as
pure op-stream operations (no parallel history). Optionally exposed as a
hand-rolled **MCP server over stdio** (JSON-RPC 2.0, ~200 lines).
**Why:** no new data model, provably revertible, auditable, vendor-free.
**Rejected:** a separate agent storage/state, vendor SDKs, non-attributable
edits.
**Evidence:** `src/ai/{runner,tools,review,mcp,schemas}.py`; `test_ai_*` suite.

## 4.7 Pillar 7 — SQLite ledger + files, one image, config-over-code

**Chosen:** SQLite keeps metadata, locks and version index; *content is
files* in a content dir (snapshots per save). Deploy as one Docker image
(or one systemd unit with hardening). Everything configurable via
`config.toml` overridden by `DOCSERVER_*` env vars.
**Why:** a storeroom ledger, not a database server; no daemon to run; file
backups are just `cp`.
**Rejected:** PostgreSQL (unneeded), object storage for content (OCIS owns
canonical bytes anyway), k8s.
**Evidence:** `src/lib/store.py`, `Dockerfile`, `docker-compose.yml`,
`systemd/`.

## 4.8 Strategy map (requirements → pillars)

| Requirement | Primary pillar(s) |
|-------------|-------------------|
| Edit DOCX/ODT via OpenCloud | 2, 4 |
| Simplicity / maintainability | 1, 3, 7 |
| Authoring collaboration | 5 |
| AI assistance w/ oversight | 5, 6 |
| Versioning & export | 2, 7 |
| Self-hosted sovereignty | 4, 7 |
| Testability | 2, 4, 5, 6 (differential, WOPI e2e, CRDT property, agent tests) |
