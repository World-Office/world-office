# 08 — Cross-cutting Concepts

> arc42 §8 · Concepts that cut across several building blocks.
> **Revision:** 2026-09-05

## 8.1 Security

**Threat model:** an unauthenticated web attacker, a malicious/compromised
document, and a hostile AI-edit surface; OCIS is a trusted peer that issues
valid JWTs.

| Concept | Mechanism |
|---------|-----------|
| **Authentication** | WOPI `access_token` = JWT `HS256` signed with the shared `DOCSERVER_JWT_SECRET` (`src/lib/crypto.py` — PyJWT primary, python-jose fallback). TTL default 3600 s. `/wopi/*` and launch paths validate tokens (`src/wopi/auth.py`). |
| **File-id safety** | `invalid_doc_id()` rejects path-traversal ids on the WOPI *and* editor APIs before they reach `DocumentStore.content_path` |
| **XSS defense** | Every HTML payload (converted doc, agent output, saved body) passes `src/editor/sanitize.py` — a `HTMLParser` rebuild that drops `script/iframe/object/form/...` and dangerous attributes while keeping the formatting subset |
| **Least privilege (process)** | systemd: non-root `docserver` user, `NoNewPrivileges`, `ProtectSystem=strict`, `ProtectHome`, `PrivateTmp`, read-only outside `data/` |
| **CORS** | `DOCSERVER_CORS_ORIGINS` (default `*` for dev; locked down in prod); exposes only `X-WOPI-Lock`, `X-WOPI-ItemVersion` |
| **Secrets management** | Secret never committed; generated via `openssl rand -base64 48`; env-file mode 600 |
| **Agent gate** | `DOCSERVER_AGENTS=0` disables all AI tools (`agents_disabled` responses); agents claim a named site (`agent=<name>`) for attribution |
| **Lock integrity** | WOPI lock tokens checked on PutFile (`X-WOPI-Lock`); lock mismatch → 409 with current lock; client-mode lock carried on the session, not globals |

## 8.2 Persistence

- **SQLite ledger** (`DocumentStore`, `src/lib/store.py`): table
  `documents` (id, name, size, created_at, updated_at, lock_token,
  lock_user) + table `versions` (doc_id, ts, author, size; indexed
  `doc_id, ts DESC`). A dashboard of bookkeeping, not a warehouse.
- **Content is files**: one file per doc id in `data/documents/`, raw
  Office bytes; version snapshots as `data/documents/versions/<id>/<ts>.bin`.
- **Consistency**: one shared SQLite connection serialized through a
  reentrant lock (`threading.RLock`) so concurrent HTTP handlers never
  interleave writes; `put_content → put_version` stays atomic via the same
  lock. Corrupt/absent DB surfaces as typed `DocumentStoreError`, not a
  crash.
- In production (client mode) the *authoritative* copy is in OCIS; the
  local store is used for host mode, tests, and version snapshots.

## 8.3 Configuration & startup

- Single source: `config.toml`; every key overridable by `DOCSERVER_*`
  env var (env wins). Validated at load (`src/config.py`).
- Immutable after startup; components read it from `app.state.config`.
- Lifespan wiring in `src/main.py`: `ensure_dirs → DocumentStore →
  SessionRegistry → config`; FastAPI gives OpenAPI at `/docs` for free.

## 8.4 Concurrency & process model

- **One process, threaded** (uvicorn default). No async required for
  request/reply WOPI (Stoic constraint, C-TEC).
- Per-document concurrency handled by the **collab hub** (op ordering) and
  the **session registry** (per-launch sessions isolate editors).
- Shared store access serialized via the store's RLock; the hub itself is
  asyncio-native for SSE fan-out.
- No background daemons, no message bus; long operations (conversion,
  agent runs) are request-scoped.

## 8.5 Error handling

| Layer | Style |
|-------|-------|
| Protocol | `WopiError(status, message)` raised in `src/wopi/protocol.py`; one exception handler renders JSON + status (400/401/404/409/500) |
| Storage | `DocumentStoreError` typed failures |
| AI tools | results with `error`/`isError=true` per MCP conventions (transport errors are JSON-RPC errors) |
| Conversion | `500 conversion failed: ...` JSON — never a raw traceback to the browser |

## 8.6 Logging & observability

- stdlib `logging` → **structured, leveled** lines to stdout/stderr; no log
  files (ops reads them via the stack's log collector).
- `GET /health` returns process + document-count status for probes.
- Grafana panel `grafana/docserver-health.json` renders the health signal
  (deployed per ops runbook).

## 8.7 Frontend architecture (editor)

- **Vanilla JS SPA**, contenteditable surface; PWA (service worker `sw.js`
  + `manifest.json`) for offline/installability; `i18n.js` label table.
- Realtime: op sync via `collab/ops` poll + `collab/stream` SSE; presence;
  AI review pane driven by `ai/*` endpoints.
- **No build step**: `web/` served statically by FastAPI; templates via
  Jinja2 (`home.html`).

## 8.8 Testing concept

- **Unit + protocol**: `tests/test_wopi*.py`, `test_crypto.py`, `test_store*.py`.
- **Round-trip fidelity**: differential + golden + hypothesis
  (`test_converter*.py`, `test_snapshot_golden.py`, `.hypothesis/` corpus).
- **CRDT correctness**: model-based & concurrent-edge tests
  (`test_collab_modelbased.py`, `test_crdt_concurrent_edges.py`).
- **AI**: tool fuzz, provider-fail, budgets, review control
  (`test_ai_*.py`); `test_agent_collab_interleave.py` mixes human+agent ops.
- **Security**: `test_sanitizer_adversarial.py`, `test_sanitizer_agent_html.py`.
- **Browser E2E**: Playwright suites in `e2e/` (edit depth, toolbar sweep,
  multisession, share lifecycle) against the WOPI test host.
- **Resilience**: `test_store_crash.py`, `test_resilience.py`, mutation
  script `scripts/mutation-test.py`.

## 8.9 Development workflow & tooling

- `uv` (lockfile `uv.lock`), Ruff lint (`E,F,W,I,UP`, line-length 100),
  pytest + pytest markers (`integration`); `Makefile` shortcuts.
- OpenSpec change-driven: `openspec/changes/*` (proposal/design/specs/tasks)
  feeding TDD; backlog epics E1–E23 as the upstream product input.
