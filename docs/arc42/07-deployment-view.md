# 07 — Deployment View

> arc42 §7 · How the system is packaged, deployed, and run in its environments.
> **Revision:** 2026-09-05

## 7.1 Topology

```
                    ┌─────────────── VPS 178.254.2.90 ────────────────┐
                    │                                                │
                    │   Traefik (host reverse proxy, TLS)            │
                    │    ├─ cloud.graphwiz.ai      → OpenCloud at :9200
                    │    └─ editor.cloud.graphwiz.ai → docserver at :8082
                    │                                                │
                    │   Docker project "opencloud-compose"           │
                    │    ┌─────────────┐   ┌──────────────────┐      │
                    │    │  OpenCloud   │   │  docserver       │      │
                    │    │  (OCIS)  :9200│  │  FastAPI  :8082  │      │
                    │    │  WOPI host    │   │  WOPI client     │      │
                    │    └──────┬───────┘   └────────┬─────────┘      │
                    │           └──── WOPI ──────────┘                │
                    │   staging (separate) "ocstaging" :9201          │
                    │   data: docs in OCIS store; docserver: SQLite   │
                    │   + content dir on volume                       │
                    └────────────────────────────────────────────────┘
```

Production URLs (per `plan/operations-runbook.md` v2.0):

| Service | URL | Port | Health |
|---------|-----|------|--------|
| OpenCloud (OCIS) | `https://cloud.graphwiz.ai` | 9200 | `GET /status` |
| Docserver / editor | `https://editor.cloud.graphwiz.ai` | 8082 | `GET /health` |

There are **two parallel stacks** on the VPS: the public
`opencloud-compose` project (OCIS :9200, python-docserver :8082 behind the
Traefik fronting the public hostnames) and a separate staging compose
(`ocstaging`, caddy, :9201) for integration testing. Mind the routing in
memory: *public = `opencloud-compose`*, *ocstaging = :9201 only*; changes
must never point the public hostname at the staging stack.

## 7.2 Packaging

- **Docker image** `opencloud-docserver:latest` built from `Dockerfile`
  (Python 3.12, `uv` sync of `uv.lock`).
- **Compose** (`docker-compose.yml`): services `docserver` + `traefik`
  (edge router with `Host(`docs.local`)` rule for dev). `DOCSERVER_JWT_SECRET`
  from the environment; `docserver-data` volume binds `data/`.
- **systemd** fallback (`systemd/opencloud-docserver.service`): hardened
  unit — `User=docserver`, `NoNewPrivileges`, `PrivateTmp`,
  `ProtectSystem=strict`, `ProtectHome`, `ReadWritePaths=.../data`,
  `LimitNOFILE=4096`, `Restart=on-failure`; env from
  `/etc/opencloud-docserver/env` (mode 600). `scripts/deploy-systemd.sh`
  drives the install.

## 7.3 Configuration at runtime

- `config.toml` sections: `[server]` (port 8000, host 0.0.0.0), `[security]`
  (jwt_secret, jwt_ttl=3600), `[storage]` (database, content_dir), `[app]`
  (public_url, cors_origins).
- Precedence: **environment variables (`DOCSERVER_*`) → config.toml → defaults**.
- Required in production: a real secret (`openssl rand -base64 48`) shared
  with OCIS, a reachable `DOCSERVER_PUBLIC_URL`, and a locked-down
  `DOCSERVER_CORS_ORIGINS`.

## 7.4 OpenCloud integration steps (production)

1. Deploy the docserver (Docker or systemd) behind TLS.
2. Make the discovery XML reachable: `GET {public_url}/hosting/discovery`
   advertises `view`/`edit` actions for **docx** and **odt**.
3. Register the docserver as the app provider for office extensions — either
   via OpenCloud "Open App Registry" env vars (see compose comments) or the
   OCS API helper `register_wopi_provider.py` (OCIS admin creds; previously
   needed because the OpenCloud collaboration service was unreliable).
4. Share the JWT secret; verify end-to-end with
   `scripts/validate_wopi_e2e.sh` (~tests `test_wopi_*`, `test_client_mode.py`).

## 7.5 Environments

| Environment | Purpose | Form |
|-------------|---------|------|
| **Local dev** | Fast ramp-up | `uv run uvicorn src.main:app --reload`; local WOPI host mode; SQLite in `data/`; WOPI discovery + `/docs` (OpenAPI) |
| **CI** | Correctness gate | GitHub Actions `docserver.yml`: pytest (unit, golden, property, AI, WOPI), ruff, Playwright browser E2E against a mock/local WOPI host |
| **Staging** (`ocstaging`, :9201) | Integration w/ OCIS | Separate compose on VPS; used for OCIS-versioned WOPI validation |
| **Production** (`opencloud-compose`) | Live | See 7.1; rolling update = rebuild image + `docker compose up -d --build`; data survives on volume |

## 7.6 Data & backup

- Canonical user documents: **inside OCIS** (its store). The docserver's
  SQLite + content dir are caches/ledger for host mode and version snapshots.
- Version snapshots under `data/documents/versions/<doc_id>/<ts>.bin` —
  backup the volume; restoring a snapshot is a product feature
  (`POST .../versions/{ts}/restore`).

## 7.7 Observability

- `GET /health` for probes; structured `logging` to stdout/stderr (no log
  files, per Stoic convention).
- Grafana panel `grafana/docserver-health.json` (`/health` metrics);
  Prometheus scrapes the health endpoint per ops runbook.
- Failure modes are self-describing: `WopiError` → JSON + status; store
  corruption is reported as typed `DocumentStoreError` and storage is not a
  valid DB ⇒ the process stays up long enough for operators to react.
