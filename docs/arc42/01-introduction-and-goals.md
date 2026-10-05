# 01 — Introduction & Goals

> arc42 §1 · Requirements Overview / Quality Goals / Stakeholders
> **Revision:** 2026-09-05 (matches `opencloud-docserver` v0.1.0)

## 1.1 Requirements Overview

World-Office is a **self-hosted, sovereign document editing suite** for
[OpenCloud](https://opencloud.eu) (OCIS) — an in-house alternative to
proprietary cloud office suites such as Microsoft 365 or Google Docs. It
lets a user browse their files in OpenCloud, click an office document, and
edit it directly in a browser editor served by World-Office, with the file
saved back into the user's OpenCloud storage.

The product is deliberately **minimal**: one process, one job — *edit office
documents through OpenCloud*. This is the outcome of a conscious rewrite
decision (2026-08-19, confirmed 2026-08-21): the earlier prototype ("the
cathedral") accumulated ~62,000 files across Rust, TypeScript, PHP and
Node.js, required a nightly compiler and a WASM pipeline, and took 45+
minutes to build. It was discarded in favor of the Stoic Python rewrite that
this documentation describes.

**Canonical** (in scope today):

- WOPI protocol server that speaks to OpenCloud as the document host.
- DOCX **and** ODT document editing via a browser editor.
- Real-time collaborative editing between multiple editors (CRDT-based).
- Optional **AI editing agents** that operate on documents like a human
  collaborator, with per-operation human review.
- Versioning with one-click restore of earlier snapshots.
- Export to HTML / DOCX / ODT / PDF.

**Out of scope** (deliberately — see [03 — Context & Scope](03-context-and-scope.md)):

- Spreadsheets, presentations, PDF editing, DTP/print fidelity.
- A collaboration server (deferred to OCIS), microservices, Kubernetes.
- Mobile/desktop native clients; the editor is a responsive web app only.

## 1.2 Goals

The overarching quality goals are the four Stoic virtues applied to
software, stated in `plan/RETHINK_WORLD_OFFICE.md` §3:

| Goal | Meaning | Measurable proxy |
|------|---------|------------------|
| **G1 · Simplicity** | Easy to understand, modify, deploy. "An editor you never have to patch at 3 AM." | ~3,000 LOC total; files ≤ 400 lines; ≤ 3 levels of nesting under `src/`; ~9 runtime dependencies |
| **G2 · Reliability** | Fewer lines = fewer bugs; lossless-enough round-trips. | Continuous test suite (unit, golden, mutation, property-based, browser E2E) green on every change |
| **G3 · Focus** | Does one thing (edit office docs through OpenCloud) and does it well. | Every feature must serve the OpenCloud editing loop (Stoic check #1) |
| **G4 · Maintainability** | Standard tooling, stdlib first, no nightly compilers or WASM chains. | Python 3.12 + `uv`; zero build step; pinned deps in `pyproject.toml` |
| **G5 · Security & least privilege** | JWT-authenticated WOPI, HTML sanitization, hardened process. | `NoNewPrivileges`, `ProtectSystem=strict`, sanitizer tests, auth tests |
| **G6 · Correct collaboration** | Concurrent editors converge; agent work is attributable and reversible. | CRDT convergence/property tests; AI review accept/reject semantics |

Quality requirements are detailed in [10 — Quality Requirements](10-quality-requirements.md).

## 1.3 Stakeholders (and their interests)

| Role | Stakeholder | Main interest |
|------|-------------|---------------|
| **End user** | Teachers, authors, knowledge workers | Edit DOCX/ODT quickly in the browser; not lose content; collaborate; trust their files stay on their own server |
| **Product owner** | World-Office maintainers (`weiss`) | The Stoic mission; feature roadmap (`docs/backlog-epics-and-user-stories.md`); sovereign/European positioning |
| **Integration partner** | OpenCloud (OCIS) project | A working WOPI app provider; correct WOPI client behavior; interoperability |
| **Development** | Contributors to `server/opencloud-docserver/` | Simple codebase, fast local loop, spec-driven changes (OpenSpec) |
| **Operations** | VPS administrator (host `178.254.2.90`) | Easy deployment (Docker/systemd), health signals, minimal moving parts |
| **Security reviewers** | External / project auditors | Least privilege, JWT handling, XSS resistance, no path traversal |

## 1.4 "As-is" → "To-be" summary

| Aspect | As-is (deprecated cathedral) | To-be (documented here) |
|--------|------------------------------|--------------------------|
| Languages | Rust, TypeScript, PHP, Node.js, EJS | Python 3.12 + vanilla JS |
| Footprint | ~62,000 files, ~200 crates, ~10,000 npm packages | ~40 files, ~9 pip packages |
| Build | 45+ min (Rust+TS+WASM) | None (interpreted) |
| Services | 9 microservices + 8 web apps | 1 process |
| Nightly compiler | Required (ICE on stable) | Never |
| Formats | 16 Rust parsers | `python-docx`, `odfpy` (DOCX + ODT) |

A comparison table with exact metrics lives in `RETHINK_WORLD_OFFICE.md` §8.

## 1.5 Short present state of development

The canonical product is live/under active development in
`server/opencloud-docserver/` with an extensive test suite (~100 test
modules incl. WOPI protocol, CRDT, AI, sanitizer, browser E2E), OpenSpec
changes (`editor-format-parity`, `cloud-editor-complete`,
`editor-ui-completeness`), and a staged deployment (Docker + systemd) —
see [07 — Deployment View](07-deployment-view.md). The product backlog of
epics (E1–E23) is maintained in
`docs/backlog-epics-and-user-stories.md`.
