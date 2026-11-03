# 5. Building Block View

## Level 1 — the five subsystems

```
World-Office
├── 1. Rust core (35 crates)     format parse/render/convert, protocols
├── 2. wo-docserver              THE deployment unit: editor UI host + WOPI +
│                                conversion API + collab hub (axum binary)
├── 3. Editor frontends          browser apps (word WYSIWYG live; React apps)
├── 4. Services                  thin axum/tokio microservices + 1 Node.js
│                                (mostly future decomposition surface)
└── 5. Integrations & desktop    OpenCloud, Nextcloud, Android, Tauri, WASM
```

## Level 2 — Rust core crate groups (`server/core/crates/`, 35 crates total:
27 core-workspace members + 8 sibling crates)

| Group | Crates | Role |
|-------|--------|------|
| Shared base | wo-common, wo-unicode, wo-office-utils | errors, test harness, encodings |
| Text formats | wo-txt, wo-rtf, wo-html, wo-fb2, wo-epub | `FormatRoundtrip` parsers |
| Office formats | wo-ooxml (DOCX/XLSX/PPTX), wo-odf, wo-msbinary, wo-hwp | the fidelity-critical family |
| Fixed-layout | wo-pdf, wo-pdf-render, wo-djvu, wo-xps, wo-ofd, wo-visio | read/render |
| Conversion | **wo-x2t** (+ wo-x2t-wasm) | 27 native converters, chaining |
| Rendering | wo-renderer (+ wasm), wo-fonts, wo-raster, wo-docx-renderer | canvas pipeline |
| Editing models | wo-sheet, wo-slide, wo-chart, wo-formula, wo-spell | per-domain models |
| Protocols/apps | **wo-docserver**, wo-wopi, wo-webdav, wo-route, wo-ooxml-ops | network + operations layer |

## Level 2 — wo-docserver modules (`core/crates/wo-docserver/src/`)

| Module | Responsibility |
|--------|----------------|
| `lib.rs` (113 KB) | app assembly, routes, conversion endpoints |
| `wopi.rs` | WOPI protocol implementation (largest surface) |
| `collab.rs` | collaboration hub state endpoints |
| `api_ext.rs`, `editor_docs.rs` | extended APIs, editor document plumbing |
| `store.rs` | local document store (`DOCSERVER_DATA_DIR`) |
| `static_files.rs` | editor UI hosting from `EDITOR_UI_DIR` |
| `docx_util.rs`, `ai.rs`, `config.rs` | docx helpers, AI surface, env config |

## Level 2 — frontend (`apps/web/apps/`, `packages/`)

- `documenteditor-wysiwyg/` — **the live word editor** (vendored vanilla JS:
  `index.html editor.js i18n.js style.css`, deployed as those 4 files), with
  `wo-bridge.js` as the WOPI adapter.
- `*-react/` editors (spreadsheet, presentation, pdf, visio, documenteditor),
  `editor-shell/`, `api/`, `common/`.
- `packages/` (@world-office): editor-common, editor-stores, design-system,
  collaboration-client/-react, sdk-bridge, plugin-sdk, i18n.

## Level 3 — the word editor's internal structure
(`documenteditor-wysiwyg/editor.js`, ~13k lines, guarded by **132 static
test files** run as plain node scripts)

| Block | Anchor | Role |
|-------|--------|------|
| `runCommand(cmd, val)` | top | the single command surface: every feature funnels through here (`data-cmd` delegation, keybindings, dialogs all call it) |
| `DIALOG_IDS` registry | ~line 2038 | the **62 dialogs**; ALL dialog markup lives in `index.html` (overlays), a generic Escape handler closes them; standalone dialog `*.html` files are dead code and get deleted (enforced by the audit-i18n dead-wiring gate) |
| Keydown C1 block | ~line 11065 | all Ctrl/Meta combos; `const k = ev.key.toLowerCase()` at block top; bindings follow the **OO census** where combos collide (AD-12) |
| Table panel | ~6900–7250 | applyTableBackground/Banded/BandedCols + `syncTablePanelControls` + right-panel listeners (banded rows AND columns = OO tableStyler parity) |
| Track changes | ~4037–4260 | client-side tracked ins/del markers (`beforeinput` interception), accept/reject per change, review list — no backend persistence yet |
| Page-number field | ~1245 | `insertPageNumber` plants `<span class="page-number">` (converter emits a PAGE field); Ctrl+Shift+P per OO census |
| Formula engine | ~2448 | `formulaCellValue`: table-cell `=SUM()` etc. with data-formula attributes + recalc |
| i18n | `i18n.js` | en + de maps; EVERY key must exist in BOTH (946 keys, audit-i18n gate) |

## Level 3 — the converter fidelity path (parity frontier)

```
editor HTML ──wo-html parse──▶ HTML model (Block/Inline, align, entities)
   │                                       │
   ▼ wo-x2t html_to_ooxml                  ▼ wo-x2t docx_body_to_html_blocks
OOXML model (DocxRun props, numPr,          │
hyperlink_rid, TextAlignment) ◀─wo-ooxml────┘
   │            parse (document.xml, numbering.xml,
   │            document.xml.rels incl. hyperlinks)
   ▼ wo-ooxml serialize (docx zip: document.xml, numbering.xml,
     styles.xml, rels with TargetMode="External", media)
```

Key invariant: **all fidelity fixes stay in the converters + format models**
(wo-html, wo-ooxml, wo-x2t) — never in the editor or the docserver.

## Services (`server/services/`)

Thin binaries wrapping library crates (axum + tokio): api-gateway,
identity-service (JWT/OAuth2), session-service, storage-service (SQLite
metadata + disk blobs — the only one with full CRUD + tests),
conversion-service (calls wo-x2t), coauthoring-service, mcp-server,
admin-panel (TS dashboard), **plus `services/server/` — a Node.js service**
(the only JS service; ESLint 9 + Prettier linting, own CI; ships the
DocBuilder document-conversion CLI). These are a **future decomposition
surface**, not the current production path — production traffic goes through
wo-docserver.
