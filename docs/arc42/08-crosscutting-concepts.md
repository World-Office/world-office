# 8. Cross-cutting Concepts

## Document models and the FormatRoundtrip trait

Every format crate implements parse → shared model → serialize. Fidelity work
is contract-first: extend the model (e.g. `Paragraph.align`,
`DocxBody.hyperlink_rels`), wire parse + serialize, then pin with roundtrip
gate tests. Conventions that have emerged:

- **Entity policy (wo-html):** XML's five named entities stay named; known
  HTML named entities become numeric refs; unknown names degrade to literal
  text. A parse error is never acceptable.
- **numPr convention (wo-ooxml):** numId 1 = bullet, numId 2 = decimal,
  style `ListParagraph`; `word/numbering.xml` is emitted only when some
  paragraph carries numPr (documents without lists keep their old byte shape).
- **Relationships:** hyperlink rels ride the same rels vector as images
  (`rIdLnk{n}` ids), with `TargetMode="External"`.

## Fidelity gates (testing concept)

Four layers, all machine-checkable:

1. **Unit gates** — `cargo test` per crate (wo-html 114, wo-ooxml 66,
   wo-x2t 440 + converter gap suites like `tests/converter_gaps.rs`),
   renderer-wasm 153 in the wasm path.
2. **Roundtrip probes** — `wo-test-harness` roundtrip.cjs drives the real
   editor via Playwright, converts through the real HTTP endpoint, asserts
   markers survive (31/31, no sanitization allowed).
3. **Static JS gates** — 132 editor test files: substring pins
   (`src.includes(...)`, no regex) on the landed editor bundle, because
   regex escapes in JS strings are a recurring trap. One jest suite
   (shift-enter behavior) runs via the root jest.config.cjs; the rest are
   plain node scripts.
4. **Live behavioral probes** — census acts probes + post-deploy pageerror
   sweeps on the real staging editor.

## Parity methodology

Feature census of OnlyOffice (ground-truth JSONs, read-only — ribbon,
dialogs, deep menus, context menu, **keyboard shortcut types**, behavioral
acts probes) → ledger (**527 features, 18 clusters, L0 unknown → L4
verified**; L0 = 0 since 2026-11-03) → gap campaigns (agentflow or manual,
same gates) → gates land with each fix → harness reruns re-score. Noise
filtering rules (regex, backstage exclusion lists, statusbar raw ids) live in
`wo-test-harness/census/ledger/build.py`; the L2 ("acts") level promotes
rows via live probe transitivity — `census-wo-acts.cjs` drives every visible
`button[data-cmd]` in the real editor and records DOM mutations, and
`COMBO_COMMAND` maps keyboard combos onto those same commands. Deep dive:
`wo-test-harness/docs/arc42/census-ledger-pipeline.md`.

**Anti-fabrication rule (2026-11-03):** audit documents and "missing feature"
claims must derive from the census JSONs — never from memory of Word or
another suite. Two incidents (invented Ctrl+Alt+E/M "track changes/comment"
shortcuts; a "table sorting" gap that grep proves OnlyOffice does not ship)
led to retracted audit rows. OO bundle greps (docker exec into the OO
container) are the arbiter when census JSONs are silent.

## Editor conventions

- Ribbon overflow scrolls by **translation**; rb-menu clicks in Playwright
  must be synthetic (`page.evaluate(el.click())`).
- **All dialogs live in `index.html`** behind the `DIALOG_IDS` registry (62
  entries); standalone dialog `*.html` files are dead code — the audit-i18n
  gate flags any `getElementById` target missing from index.html.
- i18n keys must exist in BOTH `en` and `de` (catalogs 981 keys each,
  audit-i18n: referenced 947, PASS).
- Landed JS must pass `node --check`.
- Static gates use plain substring pins (`src.includes(...)`, no regex).
- Tests pin COMMANDS, not keybindings — binding realignments (AD-12) must not
  break the suite.
- Live probes prefer **real key presses** (`page.keyboard.press`) plus
  specific DOM selectors over synthetic KeyboardEvent dispatch, which is
  unreliable for some bindings.
- TDZ rule: guard code goes after the helper's `const` declarations.

## Operations conventions

- Prod pageerror sweep after every deploy; `NO_PROXY=127.0.0.1,localhost` for
  local Node fetches; zombie probe ports hunted with `ss -tlnp`.
- Deploys verified with `curl` + `cmp` **from a known cwd** (relative paths
  have bitten before).
- `docker restart opencloud-compose-docserver-1` de-poisons the collab hub.
