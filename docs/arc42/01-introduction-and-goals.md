# 1. Introduction and Goals

## What is World-Office?

World-Office is an independent, open-source document editing suite built in
Rust + TypeScript. It provides browser-based editors for word processing,
spreadsheets, presentations, PDF, and diagrams, backed by a Rust "core" of
format parsers, converters, renderers, and protocol implementations (WOPI,
WebDAV). It integrates with OpenCloud / ownCloud Infinite Scale (OCIS) as the
file-storage and identity layer.

License: AGPL-3.0-or-later (server), MIT (artwork), enterprise extensions in a
separate private repository.

## Goals

| # | Goal | Motivation |
|---|------|-----------|
| G1 | **Functional parity with OnlyOffice (OO)** | OO is the reference suite; parity is measured, not guessed — a feature census + ledger (`wo-test-harness`) tracks **527 features in 18 clusters** across levels L0–L4 (2026-11-03: **L0 = 0** — every census feature has a scored WO counterpart; behavioral shortcut census: 57 matched, 0 missing). |
| G2 | **Format fidelity** | Documents must survive edit→save→reopen without loss. Enforced by docx roundtrip probes (31/31 gates) and unit gates per converter (wo-html 114 / wo-ooxml 66 / wo-x2t 440). |
| G3 | **Independence** | No dependence on a proprietary editor engine; the WYSIWYG word editor is vendored under `apps/web/apps/documenteditor-wysiwyg/` and served from our own stack. |
| G4 | **Standards-based integration** | WOPI protocol against OCIS/OpenCloud; WebDAV; OOXML/ODF/legacy formats via the `FormatRoundtrip` trait family. |
| G5 | **Operable by one person** | Single-host Docker Compose deployments (prod + staging), weekly security audits, pageerror sweeps after every deploy, md5-verified deploys on both endpoints. |

## Non-goals (currently)

- Multi-tenant SaaS operation (single-tenant deployments only).
- A mobile web app (an Android app exists as an integration; desktop via Tauri PoC).
- Re-implementing storage/identity — delegated to OCIS.

## Stakeholders

- **Users:** people editing documents through OpenCloud and the public editor
  (`editor.cloud.graphwiz.ai`).
- **Maintainer/operator:** single primary developer + AI agent fleet
  (agentflow/taskfleet campaigns execute much of the implementation work;
  the parity program has run 20+ campaigns — af where the worker model holds
  up, manual execution when it doesn't, same gates either way).
- **Integrators:** Nextcloud (PHP app), OpenCloud, document-server examples
  (C#, Go, Java, Node.js, PHP, Python, Ruby).
