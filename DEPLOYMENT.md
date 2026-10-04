# World Office Deployment Guide

## Quick Start

```bash
# 1. Build Rust services
cargo build --release -p wo-docserver

# 2. Build frontend
cd server && pnpm install && pnpm build

# 3. Package frontend assets
# Copy dist from each editor app into frontend-dist/<type>/
# See server/docker.yml for the exact restructuring

# 4. Run the docserver
JWT_SECRET="<your-secret>" cargo run --release -p wo-docserver
```

## Production Security

### JWT Secret
- **Required**: Set `JWT_SECRET` env var to a 256+ bit random value
- Default `"test-secret"` is used in dev only
- Same secret must be shared between: wo-docserver, api-gateway, identity-service

### CSP Headers
Currently no Content-Security-Policy headers are set. For production:
```rust
// In the HTTP server response layer, add:
// Content-Security-Policy: default-src 'self'; frame-src 'self' https://*.documentcloud.org; object-src 'none'
```
The API gateway uses `tower-http` with CORS enabled. Add the `csp` feature:
```toml
tower-http = { features = ["cors", "csp", "trace"] }
```

### CORS
API gateway configures CORS for the docserver origin. Ensure the `ALLOWED_ORIGIN` env var matches your deployment URL.

### API Gateway Routes
| Route         | Auth Required | Upstream       |
|---------------|---------------|----------------|
| `/health`     | No            | —              |
| `/auth/*`     | No            | identity-service |
| `/files`      | Yes           | storage-service |
| `/convert`    | Yes           | conversion-service |
| `/collab`     | Yes           | coauthoring-service |
| `/mcp`        | Yes           | mcp-server     |

### Public Paths (no auth)
- `GET /health`
- `POST /auth/login`
- `POST /auth/register`

## Docker Deployment

```bash
# Build all Docker images
cd server && docker compose -f docker-compose.yml build

# Start with observability stack
docker compose --profile observability up -d

# Access Grafana at http://localhost:3002
# Access Prometheus at http://localhost:9090
# Access Loki at http://localhost:3100
```

## Monitoring

- **Grafana**: Pre-configured dashboards for production overview, services, conversion, and logs
- **Prometheus**: Scrapes all 8 services at `/metrics`
- **Loki**: Centralized log aggregation
- **Tempo**: Distributed tracing

## Environment Variables

| Variable       | Default        | Required | Description |
|---------------|----------------|----------|-------------|
| `JWT_SECRET`  | `test-secret`  | ✅ Prod  | Shared JWT signing secret |
| `LISTEN_ADDR` | `0.0.0.0:8082` | —        | Docserver listen address |
| `RUST_LOG`    | `info`         | —        | Log level for Rust services |
| `ALLOWED_ORIGIN` | `*`         | ✅ Prod  | CORS allowed origin |

## Service Health Endpoints

All 8 services expose `GET /health` returning:
```json
{ "status": "ok", "service": "<name>", "version": "0.1.0" }
```

## WOPI Integration

World Office uses WOPI protocol for cloud storage access:

1. **OCIS** serves as WOPI host (file provider)
2. **wo-docserver** proxies WOPI requests to OCIS
3. **React editors** load in embedded mode via iframe
4. **Auto-save** uses debounced WOPI PutFile (3s)

See `integrations/nextcloud/` for Nextcloud WOPI integration.

## MCP Server

The MCP server provides AI tool integration:
- 15 tools: list_documents, read_document, create_document, add_comment, etc.
- Runs on stdio transport
- Connect via Claude Desktop or any MCP client
- See `services/mcp-server/` for details

## Backup

```bash
# Run backup script
./scripts/backup.sh
```
Backs up: SQLite databases, blob storage, configuration.
