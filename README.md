# World-Office Website

Landing page for [world-office.graphwiz.ai](https://world-office.graphwiz.ai) — the independent,
open-source document editing suite built in Rust ([World-Office on Codeberg](https://codeberg.org/World-Office)).

- Single-file static page (`index.html`) — no build step
- Mirrors the mykovolt-landing pattern: nginx:alpine + Traefik labels, `traefik-web` network
- Brand assets from [World-Office-artwork](https://codeberg.org/World-Office/artwork) (deconstructivist geometric style: deep indigo, electric cyan, warm gold)

## Demo sandbox

`demo/docker-compose.yml` deploys a public, anonymous editor sandbox at
`world-office-demo.graphwiz.ai` (covered by the `*.graphwiz.ai` wildcard → CI — no DNS record needed).
Runs upstream **OnlyOffice Document Server** (the engine the World-Office fork is based on)
until the fork publishes its own images (codeberg registry currently empty). Swap the image in
this compose file once `codeberg.org/world-office/docserver` becomes pullable.

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
