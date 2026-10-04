# 05 — Building Block View

> arc42 §5 · Whitebox decomposition — level 1 (system) and level 2 (docserver internals).
> **Revision:** 2026-09-05 · Paths relative to `server/opencloud-docserver/`.

## 5.1 Level 1 — System context decomposition

```
                        +-----------------------+
                        |  browser (editor SPA) |
                        +-----------+-----------+
                                    |
                     HTTP/REST · SSE · static assets
                                    |
                        +-----------v-----------+       WOPI (client mode)
                        |   opencloud-docserver  |<-----------------------+
                        |   (FastAPI process)    |                         |
                        +----+-------+-------+---+        +--------------+  |
                             |       |       |            | OpenCloud    |  |
                        +----v---+ +-v----+ +-v--------+  | (OCIS)       |  |
                        | SQLite | |files/| | AI/MCP   |  | WOPI host    |  |
                        | ledger | |content| | stdio   |  +--------------+  |
                        +--------+ +-------+ +---------+                    |
                        (WOPI host mode: OCIS wire role is absent; the       |
                         docserver serves the same WOPI surface locally)     |
                        +----------------------------------------------------+
```

Production mode is **client mode**: the docserver never stores the user's
canonical document — it fetches it from OCIS per session and forwards saves
back. Local **host mode** (SQLite + content dir) is the dev/test loop and
the fallback deployment.

## 5.2 Level 2 — Docserver internal building blocks

```
                       src/main.py — create_app() wiring
   ┌───────────────┬───────────────┬───────────────┬───────────────┬───────────────┐
   │ WOPI layer    │ Editor layer  │ Collab layer  │ AI layer      │ Support       │
   └───────┬───────┴───────┬───────┴───────┬───────┴───────┬───────┴───────┬───────┘
           │               │               │               │               │
   +-------+-------+ +-----+------+ +------+------+ +------+------+ +------+------+
   │ wopi/router   │ │ editor/     │ │ editor/     │ │ ai/          │ │ lib/        │
   │ router, auth, │ │ router,     │ │ collab.py   │ │ runner,      │ │ store,      │
   │ protocol,     │ │ converter,  │ │ RGA CRDT +  │ │ tools,       │ │ crypto      │
   │ testhost      │ │ odt_converter│ │ hub, SSE    │ │ schemas,     │ │             │
   │               │ │ sanitize,   │ │             │ │ review, mcp  │ │ config.py   │
   │               │ │ session     │ │             │ │              │ │ cli.py      │
   +---------------+ +------------+ +------------+ +--------------+ +------------+
                               Web assets: web/ {index.html, editor.js, i18n.js,
                               style.css, home.html, sw.js, manifest.json}
```

### 5.2.1 WOPI layer — `src/wopi/`

| Block | Responsibility | Key surfaces |
|-------|----------------|--------------|
| `router.py` | WOPI **host** surface (local store) | `GET/POST /wopi/files/{id}`, `/contents`, `/lock`, `/unlock`, `/refreshlock`, `/getlock`; content-type map `.docx/.odt/.txt/.md`; 128 MiB cap |
| `auth.py` | Token validation middleware | JWT HS256 verify, 401 on failure |
| `protocol.py` | WOPI types/signatures, errors | `WopiError`, lock headers `X-WOPI-Lock`/`X-WOPI-ItemVersion`, `invalid_doc_id()` path-traversal guard |
| `testhost.py` | Mock WOPI host for E2E | Local host used by tests/`e2e` |

### 5.2.2 Editor layer — `src/editor/`

| Block | Responsibility | Key surfaces |
|-------|----------------|--------------|
| `router.py` | HTTP API + editor page + collab/AI endpoints | `/editor`, `/hosting/discovery` (WOPI discovery XML), `/api/documents/*` (html, save, export, new, contents, versions, restore, lock/unlock, upload, list), `/collab/*`, `/ai/review`, `/ai/review/reject` |
| `converter.py` | DOCX ↔ HTML | `docx_to_html` / `html_to_docx` via `python-docx`; format routed by extension (`.docx` fallback) |
| `odt_converter.py` | ODT ↔ HTML | `odt_to_html` / `html_to_odt` via `odfpy`; images as `data:` URIs / `draw:frame`, `svg:title` alt text, tables incl. covered cells & ragged rows |
| `sanitize.py` | XSS defense on all HTML | Safe-tag allowlist, attribute stripping via `HTMLParser` rebuild |
| `session.py` | Editor session + OCIS forwarding | `EditorSession`, `SessionRegistry`, `RemoteWopiClient` (WOPI client role), lock-token carry-over |

### 5.2.3 Collab layer — `src/editor/collab.py`

| Block | Responsibility |
|-------|----------------|
| `TextCRDT` | Tombstone RGA: items `(site, seq)`, Lamport counters, deterministic `(seq, site)` ordering, pending-delete parking, `to_string()` |
| `CollabHub` | Per-document state: op log with monotonic **revision**, dedup/idempotency, late-join replay, seeding from stored doc as site `__base__`, SSE fan-out, presence |

Wire ops: `insert {t,s,b,n,chars,originSite,originSeq}` / `delete {t,s,ids}`.
Endpoints: `GET /collab/state`, `GET /collab/ops?since=`, `POST /collab/ops`,
`POST /collab/sync`, `/collab/resync`, `GET /collab/stream` (SSE), `POST|GET
/collab/presence`.

### 5.2.4 AI layer — `src/ai/`

| Block | Responsibility |
|-------|----------------|
| `runner.py` | Model-agnostic `AgentRunner` loop: `model(messages) -> tool calls`; step/op budgets; structured `AgentReport` |
| `tools.py` | Tool surface compiled to CRDT ops: `read_doc`, `apply_ops`, `get_versions`, `lock`, `presence`; versioned registry asserted against catalog |
| `schemas.py` | Versioned `TOOL_CATALOG` (model-agnostic schemas) |
| `review.py` | Agent op listing + accept/reject as pure op-stream ops (attach/detach agent site ops; invertibility from CRDT) |
| `mcp.py` | Hand-rolled MCP server over stdio: JSON-RPC 2.0, `initialize`, `tools/list`, `tools/call`, `ping`; gate `DOCSERVER_AGENTS=0` |

### 5.2.5 Support — `src/lib/`, `src/config.py`, `src/cli.py`, `src/main.py`

| Block | Responsibility |
|-------|----------------|
| `lib/store.py` | `DocumentStore`: SQLite ledger (`documents`, `versions` tables) + content dir; per-document version snapshots (`versions/<id>/<ts>.bin`); WOPI lock state; RLock-serialized shared connection |
| `lib/crypto.py` | JWT HS256 encode/decode (PyJWT primary) |
| `config.py` | `config.toml` + `DOCSERVER_*` env overrides; validation + precedence |
| `main.py` | `create_app()` — FastAPI app, lifespan wiring (store/session registry/config), CORS, WOPI error handler, `/health`, static mount |
| `cli.py` | CLI entry (`python -m src.main`), uvicorn bootstrap |

### 5.2.6 Web frontend — `web/`

| Asset | Role |
|-------|------|
| `index.html` / `home.html` | Editor SPA + landing/template page |
| `editor.js` | contenteditable editing, toolbar, autosave, read-only mode, collab client (op sync + SSE), AI review UI |
| `i18n.js` | UI string labels |
| `style.css` | Styling |
| `sw.js` + `manifest.json` | PWA: offline caching, installability |

### 5.2.7 Packaging & infra blocks

| Block | Role |
|-------|------|
| `Dockerfile`, `docker-compose.yml` | One image; compose = docserver + Traefik edge |
| `systemd/opencloud-docserver.{service,env}` | Hardened unit (`NoNewPrivileges`, `ProtectSystem=strict`, `ProtectHome=true`, `ReadWritePaths=data`) |
| `register_wopi_provider.py` | Registers the docserver as OpenCloud app provider via OCS API |
| `grafana/docserver-health.json`, `Makefile`, `scripts/` | Observability panel, dev shortcuts, deploy/validation scripts |
| `tests/` + `e2e/` | ~100 pytest modules + Playwright browser suites |

## 5.3 Composition hints (ownership rules)

- **Router → store/hub dependencies only via `Request.app.state`** (store,
  session registry, config) — no globals beyond the app.
- Every AI op passes through the **collab hub** (attributability);
  every HTML payload passes through the **sanitizer** before render.
- Configuration is read once at startup and frozen into `app.state.config`.
