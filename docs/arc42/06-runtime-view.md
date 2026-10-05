# 06 — Runtime View

> arc42 §6 · Important runtime scenarios as sequence flows.
> **Revision:** 2026-09-05 · Endpoints refer to `server/opencloud-docserver/src/editor/router.py` & `src/wopi/router.py` unless noted.

## 6.1 R-1 — WOPI host mode: open, edit, save (local/dev)

The user opens the editor directly (no OCIS); the docserver is its own
WOPI host over the local store.

```
Browser                 docserver                         store
   │  GET /editor/{id}      │                                 │
   ├───────────────────────►│  _session_for() -> session      │
   │                        │  CheckFileInfo (internal)       │
   │                        ├────────────────────────────────►│ get(id)
   │                        │◄────────────────────────────────┤
   │                        │  GET /api/documents/{id}/html   │
   │                        ├────────────────────────────────►│ content bytes
   │                        │  docx_to_html()/odt_to_html()   │
   │                        │  sanitize_html()                │
   │  HTML page + HTML doc  │◄────────────────────────────────┤
   │◄───────────────────────┤                                 │
   │  (user types...)       │                                 │
   │  POST /api/documents/{id}/save {html}                    │
   ├───────────────────────►│  sanitize → html_to_docx()       │
   │                        │  html_to_odt() (by extension)    │
   │                        ├────────────────────────────────►│ put_content + version snapshot
   │  200 {version}         │◄────────────────────────────────┤
   │◄───────────────────────┤                                 │
```

Notes: WOPI host endpoints for this mode: `CheckFileInfo` (`GET
/wopi/files/{id}`), `GetFile` (`GET .../contents`), `PutFile` (`POST
.../contents`), lock family (`/lock`, `/unlock`, `/refreshlock`,
`/getlock`). Every save writes a version snapshot and bumps
`X-WOPI-ItemVersion`.

## 6.2 R-2 — Production: OCIS-launched (WOPI client mode)

The canonical flow. OCIS is the WOPI host and system of record.

```
Browser                OpenCloud (OCIS)            docserver
   │ "Edit" on .docx         │                          │
   ├────────────────────────►│ discovery: Which app?    │
   │◄────────────────────────┤ GET /hosting/discovery   │
   │                         │ → app urlsrc = our /editor
   │ POST /editor?WOPISrc=…  │  (urlencoded form:       │
   │  (access_token, file_id)│   access_token,file_id)  │
   ├─────────────────────────┼─────────────────────────►│
   │                         │ _parse_launch(): token,  │
   │                         │ wopi_src, doc_id         │
   │                         │ session = EditorSession(remote_host=ocis,
   │                         │   access_token, lock_token="")   │
   │                         │                          │
   │                         │  (client) GetFile         │
   │                         │◄─────────────────────────│ RemoteWopiClient
   │                         ├─────────────────────────►│ GET {ocis}/wopi/files/{id}/contents
   │                         │  raw DOCX bytes           │
   │                         │◄─────────────────────────┤
   │  HTML for editing       │                          │
   │◄────────────────────────┼─────────────────────────►│ convert → sanitize → render
   │                         │                          │
   │  user edits + autosave   │                          │
   │  POST /api/documents/{id}/save                      │
   ├─────────────────────────┼─────────────────────────►│ (client) PutFile  +
   │                         │                          │  Lock/RefreshLock
   │                         │  POST {ocis}/wopi/files/{id}/contents   │
   │                         ├─────────────────────────►│  (RemoteWopiClient, lock_token)
   │                         │  bytes stored in OCIS    │
   │ 200                     │◄─────────────────────────┤
```

Key invariants (validated against real OpenCloud 7.3.0):
- The discovery `urlsrc` must **not** contain `access_token=` — OCIS
  appends `WOPISrc` itself and POSTs a urlencoded form with the real token.
- The WOPI lock lives on the **session** (taken at launch); without it OCIS
  refuses PutFile with 409 "unlocked file".
- Each concurrent editor gets its **own session** (`?session=`) so sessions
  never borrow each other's lock.

## 6.3 R-3 — Real-time collaboration (CRDT)

Two or more browsers + optional AI agent all attach to the same hub.

```
Editor A                docserver CollabHub             Editor B      Agent
   │  POST /collab/ops {insert}  │                           │        │
   ├───────────────────────────►│ apply: assign revision n   │        │
   │                            │ append to op log           │        │
   │                            ├── SSE /collab/stream ──────►│        │
   │                            │◄───────────────────────────│        │
   │                            ├─ POST /collab/ops {delete} ─┤        │
   │                            │  (late join) GET /collab/ops?since=17 │
   │                            │◄───────────────────────────┤        │
   │                            ├─────────────────────────►│ (replays 1..17)
   │                            │  presence POST/GET        │        │
   │                            │◄──────────────────────────┤        │
   │  POST /collab/ops (agent site "agent=<name>") ─────────►│        │
   │  GET /ai/review → element list (rev, agent, summary)   │        │
   │  POST /ai/review/reject {revs} → inverse ops by reviewer│        │
```

Convergence guarantees: ops are idempotent and commute; deletes that arrive
before their targets are parked and flushed on integration; the hub seeds a
fresh doc from stored content as site `__base__`, so late joiners replay
the whole op log and reach the same text.

## 6.4 R-4 — AI agent session (model-agnostic)

```
Agent harness          docserver (AI layer)                     model callable*
   │  spawn MCP (stdio, JSON-RPC 2.0)                          │
   ├── initialize ──►│                                         │
   ├── tools/list ──►│ → versioned TOOL_CATALOG                │
   ├── tools/call {name: read_doc, doc} ──►│──────────────────►│ transcript → tool calls
   │◄──────────────── result                                    │
   ├── tools/call {apply_ops, ...} ──►│ hub.apply (agent site)  │
   │◄──────────────── result {rev}                              │
   │  ... loops until done / max_steps / max_ops tripped        │
   │◄── AgentReport (steps, ops_applied, stopped_reason)        │
```

*The model adapter lives on the caller's side: `model(messages) -> list of
tool calls`; the server never talks to a vendor. Gate: `DOCSERVER_AGENTS=0`
disables every tool with `agents_disabled` errors. Rejection of agent work
is just more CRDT ops authored by the `reviewer` site.

## 6.5 R-5 — Version snapshot & restore

```
Editor                docserver                  store
   │  POST /save                 │                   │
   ├────────────────────────────►│  before overwrite  │
   │                             ├──────────────────►│ put_version: write
   │                             │                   │  versions/<id>/<ts>.bin
   │                             │                   │  INSERT versions row
   │  GET /api/documents/{id}/versions               │
   ├────────────────────────────►│                   │ SELECT ... ORDER BY ts DESC
   │  [ {ts, author, size} ]     │◄──────────────────┤
   │◄────────────────────────────┤
   │  POST .../versions/{ts}/restore                 │
   ├────────────────────────────►│ restore bytes ───►│ content_path overwritten
   │  200                        │                   │ + new version snapshot
```

## 6.6 R-6 — Export

```
Browser → POST /api/documents/{id}/export?format={html|docx|odt|pdf}
   load bytes → convert to HTML → sanitize
   html: UTF-8 HTML      docx: html_to_docx()      odt: html_to_odt()
   pdf : weasyprint when available, else minimal valid PDF
   → streamed attachment (Content-Disposition; X-Export-Engine for pdf)
```

## 6.7 Health & lifecycle

`GET /health` → `{"status":"ok","documents":N,"db":...}`. The FastAPI app
creates store/session registry/config inside the lifespan, so every request
resolves collaborators via `Request.app.state` — no module-level mutable
singletons besides the app itself.

Failure classification: `WopiError(status, message)` is raised in protocol
code and rendered by a single exception handler → JSON body + correct HTTP
status (400 invalid id, 401 auth, 404 missing, 409 lock mismatch).
