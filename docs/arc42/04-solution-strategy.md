# 4. Solution Strategy

The architecture follows one dominant idea and four supporting tactics.

## Dominant idea: document-as-JSON-IR around a Rust core

Every format crate parses its binary/XML format into shared JSON-ish document
models and serializes back (`FormatRoundtrip` trait). Conversion = parse to a
model, transform, serialize to the target (`wo-x2t`, 60 registered converter pairs with
chain support). The browser editors speak HTML — so **HTML is the editor-facing
pivot format**: docx↔html fidelity is the converter frontier (see §8).

## Supporting tactics

| Tactic | Where |
|--------|-------|
| **One canonical docserver binary** instead of a Python service — decommissioned 2026-10-14; editor traffic, WOPI, conversion, collab hub all in `wo-docserver` (axum + tokio). | `server/core/crates/wo-docserver` |
| **Static-first frontend** — vendored WYSIWYG editor JS + `wo-bridge.js` WOPI adapter; deployed as plain files into `EDITOR_UI_DIR`, no server-side rendering. Editor updates roll without rebuilding Rust. | `apps/web/apps/documenteditor-wysiwyg` |
| **Measured parity** — feature census of the reference suite (OnlyOffice) + ledger (527 features, 18 clusters, L0–L4 levels; L0 = 0 since 2026-11-03) + docx roundtrip probes (31/31); every fix lands with a gate test. | `wo-test-harness` |
| **Agent-friendly engineering** — machine-checkable acceptance everywhere: `cargo test` gates, static substring pins in JS test gates, pageerror sweeps after deploys, agentflow campaigns whose accept gates are designed to fail on a no-op change (af 0.1.0 has no built-in "NO CHANGE IS NOT SUCCESS" policy — the gates themselves must enforce it; gates must also be worktree-relative, never absolute-path `cd`). | `.af-state/`, `config/tasks.json` |

## Technology summary

- Backend: Rust (edition 2024), axum + tokio + tracing, zip/roxmltree for OOXML.
- Frontend: TypeScript monorepo (pnpm + Turbo + Biome), React wrappers,
  vanilla-JS WYSIWYG core.
- Deployment: Docker multi-stage images; Docker Compose stacks beside OCIS.
- Quality: cargo test/clippy/fmt, Biome, Playwright+Jest E2E, weekly
  cargo/pnpm audits.
