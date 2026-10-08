# World-Office Website

Landing page for [world-office.graphwiz.ai](https://world-office.graphwiz.ai) — the independent,
open-source document editing suite built in Rust ([World-Office on Codeberg](https://codeberg.org/World-Office)).

- Single-file static page (`index.html`) — no build step
- Mirrors the mykovolt-landing pattern: nginx:alpine + Traefik labels, `traefik-web` network
- Brand assets from [World-Office-artwork](https://codeberg.org/World-Office/artwork) (deconstructivist geometric style: deep indigo, electric cyan, warm gold)

## Deploy

```bash
ssh weiss@195.90.216.159   # CI host (DNS world-office.graphwiz.ai → 195.90.216.159)
cd ~/git/wo-website
sudo docker compose config --quiet && sudo docker compose up -d
```

Traefik issues the Let's Encrypt certificate automatically on first HTTPS hit.
