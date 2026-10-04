# 02 — Architecture Constraints

> arc42 §2 · Anything that limits the solution space, with a compliance argument for each.
> **Revision:** 2026-09-05

Constraints are grouped by source. Each constraint states *why* it exists
and *how* the system complies.

## 2.1 Organizational constraints

| # | Constraint | Source / motivation | Compliance |
|---|-----------|---------------------|------------|
| C-ORG-1 | **Stoic Linux philosophy** — simplicity, clarity, reliability, focus | Maintainers' decision (2026-08); `plan/RETHINK_WORLD_OFFICE.md` | Yes — every PR is gated on the 7 "Stoic checks" (see [10](10-quality-requirements.md)) |
| C-ORG-2 | **OpenCloud-only integration** — no Nextcloud, no other WOPI/cloud hosts | Direction decision 2026-08-19 | Yes — integrations reduced 5→1 |
| C-ORG-3 | **Spec-driven development** — product backlog drives OpenSpec changes; TDD throughout | Working agreement; `docs/backlog-epics-and-user-stories.md`, `openspec/` | Yes — `openspec/changes/*` with specs/tasks; tests first |
| C-ORG-4 | **AGPL-3.0 server, MIT artwork** | Licensing decision | Yes — `LICENSE`, `LICENSE-COMMERCIAL` in `server/` |

## 2.2 Technical constraints

| # | Constraint | Motivation | Compliance |
|---|-----------|------------|------------|
| C-TEC-1 | **Python ≥ 3.12** | Stdlib maturity; `requires-python = ">=3.12"` | `pyproject.toml` pins it |
| C-TEC-2 | **FastAPI web framework** | Minimal ASGI framework, OpenAPI docs for free | `src/main.py` builds the app |
| C-TEC-3 | **WOPI protocol** as the integration contract with OpenCloud | Only sanctioned way to edit files through OCIS | Full WOPI host surface (`/wopi/files/...`) + client-mode forwarding |
| C-TEC-4 | **SQLite** (single file) for metadata/locks/versions | No daemon, no config, boring | `src/lib/store.py`; single table + `versions` table |
| C-TEC-5 | **uv** as package manager | 100× faster than pip, deterministic lockfile | `uv.lock`, `uv sync` in README |
| C-TEC-6 | **One Docker image** (the docserver), plus a Traefik edge proxy | Deployment minimalism ("not k8s") | `Dockerfile`, `docker-compose.yml` |
| C-TEC-7 | **systemd units** for non-Docker deployments | Process manager, not orchestration | `systemd/opencloud-docserver.{service,env}` |
| C-TEC-8 | **Config over code** — `config.toml` + `DOCSERVER_*` env overrides | Operation without code changes; env > file | `src/config.py` |
| C-TEC-9 | **stdlib first** — `sqlite3`, `urllib`, `logging`, `html.parser` before any dependency | Stoic check #3 | Visible throughout (CRDT reimplements no lib; sanitizer uses `HTMLParser`; MCP is hand-rolled JSON-RPC) |
| C-TEC-10 | **No native build step; no WASM pipeline; no nightly compiler** | Direct consequence of the rewrite | Runtime is pure Python + static web assets |
| C-TEC-11 | Max **128 MiB** file size for WOPI content | Safety ceiling (`MAX_FILE_SIZE`) | Enforced in `src/wopi/router.py` |
| C-TEC-12 | HTML sanitization on all editor/agent content | XSS resistance | `src/editor/sanitize.py`, applied on load/save/render |

## 2.3 Product constraints (scope boundaries)

| # | Constraint | Compliance |
|---|-----------|------------|
| C-PRO-1 | Editor formats: **DOCX, ODT** (+ txt/md passthrough) | `converter.py`, `odt_converter.py`, `CONTENT_TYPES` |
| C-PRO-2 | No spreadsheet/slides/PDF *editing* (PDF *export* only) | Export endpoint limited to html/docx/odt/pdf |
| C-PRO-3 | Editor is a **web page**, not a print-preview canvas — no pagination fidelity | Frontend design; see [04](04-solution-strategy.md) |
| C-PRO-4 | Collaboration served by this process (CRDT hub), **not** a separate coauthoring service | Local decision; OCIS collaboration deferred |

## 2.4 Conventions (as constraints)

| # | Constraint | Compliance |
|---|-----------|------------|
| C-CON-1 | Flat structure — **max 3 levels** under `src/` | `src/{wopi,editor,lib,ai,config,main,cli}.py` |
| C-CON-2 | Small files — **nothing over 400 lines**; functions < 40 lines | Enforced in review; largest file (`converter.py`) is a data-driven mapper |
| C-CON-3 | Structured JSON logging to stdout/stderr | `logging` module, no log files |
| C-CON-4 | Ruff linting (`E,F,W,I,UP`), line length 100 | `[tool.ruff]` in `pyproject.toml` |
| C-CON-5 | Pytest test suite; Playwright for browser E2E | `tests/` + `e2e/` |

## 2.5 Operational constraints

| # | Constraint | Compliance |
|---|-----------|------------|
| C-OPS-1 | Non-root service user, read-only filesystem except data dir | `systemd` hardening block (`NoNewPrivileges`, `ProtectSystem=strict`, `ProtectHome=true`, `ReadWritePaths=/opt/opencloud-docserver/data`) |
| C-OPS-2 | Production exposed behind HTTPS (Traefik TLS) at fixed hostnames | `cloud.graphwiz.ai`, `editor.cloud.graphwiz.ai` (see [07](07-deployment-view.md)) |
| C-OPS-3 | Health endpoint for load-balancer/probe | `GET /health` → `{"status":"ok",...}` |
| C-OPS-4 | JWT `HS256` secret shared with OpenCloud; never committed | `openssl rand -base64 48`; `DOCSERVER_JWT_SECRET` |

## 2.6 Open technical watch items (not-yet-constraints)

- OCIS service discovery uses **UUID-based service names** in NATS while
  some tooling expects fixed names — a known OpenCloud integration
  limitation tracked separately (see [11 — Risks](11-technical-risks.md)).
- Format round-trips are intentionally **lossy for out-of-subset content**
  (see [04](04-solution-strategy.md)) — this is a documented product
  decision, not an unconstrained gap.
