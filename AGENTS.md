# WORLD-OFFICE WORKSPACE

> ⚠️ **DIRECTION — REVERSED 2026-09-24.** The **Rust + TypeScript stack is CANONICAL again**: the Rust docserver (`server/services/`, container `docserver-1` on `:8082`) serves prod editor traffic (`editor.cloud.graphwiz.ai`). The Python rewrite at **`server/opencloud-docserver/`** is **DEPRECATED** (container stopped; code kept as reference — it holds line-granular pagination, header/footer page furniture, and the converter work up to commit `cf98bf78e`). See `plan/RETHINK_WORLD_OFFICE.md` (superseded) and `server/AGENTS.md`.

> ⚠️ **OLD DIRECTION (2026-08-19 → 2026-09-24, SUPERSEDED)**: minimal Stoic Unix Python rewrite at `server/opencloud-docserver/` — retained below for history.

**Generated:** 2026-07-25
**Structure:** Workspace container for World-Office project and artifacts
**License:** Mixed (AGPL-3.0 for server, MIT for artwork)

## OVERVIEW

Workspace root containing the World-Office document editing suite project, brand assets, and planning materials. All substantive code lives in `server/`.

## STRUCTURE

```
World-Office/
├── server/                    # Main project: Rust core + TypeScript monorepo (62k files)
├── artwork/                   # Brand assets, logos, banners (MIT-licensed)
├── plan/                      # Implementation plans
└── .sisyphus/                 # Sisyphus tool internal state
```

## QUICK COMMANDS

```sh
# Rust (in server/)
cargo build --workspace                    # Build all 26 core crates
cargo test --workspace --lib -- --test-threads=1  # Run unit tests (--lib flag required)
cargo clippy --workspace                   # Lint
cargo fmt --all                           # Format

# TypeScript frontend (in server/)
pnpm install                              # Install deps (uses pnpm@10.4.1, node >=20)
pnpm dev                                  # Dev server (turbo orchestrator)
pnpm build                                # Build all packages
pnpm test                                 # Run all tests
pnpm lint && pnpm typecheck               # Lint + typecheck

# E2E tests (in server/tests/)
npm test                                  # Starts Docker stack, runs Playwright + Jest
```

## KEY FACTS

- **Rust toolchain:** Nightly required. `rust-toolchain.toml` sets channel = "nightly" because stable (1.94.1) hits ICE on wo-pdf and wo-webdav.
- **TypeScript tooling:** Uses Biome (not ESLint), orchestrated via Turbo (pnpm run dev/test/lint/typecheck).
- **WASM crates:** Cannot test with `cargo test` — use `wasm-pack` or browser runtime. CI runs separate `wasm.yml` workflow.
- **CI platform:** GitHub Actions. Workflows in `.github/workflows/`. Previously documented as Forgejo — corrected.
- **CI caching:** Shared Cargo registry cache across all Rust jobs; Turbo cache persisted for TypeScript jobs; Docker BuildKit layer caching for container builds.
- **CI concurrency:** Auto-cancels stale runs on same PR/branch. Doc-only changes (`.md`, `artwork/`) skip CI entirely.
- **wo-pdf excluded:** CI excludes wo-pdf and wo-webdav from cargo fmt/clippy (ICE with stable Rust). They compile with nightly.
- **Tauri excluded:** `pnpm build` excludes `@world-office/tauri-poc` via `--filter='!@world-office/tauri-poc'` because AppImage bundling requires `linuxdeploy`.
- **Test order:** Turbo tasks have `dependsOn: ["^build"]` — building dependencies before running tests.
- **wo-docserver:** Serves editor UI and proxies WOPI requests to OCIS for the E2E test stack.

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Project development | `server/` | See `server/AGENTS.md` (comprehensive) |
| Brand assets | `artwork/` | MIT-licensed logos, banners |
| Implementation plans | `plan/` | Roadmaps and technical plans |
| Format parsers | `server/core/crates/` | 16 format crates, each with `FormatRoundtrip` trait |
| Web editors | `server/apps/web/` | Vanilla JS editors + React wrappers |
| Services | `server/services/` | 8 Rust microservices + 1 Node.js (services/server/) |
| Nextcloud integration | `server/integrations/nextcloud/` | PHP + Vue 3, separate CI |
| E2E tests | `server/tests/` | Playwright + Jest + Docker Compose |

## SERVICES/SERVER NOTE

`services/server/` is **Node.js** (not Rust). It has its own `AGENTS.md`, ESLint 9 + Prettier linting (unique in the repo), and separate CI workflows. Its DocBuilder CLI is the document conversion tool.

## PAGES NOTE

工作效率提升的部分修改详情见文档 — refer to `plan/` directory for efficiency improvement documentation.