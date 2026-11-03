# 6. Runtime View

## R1 — User opens a document in the editor

```
Browser                docserver (:80)            OCIS / collab (:9300)
   │ GET /word/index.html  │                            │
   │──────────────────────▶│ static_files               │
   │◀── editor JS + wo-bridge                            │
   │ WOPI HTML flow (token) │                            │
   │──────────────────────▶│ wopi.rs ── GetFile ───────▶│
   │◀── editor bootstraps with doc content ◀────────────│
```

Readiness signal in the browser: `#toolbar ..panel.active`.

## R2 — Editing and collaboration

- Edits stay in the browser (WYSIWYG engine); `wo-bridge.js` serializes to
  HTML and pushes via WOPI `PutFile` on save/autosave.
- Client-side features that act on the live DOM: track changes (tracked
  ins/del markers, accept/reject), banded rows/columns (`.table-banded*`
  CSS classes), formatting marks overlay, spellcheck toggle. Session-level
  visual state only — persistence goes through the html→docx converter.
- Collaborative state: `GET /api/documents/<id>/collab/state?session=…` on the
  docserver hub (`collab.rs`). The hub is in-memory; a container restart
  clears it (known operational behavior — "de-poison the hub" runbook step).

## R3 — Format conversion (API)

```
client → POST /api/conversion/convert
         { source_format, target_format, data: base64 }
       ← { status: "Success", data: base64 }   (wo-x2t pipeline)
```
Sync request/response; used by tests, harness roundtrip probes, and internal
tooling. Requires `JWT_SECRET` set on the server.

## R4 — Parity roundtrip probe (test time)

```
harness roundtrip.cjs:
  drive editor via Playwright (apply feature)
  → read editor HTML
  → convert html→docx→html via R3
  → assert marker attributes survived verbatim (31/31 gates, level 3)
```

## R5 — Deployment / health

- Container healthcheck: `curl -f http://localhost:80/health`.
- Liveness probe used by ops/scripts: `GET /healthz`.
- After every editor deploy: pageerror sweep (0 JS page errors expected),
  live behavioral probes on staging :8086 (real key presses, specific DOM
  selectors — synthetic KeyboardEvents are unreliable for some bindings),
  and md5 verification of the deployed `editor.js` against the repo on BOTH
  endpoints (:8082 prod, :8086 staging).

## R6 — Agent campaign execution (dev-time)

agentflow orchestrates worker agents in isolated git worktrees against a
task DAG (`config/<campaign>/tasks.json` + `workers.json` + `repos.json`;
state under `~/.local/share/agentflow/state-<campaign>/`). Campaign
mechanics that matter architecturally:

- **Task prose carries VERIFIED facts** (exact line anchors, schemas, CLI
  behavior) — agents must not re-derive unknowable facts from scratch.
- **Accept gates run inside the task worktree** — always repo-relative,
  never absolute-path `cd` (a gate that escapes the worktree fails
  deterministically and burns retry budget).
- **Gates must fail on a no-op change** — af 0.1.0 has no built-in policy
  for that; grep-style gates need to assert the absence/presence of the
  exact changed bytes.
- **Rejected attempts keep their branch** (`tf/<TASK>-rejected-<n>-<ts>`) —
  a fixed gate can recover the work instead of re-running the agent.
- **Model reliability decides the channel**: glm-4.7 workers landed the
  editor-parity campaigns; glm-5.2 workers exited 1 with zero commits and
  that campaign was completed manually with the same gates. af is a tool,
  not a dependency.
- Same-repo tasks that touch overlapping files are chained via `deps` so
  merges never race.
