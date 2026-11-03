# 7. Deployment View

## Production (host 192.168.42.42, `/home/weiss/opencloud-compose`)

```mermaid
flowchart TB
    Internet -->|editor.cloud.graphwiz.ai| TR["traefik edge router<br/>(host-managed)"]
    TR --> DS1["opencloud-compose-docserver-1<br/>worldoffice/documentserver:latest<br/>192.168.42.42:8082 → 80"]
    subgraph OC["OpenCloud compose network"]
        OCIS["opencloud (OCIS)"]
        COLLAB["collaboration<br/>:9300"]
        DS1
    end
    VOLS["volumes (read-only mounts):
      /home/weiss/editor-ui-ebae8fc3/{word,sheet,slide,diagram,pdf}
      server/assets/dictionaries"]
    DS1 --- VOLS
```

- Image: `worldoffice/documentserver:latest` — multi-stage Docker build from
  `server/` root (`core/crates/wo-docserver/Dockerfile`; release build,
  frontend-dist + dictionaries baked in; cargo layers cached in CI/host).
- Editor UI is **volume-mounted over** the baked-in `editor-ui/word` — editor
  releases are file copies into `~/editor-ui-ebae8fc3/word/`
  (`index.html editor.js i18n.js style.css`). **One shared mount serves BOTH
  containers** (prod :8082 and staging :8086) — a single `cp` updates both,
  verified by md5 on each endpoint. Image rebuilds are only needed when the
  docserver *binary* changes (a full build from the `server/` root routinely
  exceeds 300 s and times out in the dev loop).
- Env: `JWT_SECRET` (shared with OCIS), `WOPI_HOST_URL`, `WOPI_TOKEN_MODE=passthrough`,
  `EDITOR_UI_DIR=/app/editor-ui`, `DOCSERVER_PORT=80`.

## Staging (host 192.168.42.42, `/home/weiss/opencloud-compose-staging`)

Same shape: `ocstaging-docserver-1` on `192.168.42.42:8086 → 80`, plus
`ocstaging-collaboration-1`, `ocstaging-opencloud-1`, `ocstaging-caddy-1`.
**Operational rule:** verify converter behavior against a freshly built local
binary or a freshly rebuilt image — staging images can be weeks stale and must
not be trusted as ground truth for converter gaps.

## CI/CD

- GitHub Actions workflows: ci, deploy, docker, release, security, wasm.
- Docker build matrix covers: identity-, storage-, conversion-,
  coauthoring-, session-service, api-gateway, wo-docserver.
- Rust CI: shared cargo registry cache, `--test-threads=1` for unit tests;
  wo-pdf/wo-webdav excluded from fmt/clippy; cold-worktree `cargo test` is
  the expensive gate (>620 s).
- Deploy sequence (editor): build gates → copy 4 files to the shared mount →
  md5-verify on :8082 AND :8086 → pageerror sweep + live behavioral probes
  on staging → prod container restart only if the docserver binary changed
  (rebuild image first).
