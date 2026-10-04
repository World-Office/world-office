# Tauri Desktop: Distribution Pipeline — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Automate Tauri desktop builds, publish .deb packages to a Codeberg Pages Debian repository, and establish a release workflow.

**Architecture:** A Forgejo Actions workflow performs nightly and tag-triggered builds of the existing Tauri 2.0 app in `desktop/tauri-poc/`. On tag push (`v*`), the workflow builds the app, signs the .deb with a GPG key, and publishes it to a dedicated `world-office/debian-repo` repository on the `pages` branch — served as an APT repository via Codeberg Pages. The same workflow also publishes AppImage and source tarball as release artifacts.

**Tech Stack:** Forgejo Actions, Tauri 2.0 CLI, GPG (apt repo signing), Codeberg Pages, `dpkg-deb` (manual repo index generation)

**Prerequisites:**
- Codeberg org `world-office` exists
- Repo `world-office/debian-repo` exists (private or public) with `pages` branch
- GPG key pair generated (done in Task 1)
- `DEB_GPG_PRIVATE_KEY` and `DEB_GPG_PASSPHRASE` stored in Forgejo repo secrets
- `FORGEJO_TOKEN` secret exists for release creation

---

### Task 1: Generate GPG Key and Set Up Secrets

**Files:** (no source files)

- [ ] **Step 1: Generate GPG signing key**

Run on a local dev machine once:

```bash
gpg --full-gen-key --batch <<EOF
Key-Type: RSA
Key-Length: 4096
Key-Usage: sign
Name-Real: World Office Repository
Name-Email: repo@world-office.org
Expire-Date: 3y
Passphrase: <generate-and-store-securely>
EOF

# Export private key (for CI secret)
gpg --armor --export-secret-key repo@world-office.org > debian-repo-private.asc

# Export public key (for users to trust)
gpg --armor --export repo@world-office.org > world-office.gpg
```

- [ ] **Step 2: Store secrets in Forgejo**

Using `tea` CLI or web UI:
- `DEB_GPG_PRIVATE_KEY` — contents of `debian-repo-private.asc`
- `DEB_GPG_PASSPHRASE` — the passphrase used during key generation
- `FORGEJO_TOKEN` — a Forgejo application token with `release` scope

```bash
# Using tea CLI
tea secret add DEB_GPG_PRIVATE_KEY --repo world-office/server
tea secret add DEB_GPG_PASSPHRASE --repo world-office/server
```

- [ ] **Step 3: Commit the public key to the debian-repo**

In `world-office/debian-repo`:

```bash
git checkout pages
mkdir -p .debian/
cp /path/to/world-office.gpg .debian/world-office.gpg
git add .debian/world-office.gpg
git commit -m "feat: add repository signing public key"
git push origin pages
```

---

### Task 2: CI Workflow — Build Tauri Linux Package

**Files:**
- Create: `server/.forgejo/workflows/desktop-release.yml`

- [ ] **Step 1: Create the workflow file**

```yaml
# .forgejo/workflows/desktop-release.yml
name: Desktop Build & Publish

on:
  push:
    tags:
      - v*
    branches:
      - main

concurrency:
  group: desktop-${{ forgejo.ref }}
  cancel-in-progress: true

jobs:
  build-linux:
    name: Build Tauri Desktop (Linux)
    runs-on: docker://node:20-bookworm
    steps:
      - name: Checkout
        uses: https://data.forgejo.org/actions/checkout@v4
        timeout-minutes: 5

      - name: Install Rust toolchain
        uses: https://github.com/dtolnay/rust-toolchain@stable
        timeout-minutes: 10

      - name: Install system dependencies (Tauri)
        run: |
          apt-get update && apt-get install -y --no-install-recommends \
            pkg-config libssl-dev \
            libwebkit2gtk-4.1-dev libappindicator3-dev \
            librsvg2-dev patchelf
        timeout-minutes: 5

      - name: Setup Node.js and pnpm
        uses: https://github.com/pnpm/action-setup@v4
        with:
          version: 10
        timeout-minutes: 5

      - name: Setup Node.js
        uses: https://data.forgejo.org/actions/setup-node@v4
        with:
          node-version: 20
          cache: pnpm
        timeout-minutes: 5

      - name: Cache Cargo registry
        uses: https://data.forgejo.org/actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            desktop/tauri-poc/src-tauri/target
          key: desktop-cargo-${{ hashFiles('desktop/tauri-poc/src-tauri/Cargo.lock') }}
          restore-keys: desktop-cargo-
        timeout-minutes: 5

      - name: Install dependencies
        run: pnpm install --frozen-lockfile
        timeout-minutes: 10

      - name: Build web frontend
        run: pnpm --filter @world-office/tauri-poc run build:web
        timeout-minutes: 10

      - name: Build Tauri application
        run: pnpm --filter @world-office/tauri-poc run build
        timeout-minutes: 30

      - name: Upload .deb artifact
        uses: https://data.forgejo.org/actions/upload-artifact@v4
        with:
          name: tauri-deb
          path: desktop/tauri-poc/src-tauri/target/release/bundle/deb/*.deb
        timeout-minutes: 5

      - name: Upload AppImage artifact
        uses: https://data.forgejo.org/actions/upload-artifact@v4
        with:
          name: tauri-appimage
          path: desktop/tauri-poc/src-tauri/target/release/bundle/appimage/*.AppImage
        timeout-minutes: 5

      - name: Upload source tarball
        uses: https://data.forgejo.org/actions/upload-artifact@v4
        with:
          name: source-tarball
          path: |
            !desktop/tauri-poc/src-tauri/target/
            desktop/tauri-poc/
        timeout-minutes: 5

  release:
    name: Create Release
    if: startsWith(forgejo.ref, 'refs/tags/v')
    runs-on: docker://node:20-bookworm
    needs: [build-linux]
    steps:
      - name: Checkout debian-repo
        uses: https://data.forgejo.org/actions/checkout@v4
        with:
          repository: world-office/debian-repo
          ref: pages
          token: ${{ secrets.FORGEJO_TOKEN }}
        timeout-minutes: 5

      - name: Download .deb
        uses: https://data.forgejo.org/actions/download-artifact@v4
        with:
          name: tauri-deb
          path: artifacts/deb/
        timeout-minutes: 5

      - name: Import GPG key
        run: |
          echo "${{ secrets.DEB_GPG_PRIVATE_KEY }}" | gpg --batch --import
        timeout-minutes: 2

      - name: Install repo management tools
        run: |
          apt-get update && apt-get install -y --no-install-recommends \
            dpkg-dev apt-utils
        timeout-minutes: 3

      - name: Publish to Debian repository
        run: |
          VERSION="${forgejo.ref_name#v}"
          POOLDIR=".debian/pool/main/w/world-office"
          mkdir -p "$POOLDIR"

          # Copy .deb into pool
          cp artifacts/deb/*.deb "$POOLDIR/world-office_${VERSION}_amd64.deb"

          # Regenerate Packages
          cd .debian
          dpkg-scanpackages --multiversion pool/main > dists/stable/main/binary-amd64/Packages 2>/dev/null
          gzip -9c dists/stable/main/binary-amd64/Packages > dists/stable/main/binary-amd64/Packages.gz

          # Regenerate Release
          cd dists/stable
          apt-ftparchive release . > Release 2>/dev/null
          gpg --batch --passphrase "${{ secrets.DEB_GPG_PASSPHRASE }}" \
            --pinentry-mode loopback \
            --clearsign -o InRelease Release
          gpg --batch --passphrase "${{ secrets.DEB_GPG_PASSPHRASE }}" \
            --pinentry-mode loopback \
            -abs -o Release.gpg Release
        timeout-minutes: 5

      - name: Commit and push to pages branch
        run: |
          git config user.name "World Office Bot"
          git config user.email "bot@world-office.org"
          git add .debian/
          git commit -m "chore: add world-office ${VERSION}"
          git push origin pages
        timeout-minutes: 5

      - name: Create Forgejo release
        uses: https://codeberg.org/forgejo/release@v2
        with:
          direction: upload
          title: "World Office Desktop ${{ forgejo.ref_name }}"
          tag: ${{ forgejo.ref_name }}
          token: ${{ secrets.FORGEJO_TOKEN }}
          files: |
            artifacts/deb/*.deb
        timeout-minutes: 10
```

- [ ] **Step 2: Add `FORGEJO_TOKEN` secret to the `world-office/server` repo**

Via Codeberg web UI: Settings → Actions → Secrets → Add `FORGEJO_TOKEN`

- [ ] **Step 3: Commit the workflow**

```bash
git add .forgejo/workflows/desktop-release.yml
git commit -m "feat: add desktop build and publish workflow"
git push
```

---

### Task 3: Create Debian Repository Management Script

**Files:**
- Create: `server/desktop/tauri-poc/scripts/debian-repo.sh`

- [ ] **Step 1: Create the script**

```bash
#!/usr/bin/env bash
# debian-repo.sh — Manage the World Office Debian APT repository
# Usage: ./debian-repo.sh <command> [args...]
#
# Commands:
#   init <repo-dir>      — Initialize a new repository structure
#   add <repo-dir> <deb> — Add a .deb package to the repository
#   sign <repo-dir>      — Regenerate Packages/Release files and sign
#   verify <repo-dir>    — Verify repository structure is valid

set -euo pipefail

REPO_DIR="${2:?Usage: $0 <command> <repo-dir> [deb-file]}"
COMMAND="${1:?Usage: $0 <command> <repo-dir> [deb-file]}"

case "$COMMAND" in
  init)
    mkdir -p "${REPO_DIR}/dists/stable/main/binary-amd64"
    mkdir -p "${REPO_DIR}/pool/main/w/world-office"
    echo "Initialized Debian repo at ${REPO_DIR}"
    ;;

  add)
    DEB_FILE="${3:?Usage: $0 add <repo-dir> <deb-file>}"
    if [ ! -f "$DEB_FILE" ]; then
      echo "Error: .deb file not found: $DEB_FILE"
      exit 1
    fi
    POOL_DIR="${REPO_DIR}/pool/main/w/world-office"
    VERSION=$(dpkg-deb -f "$DEB_FILE" Version)
    ARCH=$(dpkg-deb -f "$DEB_FILE" Architecture)
    cp "$DEB_FILE" "${POOL_DIR}/world-office_${VERSION}_${ARCH}.deb"
    echo "Added: world-office_${VERSION}_${ARCH}.deb"
    ;;

  sign)
    DIST="${3:-stable}"
    COMP="${4:-main}"
    ARCH="${5:-amd64}"

    # Regenerate Packages
    cd "${REPO_DIR}"
    dpkg-scanpackages --multiversion "pool/${COMP}" > "dists/${DIST}/${COMP}/binary-${ARCH}/Packages" 2>/dev/null
    gzip -9c "dists/${DIST}/${COMP}/binary-${ARCH}/Packages" > "dists/${DIST}/${COMP}/binary-${ARCH}/Packages.gz"

    # Regenerate Release
    cd "dists/${DIST}"
    apt-ftparchive release . > Release 2>/dev/null

    # Sign
    if [ -n "${GPG_PASSPHRASE:-}" ]; then
      gpg --batch --passphrase "$GPG_PASSPHRASE" --pinentry-mode loopback \
        --clearsign -o InRelease Release
      gpg --batch --passphrase "$GPG_PASSPHRASE" --pinentry-mode loopback \
        -abs -o Release.gpg Release
    else
      gpg --clearsign -o InRelease Release
      gpg -abs -o Release.gpg Release
    fi
    echo "Signed dists/${DIST}/"
    ;;

  verify)
    DIST="${3:-stable}"
    ARCH="${4:-amd64}"
    PASS=true

    # Check key files exist
    for f in "${REPO_DIR}/dists/${DIST}/InRelease" \
             "${REPO_DIR}/dists/${DIST}/Release.gpg" \
             "${REPO_DIR}/dists/${DIST}/main/binary-${ARCH}/Packages.gz"; do
      if [ ! -f "$f" ]; then
        echo "MISSING: $f"
        PASS=false
      fi
    done

    if [ "$PASS" = true ]; then
      echo "Repository structure valid."
    else
      echo "Repository structure INVALID."
      exit 1
    fi
    ;;

  *)
    echo "Unknown command: $COMMAND"
    echo "Usage: $0 <init|add|sign|verify> <repo-dir> [args...]"
    exit 1
    ;;
esac
```

- [ ] **Step 2: Make executable and commit**

```bash
chmod +x desktop/tauri-poc/scripts/debian-repo.sh
git add desktop/tauri-poc/scripts/debian-repo.sh
git commit -m "feat: add Debian repository management script"
git push
```

---

### Task 4: Write Release Process and Build Documentation

**Files:**
- Create: `server/docs/desktop/build-guide.md`

- [ ] **Step 1: Write the build guide**

```markdown
# Desktop Build Guide

## Prerequisites

- Rust nightly toolchain (see root `rust-toolchain.toml`)
- Node.js 20+ and pnpm 10
- Tauri system dependencies:
  - **Linux:** `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`
  - **macOS:** Xcode Command Line Tools
  - **Windows:** Visual Studio Build Tools + WebView2 SDK

## Development Build

```bash
cd server
pnpm install
cd desktop/tauri-poc
pnpm run build:web   # Build web frontend
pnpm tauri dev       # Run in development mode
```

## Release Build

```bash
cd server
pnpm install
cd desktop/tauri-poc
pnpm run build:web
pnpm tauri build     # Produces .deb, .AppImage (Linux)
```

Artifacts appear in `src-tauri/target/release/bundle/`.

## CI Build

On push of tag `v*`, the Forgejo Actions workflow in `.forgejo/workflows/desktop-release.yml`:
1. Builds the Tauri app in a Docker container
2. Signs the .deb with the repository GPG key
3. Publishes to the Codeberg Pages Debian repo
4. Creates a Forgejo release with attached artifacts

## Manual macOS/Windows Build

```bash
# Any platform with Tauri system deps installed:
cd desktop/tauri-poc
pnpm tauri build
```

macOS/Windows builds are not yet in CI (requires self-hosted runners). Build locally and sign manually.

## Debian Repository Install

```sh
# One-time setup
sudo curl -fsSL https://pages.codeberg.org/world-office/debian/world-office.gpg \
  -o /usr/share/keyrings/world-office.gpg

echo "deb [signed-by=/usr/share/keyrings/world-office.gpg] \
  https://pages.codeberg.org/world-office/debian stable main" \
  | sudo tee /etc/apt/sources.list.d/world-office.list

# Install
sudo apt update && sudo apt install world-office
```

## Release Checklist

1. Update version in `desktop/tauri-poc/src-tauri/Cargo.toml` and `package.json`
2. Update `CHANGELOG.md`
3. Commit: `chore: bump version to X.Y.Z`
4. Tag: `git tag vX.Y.Z && git push origin vX.Y.Z`
5. CI builds and publishes automatically
6. Verify: `apt update && apt install world-office` from a clean Debian VM
```

- [ ] **Step 2: Commit**

```bash
git add docs/desktop/build-guide.md
git commit -m "docs: add desktop build guide and release documentation"
```

---

### Task 5: Initialize the Debian Repository on Codeberg Pages

**Files:** (remote repo, not in server/ tree)

- [ ] **Step 1: Create the debian-repo repository on Codeberg**

Via Codeberg web UI or `tea`:
- Create `world-office/debian-repo`
- Initialize with a `pages` branch
- Disable issues/wiki (it's a package repo)

```bash
# Using tea CLI
tea repo create world-office/debian-repo --private=false
```

- [ ] **Step 2: Initialize repository structure**

```bash
git clone https://codeberg.org/world-office/debian-repo
cd debian-repo
git checkout --orphan pages
rm -rf *

# Copy the public signing key
mkdir -p .debian
cp /path/to/world-office.gpg .debian/

# Initialize with the script
../server/desktop/tauri-poc/scripts/debian-repo.sh init .debian/

git add .debian/
git commit -m "chore: initialize Debian repository"
git push origin pages
```

**The resulting repo URL:** `https://codeberg.org/world-office/debian-repo`
**Published at:** `https://world-office.codeberg.page/debian/`
**APT source line:**
```
deb [signed-by=/usr/share/keyrings/world-office.gpg] https://world-office.codeberg.page/debian stable main
```

---

### Task 6: Verify the End-to-End Pipeline

- [ ] **Step 1: Create a test tag**

```bash
git tag v0.1.0-test.1
git push origin v0.1.0-test.1
```

- [ ] **Step 2: Monitor CI run**

Watch the workflow at `https://codeberg.org/world-office/server/actions`.

**Expected timeline:**
- 0-10s: Workflow starts
- 2-5m: Dependencies install (Cargo cache hit helps)
- 5-10m: Web frontend builds
- 10-25m: Tauri builds (bottleneck — Rust compilation)
- 25-28m: .deb signed and pushed to debian-repo
- 28-30m: Forgejo release created

- [ ] **Step 3: Verify the Debian repo**

From a clean Debian/Ubuntu VM:

```sh
sudo curl -fsSL https://world-office.codeberg.page/debian/world-office.gpg \
  -o /usr/share/keyrings/world-office.gpg
echo "deb [signed-by=/usr/share/keyrings/world-office.gpg] \
  https://world-office.codeberg.page/debian stable main" \
  | sudo tee /etc/apt/sources.list.d/world-office.list
sudo apt update
apt-cache show world-office
sudo apt install world-office
# Verify app launches (headless): world-office --help
```

- [ ] **Step 4: Verify the Forgejo release**

Check `https://codeberg.org/world-office/server/releases` for the release with attached .deb and .AppImage.

---

## Verification Checklist

- [ ] `desktop-release.yml` workflow exists and parses in Forgejo
- [ ] Tag push triggers build-linux → release jobs
- [ ] .deb artifact appears in workflow run
- [ ] GPG-signed .deb added to `world-office/debian-repo` `pages` branch
- [ ] `http://world-office.codeberg.page/debian/dists/stable/InRelease` returns 200
- [ ] `apt update` from clean Debian VM succeeds
- [ ] `apt install world-office` installs and app runs
- [ ] Forgejo release created with .deb + AppImage attachments
- [ ] `debian-repo.sh` script works for local repo management
- [ ] `build-guide.md` documents all steps
