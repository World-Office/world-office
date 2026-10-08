# World-Office Website

Landing page for [world-office.graphwiz.ai](https://world-office.graphwiz.ai) — the independent,
open-source document editing suite built in Rust ([World-Office on Codeberg](https://codeberg.org/World-Office)).

- Single-file static page (`index.html`) — no build step
- Mirrors the mykovolt-landing pattern: nginx:alpine + Traefik labels, `traefik-web` network
- Brand assets from [World-Office-artwork](https://codeberg.org/World-Office/artwork) (deconstructivist geometric style: deep indigo, electric cyan, warm gold)

## Demo sandbox

Test page at `world-office.graphwiz.ai/demo/` (directory-mounted → git pull propagates live).
Visitors pick a **predefined document** (`demo/files/*.docx`, generated minimal OOXML — regen via
`python3 gen-demo-docx.py demo/files` pattern in git history) and type in it. **No upload, no login.**
The embedded editor comes from `world-office-demo.graphwiz.ai` (upstream OnlyOffice Document
Server — the engine the fork is based on; swap its image once codeberg publishes the fork's own).

Wiring: `document.url` points at the public site URL (passes the DS private-IP SSRF filter,
reachable via hairpin from the demo container — verified); save callbacks hit
`/wo-demo/callback` in `nginx-default.conf`, a JSON no-op (`{"error":0}`) so nothing persists.
Doc key = `<file>-<YYYY-MM-DD-HH>` → same-hour visitors co-edit live; the hourly purge
(recreates the DS container) wipes redis+cache, so the next visitor starts pristine.
The engine host is noindexed via `X-Robots-Tag` middleware; `EXAMPLE=true` was dropped
(the example service never starts in this image — supervisor shows `ds:example STOPPED`).

No-harm properties: container-local storage (docs vanish on `docker compose down`),
mem 3G / no swap, 2 CPUs, OOM score 500, traefik rate limit 50 rps avg / 100 burst,
JWT disabled (nothing to protect), example app pre-seeded with sample docs.

```bash
# on CI, from ~/git/wo-website
sudo docker compose -f demo/docker-compose.yml up -d
# reset the sandbox (wipes all demo documents)
sudo demo/purge.sh
```

Automatic purge: hourly root cron on CI (`15 * * * *`) runs `demo/purge.sh` —
recreates the container, wiping all demo content (~15 s outage at :15 each hour).

## Deploy

```bash
ssh weiss@195.90.216.159   # CI host (DNS world-office.graphwiz.ai → 195.90.216.159)
cd ~/git/wo-website
sudo docker compose config --quiet && sudo docker compose up -d
```

Traefik issues the Let's Encrypt certificate automatically on first HTTPS hit.
