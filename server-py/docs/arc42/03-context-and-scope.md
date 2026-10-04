# 03 — Context & Scope

> arc42 §3 · The system embedded in its environment: business + technical context, scope.
> **Revision:** 2026-09-05

## 3.1 Business context

World-Office is embedded in a self-hosted document workflow. The user's
facts: their office documents live in **OpenCloud (OCIS)**, which is the
system of record for files and identity. World-Office is a **document
editing service**: it does not store the user's canonical files, does not
manage accounts, and is not itself a file browser.

```
┌────────────────────────────────────────────────────────────────┐
│                        End user (browser)                      │
│   browses files in OpenCloud · clicks a .docx / .odt → Edit    │
└───────────────────────────────┬────────────────────────────────┘
                                │
              ┌─────────────────▼───────────────────────┐
              │            OpenCloud (OCIS)             │
              │  system of record: files, users, auth   │
              │  launches World-Office as its WOPI app  │
              └─────────────────┬───────────────────────┘
                                │ WOPI (access token, file bytes)
              ┌─────────────────▼───────────────────────┐
              │         World-Office docserver          │
              │   (the system described by this doc)    │
              │   edits Office docs through OpenCloud   │
              └─────────────────────────────────────────┘
```

Business goals served: **sovereignty** (files never leave the operator's
infrastructure), **simplicity** (a single maintainable service), and
**interoperability** (standard WOPI + OpenDocument formats).

People/systems interacting with the system:

| Neighbor | Direction | Purpose |
|----------|-----------|---------|
| End user (browser) | → | Opens editor, edits, saves, collaborates, reviews AI work |
| OpenCloud (OCIS) | ←→ | WOPI host: provides file bytes + access tokens; receives saved content |
| OpenCloud admin | → | Registers World-Office as app provider (via `register_wopi_provider.py` or OpenCloud env config) |
| Ops / monitoring | → | Health checks, logs, Grafana panel |
| AI model provider (optional) | ← | Model-agnostic: agent loop calls a *model callable*; no vendor SDK baked in |

## 3.2 Technical context

The docserver plays a **dual WOPI role**:

1. **WOPI client** (production, OCIS-launched): OCIS is the WOPI *host*;
   World-Office is the WOPI *client/editor*. OCIS redirects the browser to
   the editor with an `access_token`; the docserver fetches and forwards
   file bytes to/from OCIS's WOPI service.
2. **WOPI host** (local dev/tests): the docserver itself implements the WOPI
   host surface over its local SQLite store, so the whole loop runs with
   zero external services.

```
          ┌──────────── browser ────────────┐
          │  editor (vanilla JS, PWA)       │  web assets / REST / SSE
          └──────────────┬──────────────────┘
                         │ HTTP  editor+API+WOPI
          ┌──────────────▼──────────────────┐
          │  opencloud-docserver (FastAPI)  │
          │  ─ WOPI host   (local store)    │  ── local mode
          │  ─ WOPI client (OCIS mode)      │
          └──────┬──────────────────┬───────┘
                 │ WOPI client      │            ┌──────────────────┐
                 │                  └───────────►│  OpenCloud OCIS   │
                 │                               │  WOPI host        │
                 │ (no host)                     └──────────────────┘
          ┌──────▼──────┐
          │ SQLite +    │
          │ content dir │
          └─────────────┘
```

### 3.2.1 Adjacent systems & their protocols

| System | Protocol / contract | Notes |
|--------|---------------------|-------|
| OpenCloud (OCIS) | WOPI (CheckFileInfo, GetFile, PutFile, Lock/Unlock/RefreshLock/GetLock); OCS API (app-provider registration) | Token via JWT `HS256`; discovery XML served by `/hosting/discovery` |
| Browser (editor) | HTTP + REST (`/api/documents/...`), **SSE** (`/collab/stream`), PostMessage bridge | Service worker enables PWA caching |
| AI agent harness | **MCP over stdio** (JSON-RPC 2.0, newline-delimited) | `uv run python -m src.ai.mcp`; or direct `AgentRunner` in-process |
| Prometheus/Grafana (ops) | HTTP `/health`, custom Grafana panel | `grafana/docserver-health.json` |
| Docker / Traefik | container + HTTP routing | See [07 — Deployment](07-deployment-view.md) |

### 3.2.2 External data stores / infra used by the system

- **SQLite** file (`data/docserver.db`) + content directory (`data/documents/`)
  with per-document version snapshots (`data/documents/versions/<id>/`).
- No external message bus, no Redis, no object storage; OCIS remains the
  object store for real documents in production mode.

## 3.3 Scope

### In scope (whitelist)

- WOPI protocol surface: discovery, check-file-info, get/put contents,
  lock/unlock/refresh-lock/get-lock.
- DOCX and ODT conversion to/from editable HTML (server-side).
- Browser editor: formatting toolbar, headings, lists, tables, links,
  images, undo/redo, autosave, read-only mode, i18n labels.
- Real-time collaboration: character-level CRDT, op log replay, SSE stream,
  presence.
- AI agents: tool surface (read/apply-ops/versions/lock/presence), agent
  runner with budgets, reviewable/revertible ops, MCP server.
- Versioning: snapshots per save, list + restore.
- Export: HTML, DOCX, ODT, PDF (weasyprint or minimal fallback).
- Health/monitoring, config, Docker + systemd packaging.

### Out of scope (blacklist — and why)

| Excluded | Reason |
|----------|--------|
| Spreadsheets / presentations / PDF editing | Focus (G3); OpenCloud routes only word docs to us today |
| Print-fidelity pagination | Editor is web-native, not a print preview ([04](04-solution-strategy.md)) |
| Separate collaboration service | Deferred to OCIS; local CRDT hub is enough (C-PRO-4) |
| Authentication / user management | Owned by OCIS; docserver trusts its JWTs |
| Object storage / file sync | Owned by OCIS |
| Mobile/desktop native apps, Tauri shell | Web editor only |
| The deprecated Rust/TypeScript stack | Reference only; not maintained |
| Kubernetes, service mesh, DB clusters | Contradicts every Stoic constraint |
| Vendor-specific AI SDKs | Model-agnostic `AgentRunner` only |

### Scope boundaries at a glance

```
Everything the docserver does ---------- everything it does NOT do
 edit DOCX/ODT via WOPI+OCIS    |   spreadsheets, slides, PDF editing
 real-time collab (CRDT)        |   auth/user mgmt (OCIS owns it)
 AI agents (revertible)         |   native/mobile apps
 versions + export              |   distributed orchestration
 local WOPI host mode (dev)     |   proprietary office engines
```
