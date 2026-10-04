# 09 — Architectural Decisions

> arc42 §9 · Important architectural decisions (ADR style), newest first.
> **Revision:** 2026-09-05
> Status legend: ✅ decided & implemented · 🔶 decided, partially implemented · 🔮 candidate

---

## ADR-001 · Discard the Rust/TypeScript cathedral → Stoic Python rewrite

- **Status:** ✅ (2026-08-19, confirmed 2026-08-21)
- **Context:** 26 Rust crates, 13 TS packages, 8 web apps, 9 services, ~62k
  files; nightly Rust + WASM; 45-min builds; collaboration half-wired.
- **Decision:** A single Python 3.12 + FastAPI process at
  `server/opencloud-docserver/`, vanilla-JS editor, SQLite, uv. Old stack
  kept **unmodified** as reference only.
- **Consequences:** +simplicity/reliability; −feature breadth (only DOCX/ODT
  editing); roadmap must say *no* to non-core asks.
- **Evidence:** `plan/RETHINK_WORLD_OFFICE.md`.

## ADR-002 · DOCX/ODT ↔ HTML round-trip instead of canvas rendering

- **Status:** ✅
- **Context:** Canvas/WASM rendering requires layout, line-breaking and
  pagination engines (the old wo-pdf/wo-docx-renderer crates).
- **Decision:** `python-docx`/`odfpy` convert Office ↔ HTML server-side; the
  browser edits HTML and re-encodes on save. Editor is a web page, not a
  print preview; lossy beyond the mapped subset, documented per converter.
- **Consequences:** free editing via contenteditable; fidelity bounded —
  tracked by `editor-format-parity` OpenSpec; differential/golden tests
  guard the round-trip.

## ADR-003 · No framework — vanilla JS editor

- **Status:** ✅ (TipTap as an *evolution candidate* under
  `cloud-editor-complete`, not the base)
- **Context:** 15k npm files for a contenteditable editor is anti-Stoic.
- **Decision:** `web/editor.js` + CSS + `i18n.js`; zero npm deps; PWA.
- **Consequences:** auditable, instant load; richer formatting must be built
  or the Tiptap path adopted deliberately.

## ADR-004 · SQLite ledger + file-backed content

- **Status:** ✅
- **Context:** The docserver keeps metadata, locks, version index — not a
  warehouse.
- **Decision:** SQLite (`documents`, `versions` tables, RLock-serialized
  shared connection) + raw content files + per-save version snapshots.
- **Consequences:** no daemon/config; trivial backup; PostgreSQL remains a
  behind-the-same-SQL swap if ever needed.

## ADR-005 · Dual WOPI role: host (local) and client (OCIS)

- **Status:** ✅
- **Context:** Production requires speaking to OCIS as WOPI *host*; full
  dev/test loop wants zero external services.
- **Decision:** Implementation of full WOPI host surface over the local
  store *and* `RemoteWopiClient` forwarding for OCIS-launched sessions,
  switched by presence of `wopi_host`/token.
- **Consequences:** one codebase, two modes; session-scoped lock tokens
  prevent editors borrowing each other's locks.

## ADR-006 · CRDT (tombstone RGA) for collaboration — not OT, not lock-only

- **Status:** 🔶 (engine + hub live; editor surface completion in-flight)
- **Context:** OT needs centralized ordering and is hard to reason about;
  full-document locks kill collaboration.
- **Decision:** character-level RGA sequence CRDT; Lamport-id items;
  tombstoned deletes; deterministic ordering; idempotent commuting ops;
  per-doc hub with op-log replays and SSE delivery.
- **Consequences:** convergence provable; agent ops live in the same
  history; delivery may be lossy/reordered — handled by parking tables.

## ADR-007 · AI agents as attributable, revertible CRDT collaborators

- **Status:** 🔶
- **Context:** Agent edits must be reviewable and undoable without a
  parallel history model.
- **Decision:** every agent op carries site `agent=<name>`; accept/reject is
  a pure op-stream operation; runner is model-agnostic with step/op budgets;
  MCP-over-stdio (JSON-RPC 2.0) as the optional wire.
- **Consequences:** no vendor SDK, auditing by construction;
  `DOCSERVER_AGENTS=0` kills the whole surface.

## ADR-008 · Config over code; env-over-file precedence

- **Status:** ✅
- **Context:** Operators need to tune without code edits; twelve-factor.
- **Decision:** `config.toml` + `DOCSERVER_*` env override (env always
  wins), frozen after startup in `app.state.config`.
- **Consequences:** deployment differences are environment-only.

## ADR-009 · PyJWT as primary JWT implementation (HS256)

- **Status:** ✅
- **Context:** Two JWT libs were in the dependency set.
- **Decision:** PyJWT for encoding/verification robustness against
  algorithm-confusion (strict `algorithms=["HS256"]`); python-jose kept as
  build-level fallback only.
- **Consequences:** fewer surprising verify paths; secret shared with OCIS.

## ADR-010 · One Docker image + systemd; no Kubernetes

- **Status:** ✅
- **Context:** Stoic deployment: process manager, not orchestration.
- **Decision:** single `opencloud-docserver` image (service `docserver` +
  optional `traefik` edge); hardened systemd unit for non-Docker hosts.
- **Consequences:** trivial rollouts; HA/scale-out is explicitly out of
  scope for now.

## ADR-011 · Standard library first; size caps as governance

- **Status:** ✅
- **Context:** The cathedral grew because it was easy to add a crate, never
  to remove one.
- **Decision:** stdlib before dependency (sqlite3, urllib, logging,
  html.parser; hand-rolled MCP instead of an SDK); files ≤ 400 lines,
  functions < 40 lines, ≤ 3 nesting levels under `src/`; 7 Stoic merge
  checks (see [10 §10.4](10-quality-requirements.md)).
- **Consequences:** smaller surface, slower feature growth by design.

---

## Decision log notes

- The old deployment guide `DEPLOYMENT.md` describes the *deprecated* Rust
  gateway; treat it as historical.
- OCIS "UUID service discovery" quirk (memory 2026-09-04) is an *external*
  constraint; no ADR changes the docserver to chase it — the fix lives in
  compose/service registration (see [11 — Risks](11-technical-risks.md)).
