# 2. Architecture Constraints

## Technical constraints

| Constraint | Detail |
|-----------|--------|
| Rust toolchain | Edition 2024; `rust-toolchain.toml` pins **nightly** (stable hit ICE on wo-pdf/wo-webdav historically; resolved for rustc ≥ 1.92 but the pin remains). |
| WASM untestable via cargo | `wo-x2t-wasm`, `wo-renderer-wasm` require wasm-pack / browser runtime; CI runs a separate `wasm.yml` workflow. |
| Frontend toolchain | pnpm 10.4.1, Node ≥ 20, Turbo orchestration, Biome linter (not ESLint). |
| Windows dev | WSL workaround required for `dlltool` failures in windows-sys crates. |
| CI platform | GitHub Actions; doc-only changes (`.md`, `artwork/`) skip CI entirely. |
| Tauri excluded from build | `pnpm build` filters out `@world-office/tauri-poc` (AppImage bundling needs linuxdeploy). |

## Organizational constraints

- Single primary maintainer + agentic dev loop (agentflow campaigns, worktrees,
  strict task gates). Everything must be verifiable by commands (tests,
  gates, probes), because agents need machine-checkable acceptance criteria.
- Two public remotes (Codeberg origin + GitHub mirror); pushes go to both.

## Product / deployment constraints

- **WOPI contract:** the docserver speaks WOPI with OCIS; WOPI endpoints must
  never be exposed without access tokens; `JWT_SECRET` is shared between
  docserver and OCIS.
- **Single prod host** (`192.168.42.42`): OpenCloud stack + docserver + edge
  router (traefik for `editor.cloud.graphwiz.ai`). Vertical scaling only.
- **Staging parity:** staging runs the same compose shape on ports 8086.
- Editor UI is deployed as static files into `EDITOR_UI_DIR` (volume-mounted),
  independent of the docserver binary image. **Docker builds from the
  `server/` root time out in practice (>300 s)** — editor releases are
  direct file copies into the shared mount, verified by md5 on both
  endpoints; image rebuilds are reserved for docserver binary changes.
- License boundary: AGPL core vs. `services-enterprise/` (audit, SCIM,
  webhooks) — the split must stay clean.
