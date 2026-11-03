# World-Office Architecture Documentation (arc42)

Documented with the [arc42](https://arc42.org) template. One file per section.

| # | Section | File |
|---|---------|------|
| 1 | Introduction and Goals | [01-introduction-and-goals.md](01-introduction-and-goals.md) |
| 2 | Architecture Constraints | [02-architecture-constraints.md](02-architecture-constraints.md) |
| 3 | System Scope and Context | [03-system-scope-and-context.md](03-system-scope-and-context.md) |
| 4 | Solution Strategy | [04-solution-strategy.md](04-solution-strategy.md) |
| 5 | Building Block View | [05-building-block-view.md](05-building-block-view.md) |
| 6 | Runtime View | [06-runtime-view.md](06-runtime-view.md) |
| 7 | Deployment View | [07-deployment-view.md](07-deployment-view.md) |
| 8 | Cross-cutting Concepts | [08-crosscutting-concepts.md](08-crosscutting-concepts.md) |
| 9 | Architecture Decisions | [09-architecture-decisions.md](09-architecture-decisions.md) |
| 10 | Quality Requirements | [10-quality-requirements.md](10-quality-requirements.md) |
| 11 | Risks and Technical Debts | [11-risks-and-technical-debts.md](11-risks-and-technical-debts.md) |
| 12 | Glossary | [12-glossary.md](12-glossary.md) |

**Status:** initial version 2026-10-27, written against server @ `bc57a9214`;
**comprehensively updated 2026-11-03** (ledger 527/18 with L0 = 0, roundtrip
31/31, editor internals §5.3, ADRs 10–13, perf baseline, audit-integrity
rules) against server @ `8b6a3ac0a`.
Replaces the 2026-09-05 Python-era set (FastAPI docserver, decommissioned
2026-10-14) — those files are preserved under
[history/](history/) for the record.
**Further reading:** `server/AGENTS.md` (developer workspace guide), `plan/` (runbooks, guides),
`wo-test-harness` repo (parity census + ledger tooling) — the harness has
its own arc42 set + a census→ledger deep dive:
`wo-test-harness/docs/arc42/`.
