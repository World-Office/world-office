# 12 — Glossary

> arc42 §12 · Terms and abbreviations used across this documentation.
> **Revision:** 2026-09-05

| Term | Meaning |
|------|---------|
| **World-Office** | The product: a self-hosted, sovereign office-editing service. Canonical form = `opencloud-docserver`. |
| **opencloud-docserver** | The single Python FastAPI process that implements World-Office. Located at `server/opencloud-docserver/`. |
| **OpenCloud / OCIS** | Open-source, self-hosted cloud storage & file platform; World-Office's integration target and WOPI *host* (file storage + users + auth). |
| **WOPI** | Web Application Open Platform Interface — Microsoft-proposed protocol for editing files stored in a host via a web client. Contract: `CheckFileInfo`, `GetFile`, `PutFile`, lock operations. |
| **WOPI host** | The system owning file bytes (here: OCIS in production, or the docserver in local host mode). |
| **WOPI client** | The web editor that fetches/saves file bytes through the host's WOPI endpoints (here: the docserver's editor in client mode). |
| **CheckFileInfo** | WOPI action returning file metadata (name, size, version, permissions). |
| **GetFile / PutFile** | WOPI actions to download / upload full file contents. |
| **Lock / Unlock / RefreshLock / GetLock** | WOPI operations maintaining the single-editor write lock via the `X-WOPI-Lock` token. |
| **WOPI discovery** | XML (`/hosting/discovery`) advertising which actions/extensions the app supports (`view`/`edit` for docx, odt). |
| **JWT / HS256** | JSON Web Token signed with HMAC-SHA256 using the shared secret — the WOPI access-token format. |
| **Access token** | The JWT OCIS hands the editor at launch, proving authorization for a specific file. |
| **DOCX / OOXML** | Microsoft Word document (Open XML) format — edited via `python-docx`. |
| **ODT / ODF** | OpenDocument Text format — edited via `odfpy`; self-contained images inside the package. |
| **Round-trip** | Document → HTML → document; the fidelity of conversion in both directions. |
| **CRDT** | Conflict-free Replicated Data Type — data structure whose concurrent updates converge without central coordination. |
| **RGA** | Replicated Growable Array — the specific sequence CRDT used (tombstone variant). |
| **Tombstone** | A deleted CRDT item kept in the item table (marked dead) so insert graphs stay intact and ops commute. |
| **Site** | A named writer identity in the CRDT (human editor, `__base__` seeder, or `agent=<name>`). |
| **Lamport clock** | Monotonic counter used to order an item's creation within a site. |
| **Op log / revision** | The hub's append-only operation history; every op gets a global monotonic revision used for replay (`since=<rev>`). |
| **SSE** | Server-Sent Events — the `/collab/stream` channel pushing live ops to editors. |
| **Presence** | Shared view of who is currently editing (cursor/participant info). |
| **MCP** | Model Context Protocol — here: a minimal stdio JSON-RPC 2.0 server exposing the agent tool catalog. |
| **AgentRunner** | Model-agnostic agent loop: `model(messages) → tool calls`, with step/op budgets and a structured report. |
| **Tool catalog** | Versioned schemas of agent tools (`read_doc`, `apply_ops`, `get_versions`, `lock`, `presence`). |
| **Sanitizer** | Allowlist HTML re-builder (`src/editor/sanitize.py`) stripping scripts/unsafe attributes. |
| **Session** | Per-editor-launch state (doc id, token, remote host, lock token); isolates concurrent editors of one file. |
| **SessionRegistry** | In-process registry of active editor sessions. |
| **DocumentStore** | SQLite ledger + content-directory file store with version snapshots. |
| **Version snapshot** | A saved copy of document bytes (`versions/<id>/<ts>.bin`) usable for restore. |
| **Stoic Linux / Stoic checks** | The product philosophy and its 7 merge-gate questions (`RETHINK_WORLD_OFFICE.md` §7). |
| **Cathedral** | Nickname for the deprecated Rust/TypeScript World-Office stack (~62k files), kept as reference only. |
| **OpenSpec** | Spec-driven development tooling used for changes (proposal/design/specs/tasks) — `openspec/changes/*`. |
| **E1…E23** | Epic identifiers in the product backlog. |
| **uv** | Python package/venv manager used to sync `uv.lock` (successor workflow to raw pip). |
| **Traefik** | Edge reverse proxy used in the Docker stack for routing/TLS. |
| **systemd** | Service manager used for the non-Docker deployment path (hardened unit). |
| **Playwright** | Browser automation used for E2E editor tests (`e2e/`). |
| **PWA** | Progressive Web App — the editor's service-worker + manifest offline/install capability. |
| **XSS** | Cross-site scripting — mitigated by the sanitizer and auth. |
| **128 MiB** | Hard ceiling on WOPI file content size (`MAX_FILE_SIZE`). |
| **`DOCSERVER_*`** | Environment-variable prefix overriding `config.toml` values (env wins). |
| **`/health`** | Liveness/diagnostic endpoint returning `{"status":"ok",...}`. |

## Abbreviations quick reference

`API` Application Programming Interface · `CORS` Cross-Origin Resource
Sharing · `CRDT` Conflict-free Replicated Data Type · `DOCX` Word Open XML ·
`E2E` end-to-end · `HMAC` keyed-hash message authentication code · `HTML`
HyperText Markup Language · `HTTP(S)` HyperText Transfer Protocol (Secure) ·
`JWT` JSON Web Token · `JSON-RPC` JSON remote procedure call · `LOC` lines of
code · `MCP` Model Context Protocol · `OCIS` OpenCloud server · `ODF/ODT`
OpenDocument Format / Text · `OOXML` Office Open XML · `OT` Operational
Transform · `PWA` Progressive Web App · `REST` Representational State
Transfer · `RGA` Replicated Growable Array · `SQL` Structured Query Language
· `SQLite` embedded SQL database · `SSE` Server-Sent Events · `WOPI` Web
Application Open Platform Interface · `XSS` cross-site scripting.
