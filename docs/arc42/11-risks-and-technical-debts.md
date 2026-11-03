# 11. Risks and Technical Debts

## Risks

| Risk | Impact | Mitigation |
|------|--------|-----------|
| **Single-host production** (all containers on 192.168.42.42) | host failure = full outage | backups + documented compose; no HA claim |
| **Stale staging images** (was 2026-10-18 old; caused false "gaps") | wrong converter conclusions | rule: verify against freshly built binary/image, never trust a container's age |
| **Nightly toolchain pin** (AD-8) | CI breakage on nightly regressions | upgrade path known (stable ≥1.92 fixed the ICE) |
| **In-memory collab hub** | restart clears collaboration state | runbook: restart de-poisons; persistence not yet built |
| **Bus factor 1 + agent fleet** | continuity risk | AGENTS.md knowledge bases, this arc42 set, memory systems, gates-as-spec |

## Technical debts (deliberate, with ceilings)

- **Converter ceilings (documented 2026-10-27):** list `start` value not
  emitted; `ilvl` nesting flattened both directions; hand-authored docx
  numbering numIds ≠1/2 degrade to body text in Word.
- **Backend-required features stuck client-side:** track changes has a full
  client-side implementation (tracked markers, accept/reject, review list)
  but no server persistence; comments likewise need a backend; spell-check
  is a client toggle over a service that does not exist yet. Macro engine
  remains roadmap. These are the biggest structural parity items left.
- **PERF-002 (requestAnimationFrame rendering pass)** — deferred: act only
  on a profiling delta against the 2026-11-03 baseline, not on intuition.
- **wo-docserver `lib.rs` is 113 KB** — module split overdue; the crate is
  accreting surfaces (AI, editor docs, collab) that want their own modules.
- **Services are thin shells** — only storage-service has real CRUD; the
  rest await the decomposition trigger (AD-9).
- **Carried follow-ups:** `w:sdt` docx emitter, `textDirection` in wo-ooxml,
  export-to-file, chart 3D depth.
- **Resolved:** insert-spreadsheet L0 cluster (L0 → 0, 2026-11-03); harness
  roundtrip workarounds (`&nbsp;`→space, non-ASCII→numeric refs) — removed
  2026-10-27 after the converter fixes landed; probes run verbatim so
  regressions fail loudly.

## Documentation-integrity risks

- **Stale claims in reports** — a PARITY-REPORT "after" column once claimed
  L2:87/L4:22 with no ledger run behind it (caught + corrected 2026-11-03).
  Rule: report numbers must come from the actual build output of the run
  they describe; verification debt is noted explicitly.
- **Fabricated audit rows** — see AD-11: Word-memory claims retracted with
  grep evidence; re-suspect any row lacking a census/bundle citation.

## Anti-patterns to avoid (from AGENTS.md)

- Never push to main without `cargo test` on changed crates.
- No new Rust deps without Windows-compile check.
- Never expose WOPI endpoints without auth tokens.
- WASM crates are not `cargo test`-able — use wasm CI path.
