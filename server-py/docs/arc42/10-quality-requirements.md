# 10 — Quality Requirements

> arc42 §10 · Quality tree, scenarios, and how quality is enforced.
> **Revision:** 2026-09-05

## 10.1 Quality tree

```
Quality model for World-Office (leaf = concrete, testable quality)

Simplicity & Maintainability (G1, G4)  [highest]
 ├─ Understandable code      → ≤400 LOC/file, ≤40 LOC/function, flat layout
 ├─ Minimal dependencies     → ~9 runtime deps, stdlib-first
 ├─ No build step / tooling  → uv only; zero npm; no WASM/nightly
 └─ Config, not recompiles   → env-over-file precedence

Reliability (G2)
 ├─ Format round-trip fidelity → differential/golden/property tests
 ├─ Crash/restart safety     → typed store errors, reentrant-lock writes
 ├─ Collaboration convergence→ CRDT property + interleave tests
 └─ Failure clarity          → WopiError → proper status codes

Security (G5)
 ├─ AuthN                 → JWT HS256 verified on protected surfaces
 ├─ XSS                   → sanitizer allowlist, adversarial tests
 ├─ Path safety           → invalid_doc_id() on all content routes
 └─ Least privilege       → hardened systemd unit

Performance & Efficiency (secondary)
 ├─ Cold start            → <500 ms (pure Python, no build)
 ├─ Request latency       → synchronous conversion on demand
 ├─ Payload ceiling       → 128 MiB cap
 └─ Real-time sync        → SSE stream; op log replay

Portability & Ops
 ├─ Deploy forms          → Docker image OR systemd unit
 ├─ Health signal         → /health
 └─ Observable            → structured stdout logging + Grafana panel

Agileability / "Yes/No discipline"
 └─ Stoic checks (below) gate new functionality
```

## 10.2 Quality requirements in table form

| # | Quality | Requirement | Verifiable by |
|---|---------|-------------|---------------|
| Q1 | Simplicity | File ≤ 400 LOC; function < 40 LOC; ≤3 nesting levels under `src/` | review gate, `ruff`, grep |
| Q2 | Maintainability | No new runtime dep without a 3-sentence Stoic justification | review gate (check #3) |
| Q3 | Reliability | DOCX/ODT round-trips survive the mapped subset | `test_converter*`, golden, hypothesis |
| Q4 | Reliability | Store survives crash/corrupt DB with typed errors | `test_store_crash.py`, `test_resilience.py` |
| Q5 | Collaboration | any interleaving of ops on any replicas converges | `test_collab_*`, `test_crdt_concurrent_edges.py` |
| Q6 | Security | unauth/locked requests rejected (401/409) | `test_wopi_auth.py`, `test_wopi_lock_lifecycle.py`, `test_client_mode.py` |
| Q7 | Security | no XSS through saved HTML or agent output | `test_sanitizer_*.py` |
| Q8 | Security | path-traversal doc ids rejected (400) | `test_api_fuzz.py`, protocol tests |
| Q9 | Performance | cold start < 500 ms locally | dev check |
| Q10 | Interop | WOPI discovery/GetFile/PutFile/locks valid against OCIS 7.3 | `e2e/` Playwright + `validate_wopi_e2e.sh` |
| Q11 | Testability | continuous suite incl. browser E2E runs in CI | GitHub Actions `docserver.yml` |
| Q12 | AI safety | agent budgets trip; review/reject is op-stream-exact | `test_ai_runner_bounds.py`, `test_ai_review*.py` |

## 10.3 Quality scenarios (examples)

**S1 — "3 AM patch test" (Simplicity).** A new maintainer opens
`src/editor/converter.py` at 3 AM, must locate the DOCX table mapping and
fix a border bug. *QoS:* reaches the mapping in < 5 minutes; change is a
one-liner; unit test covers it. **Test:** the converter differential suite.

**S2 — Round-trip survival (Reliability).** A teacher writes a document
with headings, nested lists, a multi-column table with covered cells, and
an image with alt text; saves via World-Office to DOCX, then reopens it.
*QoS:* all those structures survive; the updated text is present; no HTML
leakage. **Test:** golden + ODT media pipeline + `test_odt_tables_frames.py`.

**S3 — Convergent collaboration (Collaboration).** Two editors and one AI
agent edit the same paragraph concurrently over a flaky connection that
reorders ops. *QoS:* every replica converges to identical text; agent edits
are attributable and individually rejectable. **Test:**
`test_collab_modelbased.py`, `test_agent_collab_interleave.py`.

**S4 — Malicious document (Security).** An uploaded/spoofed document
contains script tags and `javascript:` links posing as a formatting doc.
*QoS:* served HTML contains none of them; save strips them again.
**Test:** `test_sanitizer_adversarial.py`, `test_sanitizer_agent_html.py`.

**S5 — Stale lock (Integrity).** Editor B attempts PutFile while editor A
holds the lock. *QoS:* 409 `lock mismatch` with the current lock echoed in
`X-WOPI-Lock`; B's session shows read-only/fresh state. **Test:**
`test_wopi_lock_lifecycle.py`, `test_session_lock_adoption.py`.

## 10.4 Stoic checks (governance, from `RETHINK_WORLD_OFFICE.md` §7)

Applied to every merge; they are the *quality budget* for new work:

1. Does this serve the OpenCloud integration? If no → reject.
2. Does this add a dependency? If yes, justify in ≤ 3 sentences.
3. Can this be done with stdlib? If yes → use stdlib.
4. Is the code flat enough (≤ 3 levels)? If no → restructure.
5. Is the function < 40 lines? If no → split it.
6. Is the file < 400 lines? If no → split it.
7. Does this make the system harder to deploy? If yes → redesign.

## 10.5 Verification evidence (current)

- Test suite across `tests/` (~100 modules: unit, protocol, property,
  golden, AI, security) plus `e2e/` Playwright suites; CI in
  `.github/workflows/docserver.yml`.
- Round-trip corpus + `scripts/mutation-test.py` for test strength;
  `.hypothesis/` for property-based fuzzing.
- Backlog tracks per-epic status; OpenSpec `editor-format-parity` maps the
  fidelity gap to user-visible acceptance criteria.
