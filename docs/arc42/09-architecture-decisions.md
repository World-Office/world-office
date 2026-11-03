# 9. Architecture Decisions

Short-form ADR digest (full history lives in git + `plan/`).

## AD-1: Rust core + TypeScript frontend
**Status: accepted (2026).** 26-crate Rust core for formats/protocols; TS
monorepo for editors. Rationale: single-language fidelity core, WASM path to
the browser, no proprietary engine.

## AD-2: One canonical Rust docserver — Python docserver decommissioned
**Status: accepted 2026-10-14 (commit series ending `cf98bf78e` preserved in
history).** The Python rewrite at `server/opencloud-docserver/` was removed;
the Rust `wo-docserver` container is the only docserver. Rationale: two
docservers doubled fidelity work; Rust is the parity frontier.

## AD-3: Vendored WYSIWYG editor + WOPI bridge
**Status: accepted 2026-09-24 (`2a98734f9`).** The word editor is vanilla JS
(`documenteditor-wysiwyg/`, editor.js byte-identical to source) plus
`wo-bridge.js`; React build kept as backup. Deploys are 4-file copies.
Rationale: fastest path to a live WYSIWYG without a rewrite; editor updates
decouple from binary releases.

## AD-4: HTML as the editor pivot format
**Status: accepted.** Editors speak HTML; all document fidelity is
concentrated in html↔docx conversion (`wo-x2t` + `wo-html` + `wo-ooxml`).
Consequence: converter crates are the parity-critical surface and own ~all
recent fidelity work (entities, run styling, sup/sub, alignment, lists,
hyperlinks — LEDGER2, 2026-10-27).

## AD-5: OpenCloud/OCIS + WOPI as the integration spine
**Status: accepted.** Storage, identity, sharing delegated to OCIS; docserver
implements WOPI; JWT secret shared; `WOPI_TOKEN_MODE=passthrough` on staging.
Nextcloud integration exists in parallel (PHP).

## AD-6: Parity measured by census/ledger, not opinion
**Status: accepted (2026-10).** OnlyOffice feature census → ledger with
L0–L4 levels → campaigns with count-agnostic accept gates. "Fix the
census/map first, then implement."

## AD-7: Agent-executed implementation with strict gates
**Status: accepted.** agentflow/taskfleet campaigns in isolated worktrees;
policy "NO CHANGE IS NOT SUCCESS"; count-agnostic accept gates; every task
lands with runnable verification. Humans review merges; agents never judge
their own completion without command evidence.

## AD-8: Nightly toolchain pin
**Status: accepted, revisit-worthy.** `rust-toolchain.toml` pins nightly
(stable ICE on wo-pdf/wo-webdav is resolved on ≥1.92 but the pin remains).
Cost: CI fragility when nightly breaks.

## AD-9: Services as decomposition surface, not production path
**Status: accepted (interim).** `services/` are thin axum binaries (+ one
Node.js service, `services/server/` — DocBuilder CLI, own ESLint/Prettier
CI); only storage-service is fully CRUD-implemented. Production traffic runs
through wo-docserver. Do not split the docserver until a scaling or team
boundary demands it.

## AD-10: Shared-mount direct-copy editor deploys
**Status: accepted (2026-11).** Editor releases are `cp` of the 4 files into
`~/editor-ui-ebae8fc3/word/`, one mount serving BOTH prod (:8082) and staging
(:8086), md5-verified on both endpoints. Docker builds from the `server/`
root routinely exceed 300 s and time out in the dev loop — image rebuilds
are reserved for docserver binary changes. Trade-off accepted: no atomic
prod/staging split on the editor surface.

## AD-11: Audit claims derive from census JSONs, not memory
**Status: accepted (2026-11-03).** After two fabrication incidents (invented
Ctrl+Alt+E/M shortcut rows; a "table sorting" gap OnlyOffice does not
ship), the rule is: every audit row / "missing" claim must cite a census
JSON, and OO-bundle greps (docker exec) are the arbiter when census is
silent. Corrections land as retractions with evidence, not silent edits.

## AD-12: OO census wins keyboard-binding collisions
**Status: accepted (2026-11-03).** When a combo collides with the OO census,
WO binds to the OO combo (Ctrl+Shift+L → bulleted list, Ctrl+Shift+8 →
formatting marks, Ctrl+Shift+P → page number; plain Ctrl+L → left align).
Word-convention bindings (Ctrl+1/2/5, F7) are added only in OO-free slots.
Rationale: parity is measured against OO; tests pin commands, not bindings,
so realignments don't break the suite.

## AD-13: All dialogs live in index.html
**Status: accepted (2026-11-03).** Every dialog is markup inside
`index.html`, registered in the `DIALOG_IDS` array (62 entries), closed by
the generic Escape handler. Standalone `dialog-*.html` files are dead code
by construction (nothing fetches or iframes them) — the audit-i18n
dead-wiring gate is the enforcement; dead files get deleted on discovery.
