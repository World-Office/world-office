# Enterprise Separation

**Goal:** Move `core-enterprise/`, `services-enterprise/`, `apps-web-enterprise/` to a separate private repo.

## Current State

Enterprise crates are:
- Listed as members of the root Cargo workspace
- Have their OWN self-contained workspace (`core-enterprise/Cargo.toml`) — ready for standalone use
- Only depend on `anyhow` + `thiserror` — no dependency on main OSS crates
- Code is thin (6 crates × 1 source file each, 3 services × 1 source file each)

## Steps

### 1. Create the private repo

```bash
# Outside this workspace
mkdir world-office-enterprise
cd world-office-enterprise
git init
```

### 2. Copy enterprise directories

```bash
cp -r /path/to/server/core-enterprise  ./
cp -r /path/to/server/services-enterprise  ./
cp -r /path/to/server/apps-web-enterprise ./
git add -A && git commit -m "feat: initial enterprise code"
git remote add origin <your-private-repo-url>
git push -u origin main
```

### 3. Remove enterprise from root Cargo.toml

Delete these lines from `/home/weiss/git/World-Office/server/Cargo.toml`:
```toml
    "core-enterprise/crates/wo-digital-signature",
    "core-enterprise/crates/wo-redaction",
    "core-enterprise/crates/wo-drm",
    "core-enterprise/crates/wo-watermark",
    "core-enterprise/crates/wo-comparison",
    "core-enterprise/crates/wo-converter-pro",
    "services-enterprise/audit-service",
    "services-enterprise/scim-service",
    "services-enterprise/webhook-service",
```

### 4. Delete enterprise directories from this repo

```bash
rm -rf core-enterprise services-enterprise apps-web-enterprise
```

### 5. Update CI workflows

Add a checkout step in `.forgejo/workflows/` to clone the private repo if enterprise crates should be tested:
```yaml
- name: Checkout enterprise
  uses: https://data.forgejo.org/actions/checkout@v4
  with:
    repository: your-org/world-office-enterprise
    ssh-key: ${{ secrets.ENTERPRISE_DEPLOY_KEY }}
    path: enterprise
```

### 6. Update AGENTS.md

Update the main AGENTS.md to remove enterprise crate/service references from public tables.

---

## Cargo.toml Changes (Actionable now)

The following change can be made immediately — removing enterprise members from the workspace. This won't break builds since enterprise crates are self-contained.

I'll apply this now.
