# World-Office — arc42 Architecture Documentation

> **System:** World-Office (`opencloud-docserver`)
> **Version:** 0.1.0 · **Status:** active · **License:** AGPL-3.0 (server), MIT (artwork)
> **Format:** [arc42](https://arc42.org) template — one document per section, plus this index.

This directory documents the **canonical** World-Office product: the minimal
Stoic Unix rewrite — a single Python (FastAPI) WOPI document server at
`server/opencloud-docserver/`. The former Rust + TypeScript "cathedral"
(26 crates, 13 TS packages, ~62,000 files) is **deprecated** and kept only as
reference (see [`plan/RETHINK_WORLD_OFFICE.md`](../../plan/RETHINK_WORLD_OFFICE.md)
and [`server/AGENTS.md`](../../server/AGENTS.md)).

> **One process. One job: edit office documents through OpenCloud.**

## Document map

| # | Section | Module | TL;DR |
|---|---------|--------|-------|
| 01 | [Introduction & Goals](01-introduction-and-goals.md) | Requirements | Why World-Office exists: a simple, reliable, focused, self-hosted office editor; Stoic virtues as quality goals. |
| 02 | [Architecture Constraints](02-architecture-constraints.md) | Constraints | Python 3.12+, FastAPI, SQLite, uv, WOPI, OpenCloud-only, systemd/Docker. |
| 03 | [Context & Scope](03-context-and-scope.md) | Context | Business + technical context: browser ↔ OpenCloud ↔ docserver; what's in and out of scope. |
| 04 | [Solution Strategy](04-solution-strategy.md) | Strategy | Stoic rewrite, DOCX/ODT↔HTML round-trip, vanilla-JS editor, CRDT collab, dual WOPI role. |
| 05 | [Building Block View](05-building-block-view.md) | Building blocks | Whitebox decomposition of the docserver, the editor, and the AI layer. |
| 06 | [Runtime View](06-runtime-view.md) | Runtime | Scenarios: WOPI host mode, OCIS-launched client mode, collab, AI agents, versions. |
| 07 | [Deployment View](07-deployment-view.md) | Deployment | Docker compose, systemd hardening, production VPS layout, app-provider registration. |
| 08 | [Cross-cutting Concepts](08-cross-cutting-concepts.md) | Cross-cutting | Security, persistence, config, concurrency, observability, frontend architecture. |
| 09 | [Architectural Decisions](09-architectural-decisions.md) | Decisions | ADR-001…ADR-011 — the "why" behind the shape of the system. |
| 10 | [Quality Requirements](10-quality-requirements.md) | Quality | Quality tree + scenarios; Stoic checks as acceptance criteria. |
| 11 | [Technical Risks](11-technical-risks.md) | Risks | Known risks, mitigations, and watch items. |
| 12 | [Glossary](12-glossary.md) | Glossary | WOPI/CRDT/OCIS/… terms used across the documentation. |

## Reading guide

- **Coders / maintainers** — start with [01](01-introduction-and-goals.md),
  read [05](05-building-block-view.md) before touching code, and
  [09](09-architectural-decisions.md) before *changing* architecture.
- **Operations** — [07](07-deployment-view.md) plus the
  [operations runbook](../../plan/operations-runbook.md).
- **Product** — [01](01-introduction-and-goals.md),
  [03](03-context-and-scope.md), [10](10-quality-requirements.md); roadmap
  lives in the [epic backlog](../../server/opencloud-docserver/docs/backlog-epics-and-user-stories.md).

## Sources of truth

Primary code artifact: `server/opencloud-docserver/` (README, `config.toml`,
`pyproject.toml`, `src/`, `web/`, `tests/`, `docs/`, `openspec/`).
Supporting documents:

- `plan/RETHINK_WORLD_OFFICE.md` — Stoic rewrite rationale and direction
- `plan/operations-runbook.md` — production deployment & ops
- `DEPLOYMENT.md` — deployment guide (legacy Rust stack)
- `AGENTS.md` (root + `server/`) — workspace & conventions

## Conventions used in these documents

- Module paths are relative to `server/opencloud-docserver/`.
- "OCIS" and "OpenCloud" are used interchangeably (the OpenCloud server,
  formerly "oCIS").
- ASCII architecture diagrams (Mermaid optional) keep the docs diff-friendly.
