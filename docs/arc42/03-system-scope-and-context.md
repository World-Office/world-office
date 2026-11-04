# 3. System Scope and Context

## Context diagram

```mermaid
flowchart LR
    User([User / Browser])

    subgraph WO["World-Office (this system)"]
        DS["wo-docserver<br/>(editor UI + WOPI + conversion + collab hub)"]
        ED["Editor apps (word/sheet/slide/pdf/diagram)<br/>static JS in browser"]
    end

    subgraph Cloud["OpenCloud stack (OCIS)"]
        OCIS["OCIS<br/>files, identity, WOPI host"]
        COLLAB["collaboration service<br/>(WOPI app, :9300)"]
    end

    EDGE["traefik edge<br/>editor.cloud.graphwiz.ai"]
    NC["Nextcloud instances<br/>(PHP integration)"]
    HARN["wo-test-harness<br/>(parity census, agents)"]

    User --> EDGE --> DS
    DS -->|WOPI: GetFile/PutFile/locks| COLLAB --> OCIS
    ED -->|fetch from| DS
    HARN -->|HTTP probes| DS
    NC -.->|embedded via iframe/WOPI| DS
```

## Scope

**In scope:**
- Editor frontends (word WYSIWYG is live; sheet/slide/pdf/diagram React apps).
- wo-docserver: serves editor UI, WOPI server endpoints, conversion API
  (`POST /api/conversion/convert`), collaboration hub state
  (`GET /api/documents/<id>/collab/state`), local document store.
- Rust core: 35 crates (16 format parsers with `FormatRoundtrip`, renderer,
  fonts, raster, wo-x2t converter orchestrator, spell, route).
- Services (thin axum/tokio): api-gateway, identity, session, storage,
  conversion, coauthoring, mcp-server, admin-panel.
- Integrations: OpenCloud, Nextcloud, Android, document-server examples.
- Desktop PoC (Tauri), WASM targets.

**Out of scope:** file storage, sharing, user management (OCIS owns these);
reverse TLS termination and DNS (host-level traefik/caddy).

## Interfaces (external)

| Interface | Partner | Protocol |
|-----------|---------|----------|
| WOPI server endpoints | OCIS collaboration service | HTTP + JWT access tokens |
| Editor UI HTTP | Browser | static files over HTTP(S) |
| Conversion API | internal tools, tests, harness | JSON `{source_format, target_format, data: base64}` |
| Discovery/health | orchestrators | `/health` (container), `/healthz` (HTTP) |
| Nextcloud app | Nextcloud servers | PHP app embedding the editor |
