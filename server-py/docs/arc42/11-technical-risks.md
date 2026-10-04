# 11 — Technical Risks

> arc42 §11 · Known risks with probability/impact, mitigation, and owners.
> **Revision:** 2026-09-05 · P = probability, I = impact (L/M/H).

## 11.1 Risk register

| # | Risk | P | I | Description | Mitigation / watch |
|---|------|---|---|-------------|--------------------|
| R-1 | **OpenCloud UUID service discovery breaks fixed-name wiring** | H | M | OCIS services register under UUID names in NATS while external config expects fixed names (e.g. `eu.opencloud.api.gateway-…@172.27.0.2`), blocking the collaboration/app-provider path (memory 2026-09-04). | Keep fixes in compose/registration layers (`register_wopi_provider.py`, OCIS env); verify against each OCIS version on staging before prod; never point public routing at staging. |
| R-2 | **Lossy format round-trips lose user content** | M | M | HTML subset ↔ OOXML/ODF mapping drops out-of-subset content (tabs in exotic structures, some numbering). | Documented per converter; differential/golden/property tests; `editor-format-parity` OpenSpec narrows gaps; export path lets users pull a faithful copy. |
| R-3 | **XSS via contenteditable / pasted HTML / agent HTML** | M | H | Browser editing + AI-generated HTML is an injection surface. | Sanitizer on load/save/render (allowlist); adversarial + agent sanitizer tests; CSP hardening as follow-up. |
| R-4 | **Shared-secret JWT compromise** | L | H | `DOCSERVER_JWT_SECRET` static; if leaked, forged WOPI tokens. | `openssl rand -base64 48`, mode-600 env file, rotation procedure in runbook, TTL 3600s, least-privilege process. |
| R-5 | **SQLite single-file concurrency / multi-instance conflicts** | M | M | One shared connection + RLock keeps a single process safe; running two docserver instances against one DB risks lock/version races (host mode). | Documented single-instance constraint; `test_store_multi_instance.py` covers the failure mode; production (client mode) keeps OCIS authoritative. |
| R-6 | **CRDT convergence edge cases (deliverability, tombstones)** | M | M | Lossy/reordered transport, late join, hybrid human+agent histories. | Property/model-based tests, pending-delete parking, deterministic ordering; interleave agents; SSE+poll fallback. |
| R-7 | **AI agent runaway / destructive edits** | M | M | Agent loops, over-budget ops, unwanted deletions. | Step/op budgets in runner; `DOCSERVER_AGENTS=0` kill switch; reviewable/revertible ops; awareness that rejection is itself editable history. |
| R-8 | **OCIS version drift in WOPI behavior** | M | M | Behaviors validated against OCIS 7.3.0 (discovery POST semantics, lock handling) may change. | Validation scripts (`validate_wopi_e2e.sh`), staging stack, discovery-XML constraints documented in code comments. |
| R-9 | **Browser diversity of `contenteditable`** | M | M | Formatting/paste behavior differs across browsers → inconsistent documents. | Keep the mapped subset small and tested; Playwright suites across engines; feature detection in `editor.js`. |
| R-10 | **Single process = single point of failure** | M | M | One process, one job → outage = no editing. | systemd `Restart=on-failure`, health probes, stateless-by-design (OCIS holds content), fast cold start. |
| R-11 | **Dependency of conversion on `python-docx`/`odfpy` parsing quirks** | M | M | Real-world files exploit edge cases (corrupt XML, huge files). | 128 MiB cap, typed conversion errors, conformance corpus (`test_conformance_corpus.py`), graceful 500-with-message. |
| R-12 | **Scope creep against Stoic discipline** | M | L | Pressure to add formats/services re-creates the cathedral. | ADR-001 + Stoic checks as merge gates; backlog priorities; "say no" recorded in ADRs. |
| R-13 | **PWA/offline staleness serving old editor code** | L | M | Service worker may cache stale assets after deploys. | Versioned manifests, cache-busting review; deterministic sw update flow. |
| R-14 | **MCP stdio hand-rolled protocol drift** | L | L | Vendor protocol evolution vs ~200-line implementation. | Pinned protocol version `2024-11-05`, catalog versioning, MCP fuzz tests. |

## 11.2 Accepted risks (deliberately not fixed)

- **R-2, R-9** are product decisions (ADR-002/003) — mitigated, not
  eliminated. Fidelity is bounded by a documented subset.
- **R-10** (single process) is chosen; HA/scale-out is out of scope (C-OPS).
- **R-5** single-instance SQLite is accepted for host mode given OCIS is
  authoritative in production.

## 11.3 Watch list / early warnings

- OCIS service-discovery naming → test each OCIS upgrade on staging before
  prod routing changes.
- Any new converter mapping that claims fidelity → requires differential
  coverage before merge (Q3).
- Agent model adapter changes on the caller side → re-run
  `test_agent_*` + review-control tests.
