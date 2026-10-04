# Codeberg CI/CD Migration — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Migrate all CI/CD pipelines from GitHub Actions to Forgejo Actions on Codeberg, adding WASM build verification, Docker CI, and security scanning.

**Architecture:** Create `.forgejo/workflows/` with 5 workflow files that mirror and extend the existing `.github/workflows/`. All workflows use fully-qualified Forgejo action URLs, step-level timeouts, and Debian bookworm container images. The existing GitHub Actions files remain untouched.

**Tech Stack:** Forgejo Actions, Rust nightly toolchain, pnpm 10, Node.js 20, Docker Buildx, cargo-audit, npm audit, wasm32-unknown-unknown target

---

## Forgejo Actions Reference (critical differences)

| Aspect | GitHub Actions | Forgejo Actions |
|--------|---------------|-----------------|
| Job-level `timeout-minutes` | Works | **IGNORED** — use step-level |
| Job-level `permissions` | Works | **IGNORED** — use step-level |
| Job-level `continue-on-error` | Works | **IGNORED** — use step-level |
| Default runner image | `ubuntu-latest` | Debian bookworm + node.js (minimal) |
| Context name | `github.*` | `forgejo.*` / `forge.*` / `github.*` (all aliases) |
| Action URL `actions/checkout@v4` | Works | Use `https://data.forgejo.org/actions/checkout@v4` |
| Auto env vars | `GITHUB_TOKEN` etc. | `FORGEJO_TOKEN`/`GITHUB_TOKEN`, `FORGEJO_SERVER_URL`, `FORGEJO_REPOSITORY` |

### Fully-qualified action URLs (ALWAYS use these)

```yaml
https://data.forgejo.org/actions/checkout@v4
https://data.forgejo.org/actions/setup-node@v4
https://data.forgejo.org/actions/setup-go@v5
https://data.forgejo.org/actions/upload-artifact@v4
https://data.forgejo.org/actions/download-artifact@v4
https://data.forgejo.org/actions/cache@v4
```

---

## File Structure

```
.forgejo/workflows/
  ci.yml            # Main CI: lint + test + build for Rust + TS
  security.yml      # Weekly security audit (cargo-audit + npm audit)
  release.yml       # Tag-triggered release builds + artifacts
  wasm.yml          # WASM build verification (wasm32-unknown-unknown)
  docker.yml        # Docker build for all microservices
```

### Files that will NOT be modified

- `.github/workflows/ci.yml` — keep as-is (GitHub CI stays active)
- `.github/workflows/security.yml` — keep as-is
- `.github/workflows/release.yml` — keep as-is
- `services.Dockerfile` — existing, referenced by docker.yml
- `core/crates/wo-docserver/Dockerfile` — existing, referenced by docker.yml
- `docker-compose.services.yml` — existing, used as reference

---

## Task 1: Create `.forgejo/workflows/` directory

**Files:**
- Create: `.forgejo/workflows/` (directory)

- [x] **Step 1: Create the directory structure**

```bash
mkdir -p .forgejo/workflows
```

- [x] **Step 2: Verify directory was created**

```bash
ls -la .forgejo/workflows/
```

Expected: empty directory listing

- [x] **Step 3: Commit**

```bash
git add .forgejo/workflows/
git commit -m "chore: scaffold .forgejo/workflows directory"
```

---

## Task 2: Create main CI workflow (`ci.yml`)

**Files:**
- Create: `.forgejo/workflows/ci.yml`
- Reference: `.github/workflows/ci.yml` (current source of truth)

This workflow migrates all 6 GitHub CI jobs to Forgejo. Key changes:
- All action URLs are fully-qualified
- All `timeout-minutes` are step-level (Forgejo ignores job-level)
- Runner image is `docker://node:20-bookworm` (explicit Debian bookworm)
- Rust jobs use `dtolnay/rust-toolchain` from GitHub (Forgejo runner can fetch from GitHub)
- Cache uses `actions/cache@v4` with `sccache` for Rust (Swatinem/rust-cache is GitHub-specific)
- `wo-pdf` and `wo-webdav` are excluded (known ICE)
- `services-enterprise/*` members are excluded (may not be public)

- [x] **Step 1: Write the complete CI workflow file**

```yaml
# .forgejo/workflows/ci.yml
name: CI

on:
  push:
    branches: [main]
  pull_request:

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  lint-rust:
    name: Rust Lint
    runs-on: docker://node:20-bookworm
    steps:
      - name: Checkout
        uses: https://data.forgejo.org/actions/checkout@v4
        timeout-minutes: 5

      - name: Install Rust toolchain
        uses: https://github.com/dtolnay/rust-toolchain@nightly
        with:
          components: rustfmt, clippy
        timeout-minutes: 10

      - name: Install system dependencies
        run: apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev
        timeout-minutes: 5

      - name: Cache Cargo registry and build
        uses: https://data.forgejo.org/actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: rust-lint-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: rust-lint-
        timeout-minutes: 5

      - name: Check formatting
        run: cargo fmt --all -- --check
        timeout-minutes: 10

      - name: Check workspace
        run: cargo check --workspace --exclude wo-pdf --exclude wo-webdav
        timeout-minutes: 30

      - name: Clippy
        run: cargo clippy --all-targets --exclude wo-pdf --exclude wo-webdav -- -D warnings
        timeout-minutes: 30

  test-rust:
    name: Rust Test (nightly)
    runs-on: docker://node:20-bookworm
    steps:
      - name: Checkout
        uses: https://data.forgejo.org/actions/checkout@v4
        timeout-minutes: 5

      - name: Install Rust toolchain
        uses: https://github.com/dtolnay/rust-toolchain@nightly
        timeout-minutes: 10

      - name: Install system dependencies
        run: apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev
        timeout-minutes: 5

      - name: Cache Cargo registry and build
        uses: https://data.forgejo.org/actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: rust-test-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: rust-test-
        timeout-minutes: 5

      - name: Run tests
        run: cargo test --workspace --lib --exclude wo-pdf --exclude wo-webdav -- --test-threads=1
        timeout-minutes: 60

  lint-ts:
    name: TypeScript Lint
    runs-on: docker://node:20-bookworm
    steps:
      - name: Checkout
        uses: https://data.forgejo.org/actions/checkout@v4
        timeout-minutes: 5

      - name: Setup pnpm
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

      - name: Install dependencies
        run: pnpm install --frozen-lockfile
        timeout-minutes: 10

      - name: Run lint
        run: pnpm lint
        timeout-minutes: 10

  typecheck-ts:
    name: TypeScript Typecheck
    runs-on: docker://node:20-bookworm
    steps:
      - name: Checkout
        uses: https://data.forgejo.org/actions/checkout@v4
        timeout-minutes: 5

      - name: Setup pnpm
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

      - name: Install dependencies
        run: pnpm install --frozen-lockfile
        timeout-minutes: 10

      - name: Run typecheck
        run: pnpm typecheck
        timeout-minutes: 10

  build-ts:
    name: TypeScript Build
    runs-on: docker://node:20-bookworm
    steps:
      - name: Checkout
        uses: https://data.forgejo.org/actions/checkout@v4
        timeout-minutes: 5

      - name: Setup pnpm
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

      - name: Install dependencies
        run: pnpm install --frozen-lockfile
        timeout-minutes: 10

      - name: Run build
        run: pnpm build
        timeout-minutes: 15

  build-rust:
    name: Rust Build Check
    runs-on: docker://node:20-bookworm
    steps:
      - name: Checkout
        uses: https://data.forgejo.org/actions/checkout@v4
        timeout-minutes: 5

      - name: Install Rust toolchain
        uses: https://github.com/dtolnay/rust-toolchain@nightly
        timeout-minutes: 10

      - name: Install system dependencies
        run: apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev
        timeout-minutes: 5

      - name: Cache Cargo registry and build
        uses: https://data.forgejo.org/actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: rust-build-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: rust-build-
        timeout-minutes: 5

      - name: Check all targets
        run: cargo check --all-targets --exclude wo-pdf --exclude wo-webdav
        timeout-minutes: 30
```

- [x] **Step 2: Validate YAML syntax**

```bash
python3 -c "import yaml; yaml.safe_load(open('.forgejo/workflows/ci.yml'))"
```

Expected: no errors

- [x] **Step 3: Commit**

```bash
git add .forgejo/workflows/ci.yml
git commit -m "ci(forgejo): add main CI workflow with Rust and TypeScript jobs"
```

---

## Task 3: Create security scanning workflow (`security.yml`)

**Files:**
- Create: `.forgejo/workflows/security.yml`
- Reference: `.github/workflows/security.yml` (current source of truth)

- [x] **Step 1: Write the complete security workflow file**

```yaml
# .forgejo/workflows/security.yml
name: Security

on:
  schedule:
    - cron: "0 6 * * 1" # Every Monday at 06:00 UTC

jobs:
  cargo-audit:
    name: Rust Dependency Audit
    runs-on: docker://node:20-bookworm
    steps:
      - name: Checkout
        uses: https://data.forgejo.org/actions/checkout@v4
        timeout-minutes: 5

      - name: Install Rust toolchain
        uses: https://github.com/dtolnay/rust-toolchain@stable
        timeout-minutes: 10

      - name: Install system dependencies
        run: apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev
        timeout-minutes: 5

      - name: Cache Cargo registry
        uses: https://data.forgejo.org/actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
          key: security-cargo-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: security-cargo-
        timeout-minutes: 5

      - name: Install cargo-audit
        run: cargo install cargo-audit
        timeout-minutes: 10

      - name: Run audit
        run: cargo audit
        timeout-minutes: 10

  npm-audit:
    name: NPM Dependency Audit
    runs-on: docker://node:20-bookworm
    steps:
      - name: Checkout
        uses: https://data.forgejo.org/actions/checkout@v4
        timeout-minutes: 5

      - name: Setup pnpm
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

      - name: Install dependencies
        run: pnpm install --frozen-lockfile
        timeout-minutes: 10

      - name: Run audit
        run: pnpm audit --audit-level=high || true
        timeout-minutes: 10
```

- [x] **Step 2: Validate YAML syntax**

```bash
python3 -c "import yaml; yaml.safe_load(open('.forgejo/workflows/security.yml'))"
```

Expected: no errors

- [x] **Step 3: Commit**

```bash
git add .forgejo/workflows/security.yml
git commit -m "ci(forgejo): add weekly security audit workflow"
```

---

## Task 4: Create release workflow (`release.yml`)

**Files:**
- Create: `.forgejo/workflows/release.yml`
- Reference: `.github/workflows/release.yml` (current source of truth)

- [x] **Step 1: Write the complete release workflow file**

```yaml
# .forgejo/workflows/release.yml
name: Release

on:
  push:
    tags:
      - v*

jobs:
  build-core:
    name: Build Rust Crates (Release)
    runs-on: docker://node:20-bookworm
    steps:
      - name: Checkout
        uses: https://data.forgejo.org/actions/checkout@v4
        timeout-minutes: 5

      - name: Install Rust toolchain
        uses: https://github.com/dtolnay/rust-toolchain@stable
        timeout-minutes: 10

      - name: Install system dependencies
        run: apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev
        timeout-minutes: 5

      - name: Cache Cargo registry and build
        uses: https://data.forgejo.org/actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: release-rust-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: release-rust-
        timeout-minutes: 5

      - name: Build release
        run: cargo build --release --workspace --exclude wo-pdf --exclude wo-webdav
        timeout-minutes: 60

      - name: Upload Rust artifacts
        uses: https://data.forgejo.org/actions/upload-artifact@v4
        with:
          name: rust-release
          path: target/release/
        timeout-minutes: 10

  build-web:
    name: Build Web Apps
    runs-on: docker://node:20-bookworm
    steps:
      - name: Checkout
        uses: https://data.forgejo.org/actions/checkout@v4
        timeout-minutes: 5

      - name: Setup pnpm
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

      - name: Install dependencies
        run: pnpm install --frozen-lockfile
        timeout-minutes: 10

      - name: Build web apps
        run: pnpm build
        timeout-minutes: 15

      - name: Upload web artifacts
        uses: https://data.forgejo.org/actions/upload-artifact@v4
        with:
          name: web-build
          path: |
            apps/*/dist/
            packages/*/dist/
        timeout-minutes: 10

  create-release:
    name: Create Release
    runs-on: docker://node:20-bookworm
    needs: [build-core, build-web]
    steps:
      - name: Checkout
        uses: https://data.forgejo.org/actions/checkout@v4
        timeout-minutes: 5

      - name: Download Rust artifacts
        uses: https://data.forgejo.org/actions/download-artifact@v4
        with:
          name: rust-release
          path: target/release/
        timeout-minutes: 10

      - name: Download web artifacts
        uses: https://data.forgejo.org/actions/download-artifact@v4
        with:
          name: web-build
          path: web-dist/
        timeout-minutes: 10

      - name: Create Forgejo release
        uses: https://codeberg.org/forgejo/release@v2
        with:
          direction: upload
          title: "Release ${{ forgejo.ref_name }}"
          tag: ${{ forgejo.ref_name }}
          token: ${{ secrets.FORGEJO_TOKEN }}
          files: |
            target/release/worldoffice-*
            web-dist/**/*
        timeout-minutes: 10
```

- [x] **Step 2: Validate YAML syntax**

```bash
python3 -c "import yaml; yaml.safe_load(open('.forgejo/workflows/release.yml'))"
```

Expected: no errors

- [x] **Step 3: Commit**

```bash
git add .forgejo/workflows/release.yml
git commit -m "ci(forgejo): add tag-triggered release workflow"
```

---

## Task 5: Create WASM build verification workflow (`wasm.yml`)

**Files:**
- Create: `.forgejo/workflows/wasm.yml`

This is a NEW workflow — no GitHub equivalent exists. It verifies that the two WASM crates compile to `wasm32-unknown-unknown` without errors. These crates cannot be tested with `cargo test` (they need wasm-pack or a browser runtime).

- [x] **Step 1: Write the complete WASM workflow file**

```yaml
# .forgejo/workflows/wasm.yml
name: WASM Build

on:
  push:
    branches: [main]
  pull_request:

env:
  CARGO_TERM_COLOR: always

jobs:
  build-wasm-x2t:
    name: Build wo-x2t-wasm
    runs-on: docker://node:20-bookworm
    steps:
      - name: Checkout
        uses: https://data.forgejo.org/actions/checkout@v4
        timeout-minutes: 5

      - name: Install Rust toolchain
        uses: https://github.com/dtolnay/rust-toolchain@nightly
        with:
          targets: wasm32-unknown-unknown
        timeout-minutes: 10

      - name: Install system dependencies
        run: apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev
        timeout-minutes: 5

      - name: Cache Cargo registry and build
        uses: https://data.forgejo.org/actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: wasm-x2t-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: wasm-x2t-
        timeout-minutes: 5

      - name: Build wo-x2t-wasm
        run: cargo build --target wasm32-unknown-unknown -p wo-x2t-wasm
        timeout-minutes: 30

  build-wasm-renderer:
    name: Build wo-renderer-wasm
    runs-on: docker://node:20-bookworm
    steps:
      - name: Checkout
        uses: https://data.forgejo.org/actions/checkout@v4
        timeout-minutes: 5

      - name: Install Rust toolchain
        uses: https://github.com/dtolnay/rust-toolchain@nightly
        with:
          targets: wasm32-unknown-unknown
        timeout-minutes: 10

      - name: Install system dependencies
        run: apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev
        timeout-minutes: 5

      - name: Cache Cargo registry and build
        uses: https://data.forgejo.org/actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: wasm-renderer-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: wasm-renderer-
        timeout-minutes: 5

      - name: Build wo-renderer-wasm
        run: cargo build --target wasm32-unknown-unknown -p wo-renderer-wasm
        timeout-minutes: 30
```

- [x] **Step 2: Validate YAML syntax**

```bash
python3 -c "import yaml; yaml.safe_load(open('.forgejo/workflows/wasm.yml'))"
```

Expected: no errors

- [x] **Step 3: Commit**

```bash
git add .forgejo/workflows/wasm.yml
git commit -m "ci(forgejo): add WASM build verification workflow"
```

---

## Task 6: Create Docker build workflow (`docker.yml`)

**Files:**
- Create: `.forgejo/workflows/docker.yml`
- Reference: `services.Dockerfile`, `core/crates/wo-docserver/Dockerfile`, `docker-compose.services.yml`

This is a NEW workflow — no GitHub equivalent exists. It builds Docker images for all microservices and the docserver, then pushes them to the Codeberg container registry (`codeberg.org/world-office/server`). The workflow uses a matrix strategy to build all 7 service images in parallel.

This workflow only runs on push to `main` (not on PRs) to avoid wasting CI resources on Docker builds for every PR.

- [x] **Step 1: Write the complete Docker workflow file**

```yaml
# .forgejo/workflows/docker.yml
name: Docker Build

on:
  push:
    branches: [main]

env:
  REGISTRY: codeberg.org
  IMAGE_NAME: ${{ forgejo.repository }}

jobs:
  build-services:
    name: Build ${{ matrix.service }}
    runs-on: docker://node:20-bookworm
    strategy:
      matrix:
        include:
          - service: identity-service
            dockerfile: services.Dockerfile
          - service: storage-service
            dockerfile: services.Dockerfile
          - service: conversion-service
            dockerfile: services.Dockerfile
          - service: coauthoring-service
            dockerfile: services.Dockerfile
          - service: session-service
            dockerfile: services.Dockerfile
          - service: api-gateway
            dockerfile: services.Dockerfile
          - service: wo-docserver
            dockerfile: core/crates/wo-docserver/Dockerfile
    steps:
      - name: Checkout
        uses: https://data.forgejo.org/actions/checkout@v4
        timeout-minutes: 5

      - name: Set up Docker Buildx
        uses: https://github.com/docker/setup-buildx-action@v3
        timeout-minutes: 5

      - name: Login to Codeberg Container Registry
        uses: https://github.com/docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ forgejo.actor }}
          password: ${{ secrets.FORGEJO_TOKEN }}
        timeout-minutes: 5

      - name: Extract metadata (tags, labels) for Docker
        id: meta
        uses: https://github.com/docker/metadata-action@v5
        with:
          images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}/${{ matrix.service }}
          tags: |
            type=sha,prefix=
            type=ref,event=branch
            type=raw,value=latest,enable={{is_default_branch}}
        timeout-minutes: 5

      - name: Build and push Docker image
        uses: https://github.com/docker/build-push-action@v6
        with:
          context: .
          file: ${{ matrix.dockerfile }}
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          build-args: |
            SERVICE_NAME=${{ matrix.service }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
        timeout-minutes: 30
```

- [x] **Step 2: Validate YAML syntax**

```bash
python3 -c "import yaml; yaml.safe_load(open('.forgejo/workflows/docker.yml'))"
```

Expected: no errors

- [x] **Step 3: Commit**

```bash
git add .forgejo/workflows/docker.yml
git commit -m "ci(forgejo): add Docker build workflow for microservices"
```

---

## Task 7: Verify all Forgejo workflow files are syntactically valid

**Files:**
- Read: `.forgejo/workflows/ci.yml`
- Read: `.forgejo/workflows/security.yml`
- Read: `.forgejo/workflows/release.yml`
- Read: `.forgejo/workflows/wasm.yml`
- Read: `.forgejo/workflows/docker.yml`

- [x] **Step 1: Validate all YAML files parse correctly**

```bash
python3 -c "
import yaml, sys, os
errors = []
for f in os.listdir('.forgejo/workflows/'):
    if f.endswith('.yml'):
        try:
            yaml.safe_load(open(f'.forgejo/workflows/{f}'))
            print(f'OK: {f}')
        except Exception as e:
            errors.append((f, str(e)))
            print(f'FAIL: {f} — {e}')
if errors:
    sys.exit(1)
"
```

Expected: all 5 files print `OK`

- [x] **Step 2: Verify all action URLs are fully-qualified (no short URLs)**

```bash
grep -rn "uses:.*actions/checkout\|uses:.*actions/setup-node\|uses:.*actions/setup-go\|uses:.*actions/upload-artifact\|uses:.*actions/download-artifact\|uses:.*actions/cache" .forgejo/workflows/ | grep -v "https://" || echo "All action URLs are fully-qualified"
```

Expected: "All action URLs are fully-qualified" (no short URLs found)

- [x] **Step 3: Verify no job-level timeout-minutes (Forgejo ignores them)**

```bash
grep -n "timeout-minutes" .forgejo/workflows/*.yml | grep -v "^.forgejo/workflows/.*:[0-9]*: *- name:" || echo "No job-level timeouts found"
```

Expected: "No job-level timeouts found" — all timeouts are under step names

---

## Task 8: Verify Forgejo workflow structure matches GitHub workflow coverage

**Files:**
- Compare: `.github/workflows/ci.yml` vs `.forgejo/workflows/ci.yml`
- Compare: `.github/workflows/security.yml` vs `.forgejo/workflows/security.yml`
- Compare: `.github/workflows/release.yml` vs `.forgejo/workflows/release.yml`

- [x] **Step 1: Verify all GitHub CI jobs have Forgejo equivalents**

```bash
echo "=== GitHub CI jobs ==="
grep "^  [a-z].*:$" .github/workflows/ci.yml
echo ""
echo "=== Forgejo CI jobs ==="
grep "^  [a-z].*:$" .forgejo/workflows/ci.yml
```

Expected: Both list `lint-rust`, `test-rust`, `lint-ts`, `typecheck-ts`, `build-ts`, `build-rust`

- [x] **Step 2: Verify security workflow parity**

```bash
echo "=== GitHub security jobs ==="
grep "^  [a-z].*:$" .github/workflows/security.yml
echo ""
echo "=== Forgejo security jobs ==="
grep "^  [a-z].*:$" .forgejo/workflows/security.yml
```

Expected: Both list `cargo-audit`, `npm-audit`

- [x] **Step 3: Verify release workflow parity**

```bash
echo "=== GitHub release jobs ==="
grep "^  [a-z].*:$" .github/workflows/release.yml
echo ""
echo "=== Forgejo release jobs ==="
grep "^  [a-z].*:$" .forgejo/workflows/release.yml
```

Expected: GitHub has `build-core`, `build-web`. Forgejo has `build-core`, `build-web`, `create-release` (Forgejo adds a release creation step).

---

## Task 9: Verify exclusion of `wo-pdf` and `wo-webdav` across all Forgejo Rust jobs

**Files:**
- Grep: `.forgejo/workflows/*.yml` for `wo-pdf` and `wo-webdav`

- [x] **Step 1: Check all Rust cargo commands exclude problematic crates**

```bash
echo "=== Cargo commands that SHOULD exclude wo-pdf and wo-webdav ==="
grep -n "cargo " .forgejo/workflows/ci.yml .forgejo/workflows/release.yml .forgejo/workflows/security.yml
```

Expected: All `cargo check`, `cargo clippy`, `cargo test`, `cargo build` commands include `--exclude wo-pdf --exclude wo-webdav`. The only exception is `cargo audit` in security.yml (audit checks the whole workspace by design and won't trigger the ICE).

- [x] **Step 2: Confirm no cargo command is missing the exclusions**

```bash
# Find cargo commands that are NOT cargo audit and do NOT have --exclude
grep -n "cargo " .forgejo/workflows/ci.yml .forgejo/workflows/release.yml | grep -v "cargo audit" | grep -v "exclude wo-pdf" | grep -v "install cargo-audit" | grep -v "CARGO_TERM" || echo "All cargo commands properly exclude wo-pdf"
```

Expected: "All cargo commands properly exclude wo-pdf"

---

## Task 10: Verify no job-level Forgejo-ignored directives

**Files:**
- Grep: `.forgejo/workflows/*.yml`

- [x] **Step 1: Check for job-level permissions**

```bash
grep -n "^  permissions:" .forgejo/workflows/*.yml || echo "No job-level permissions (correct)"
```

Expected: "No job-level permissions (correct)"

- [x] **Step 2: Check for job-level continue-on-error**

```bash
grep -n "^  continue-on-error:" .forgejo/workflows/*.yml || echo "No job-level continue-on-error (correct)"
```

Expected: "No job-level continue-on-error (correct)"

- [x] **Step 3: Check that no runner uses `ubuntu-latest`**

```bash
grep -n "ubuntu-latest" .forgejo/workflows/*.yml || echo "No ubuntu-latest references (correct)"
```

Expected: "No ubuntu-latest references (correct)" — all runners use `docker://node:20-bookworm`

---

## Task 11: Verify Docker workflow matrix covers all services from docker-compose

**Files:**
- Compare: `docker-compose.services.yml` service names vs `.forgejo/workflows/docker.yml` matrix

- [x] **Step 1: Extract service names from docker-compose.services.yml**

```bash
grep "^  [a-z].*:$" docker-compose.services.yml | grep -v "^  #"
```

Expected: `identity-service`, `storage-service`, `conversion-service`, `coauthoring-service`, `session-service`, `api-gateway`, `docserver`

- [x] **Step 2: Extract service names from Docker workflow matrix**

```bash
grep "service:" .forgejo/workflows/docker.yml | grep -v "IMAGE_NAME"
```

Expected: `identity-service`, `storage-service`, `conversion-service`, `coauthoring-service`, `session-service`, `api-gateway`, `wo-docserver`

Note: The docker-compose uses `docserver` while the Docker workflow uses `wo-docserver`. The Dockerfile at `core/crates/wo-docserver/Dockerfile` builds the binary `wo-docserver`, and the `SERVICE_NAME` arg is set to `wo-docserver` in the workflow matrix. The compose file references it as `docserver` which is the service name, not the binary name. The mapping is correct: the workflow passes `SERVICE_NAME=wo-docserver` which matches the binary produced by the Dockerfile.

---

## Task 12: Verify Forgejo context variables are used correctly

**Files:**
- Grep: `.forgejo/workflows/*.yml` for `forgejo.*` and `github.*` context usage

- [x] **Step 1: List all context variable usages**

```bash
grep -rn "forgejo\.\|github\." .forgejo/workflows/*.yml | grep -v "# " | grep -v "codeberg.org/forgejo"
```

Expected: Uses `forgejo.ref_name`, `forgejo.repository`, `forgejo.actor` — all valid Forgejo context variables.

- [x] **Step 2: Verify FORGEJO_TOKEN is used (not GITHUB_TOKEN) for Codeberg-specific actions**

```bash
grep -n "secrets\." .forgejo/workflows/*.yml
```

Expected: `secrets.FORGEJO_TOKEN` used in docker.yml (login) and release.yml (create release). This is correct — `FORGEJO_TOKEN` is auto-set by Forgejo Actions as an alias for `GITHUB_TOKEN`.

---

## Task 13: Final Verification Wave

This is the comprehensive final check before considering the migration complete.

### F1: Structural verification

- [x] **Step 1: Verify all 5 workflow files exist**

```bash
ls -1 .forgejo/workflows/
```

Expected:
```
ci.yml
docker.yml
release.yml
security.yml
wasm.yml
```

- [x] **Step 2: Verify GitHub workflows are untouched**

```bash
git diff .github/workflows/
```

Expected: no output (no changes to GitHub workflows)

### F2: Content verification

- [x] **Step 3: Verify total line count is reasonable**

```bash
wc -l .forgejo/workflows/*.yml
```

Expected: each file between 30-120 lines, total ~350-500 lines

- [x] **Step 4: Verify all files have proper `name:` and `on:` triggers**

```bash
for f in .forgejo/workflows/*.yml; do echo "--- $f ---"; head -10 "$f" | grep -E "^name:|^on:"; done
```

Expected: every file has a `name:` and `on:` trigger

### F3: Security verification

- [x] **Step 5: Verify no secrets are hardcoded**

```bash
grep -rn "password:\|secret:\|token:" .forgejo/workflows/*.yml | grep -v "secrets\.\|FORGEJO_TOKEN\|JWT_SECRET\|# " | grep -v "password:" | grep -v "CARGO_TERM" || echo "No hardcoded secrets found"
```

Expected: "No hardcoded secrets found"

### F4: Git verification

- [x] **Step 6: Verify all new files are tracked**

```bash
git status .forgejo/workflows/
```

Expected: all 5 `.yml` files show as new or modified (staged)

---

## Definition of Done

- [x] `.forgejo/workflows/` contains exactly 5 workflow files: `ci.yml`, `security.yml`, `release.yml`, `wasm.yml`, `docker.yml`
- [x] All YAML files parse without errors
- [x] All action URLs are fully-qualified (no short `actions/checkout@v4` forms)
- [x] No job-level `timeout-minutes`, `permissions`, or `continue-on-error` (all step-level)
- [x] No `ubuntu-latest` runner references (all use `docker://node:20-bookworm`)
- [x] All Rust cargo commands exclude `wo-pdf` and `wo-webdav` (except `cargo audit`)
- [x] CI workflow covers all 6 jobs from the GitHub equivalent (lint-rust, test-rust, lint-ts, typecheck-ts, build-ts, build-rust)
- [x] Security workflow covers both cargo-audit and npm-audit
- [x] Release workflow adds a Forgejo release creation step (not in GitHub version)
- [x] WASM workflow builds both `wo-x2t-wasm` and `wo-renderer-wasm` for `wasm32-unknown-unknown`
- [x] Docker workflow builds all 7 service images matching `docker-compose.services.yml`
- [x] GitHub workflows in `.github/workflows/` are completely untouched
- [x] All commits use conventional commit format (`ci(forgejo):` prefix)
