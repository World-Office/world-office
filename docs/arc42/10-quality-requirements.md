# 10. Quality Requirements

Quality tree (priority-ordered) with the concrete, currently-enforced checks:

```
Q1 Functional parity (OnlyOffice reference)
 ├─ Q1.1 Feature coverage: ledger 527 features / 18 clusters, L0 = 0
 │       (2026-11-03; the completeness contract)
 ├─ Q1.2 Dialog/menu parity: OO dialog census complete; chrome parity tasks
 │       done (CHROME1–5), zero pageerrors in prod sweeps
 ├─ Q1.3 Behavioral parity: shortcut census 57/147 matched + 17 native +
 │       70 nav + 3 documented divergences, 0 missing; 68/68 documented
 │       in shortcuts-audit.md; L2 acts probes cover 100+ mutated surfaces
 └─ Q1.4 Regression protection: 132 static JS gate files (substring, no
 │        regex) + tests pin commands, not bindings
Q2 Format fidelity (converter)
 ├─ Q2.1 docx roundtrip: 31/31 probes survive verbatim (no harness
 │       workarounds — retired 2026-10-27)
 ├─ Q2.2 Unit gates: wo-html 114 / wo-ooxml 66 / wo-x2t 440 green
 │       (renderer-wasm 153 in the wasm path)
 └─ Q2.3 Ceiling management: known, documented losses only (list start
 │       value, ilvl depth, foreign numbering numIds)
Q3 Security
 ├─ Q3.1 WOPI/JWT: no unauthenticated WOPI endpoints; JWT_SECRET shared
 ├─ Q3.2 Weekly audit job (cargo audit + pnpm audit --audit-level=high)
 └─ Q3.3 License hygiene: AGPL core vs. private enterprise repo split
Q4 Operability & performance
 ├─ Q4.1 Health endpoints (/health container check, /healthz probe)
 ├─ Q4.2 Deploy verification: md5 both endpoints + pageerror sweep +
 │       live behavioral probes
 ├─ Q4.3 First measured perf baseline (2026-11-03, benchmark.js on
 │       staging :8086): cold load ~103 ms, typing latency ~1 ms,
 │       select-all+bold on large doc ~21 ms, table insert+banded
 │       ~1.26 s — PERF-002 (rAF) deferred until a re-measure justifies it
 └─ Q4.4 Observability stack available (Grafana/Prometheus/Loki/Tempo
         under server/observability)
Q5 Maintainability
 ├─ Q5.1 Crate boundaries (35 core crates, FormatRoundtrip contracts)
 ├─ Q5.2 fmt + clippy + Biome clean; tests required for all changes
 └─ Q5.3 Agent-verifiable tasks (machine-checkable acceptance)
Q6 Portability
 ├─ Q6.1 Windows dev via WSL documented
 └─ Q6.2 WASM targets build in dedicated CI workflow
```

Not yet formally measured: latency/throughput budgets (**first measured
editor baseline landed 2026-11-03** in
`wo-test-harness/census/performance/baseline-wo.json` — cold load 176 ms,
typing 124 ms, select-all+bold 26 ms, table insert 54 ms; service-startup
figures remain `(estimate)` placeholders in `server/plan/performance.md`,
last updated 2026-07-18, awaiting production measurement). Availability
targets: single-host today, no SLOs yet (see §11).
