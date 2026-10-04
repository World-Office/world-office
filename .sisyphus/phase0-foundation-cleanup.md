# Phase 0: Foundation Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix broken Rust workspace, align turbo.json with full monorepo, and harden CI pipeline so green means everything compiles.

**Architecture:** The monorepo has two build systems — Cargo (Rust, 40 crates) and Turbo/pnpm (TypeScript, 6 editor apps + packages). Currently cargo check fails due to missing workspace dependency, turbo.json lacks Rust tasks, and CI doesn't test the full workspace. This plan makes both build systems fully functional and synchronized.

**Tech Stack:** Rust (edition 2021/2024), Cargo workspaces, Turbo 2.4, pnpm 10, GitHub Actions CI

---

## Context

### Root Workspace
- File: `Cargo.toml` (82 lines) — 40 workspace members
- Edition: `2024` (root), but 21 core crates override to `2021`
- `cargo check` FAILS: `unicode-normalization` missing from `[workspace.dependencies]`
- 6 crates use `roxmltree = "0.20"` directly (not workspace dep) — works fine

### Core Sub-Workspace
- File: `core/Cargo.toml` (38 lines) — separate workspace with 21 members
- Has its own `[workspace.dependencies]` including `unicode-normalization = "0.1"`, `roxmltree = "0.20"`
- `cargo check` from core/ works fine

### Turbo
- File: `turbo.json` (30 lines) — 6 tasks (build, dev, test, lint, typecheck, clean)
- NO Rust tasks at all

### Root package.json
- File: `package.json` (26 lines) — scripts delegate to turbo
- No `cargo:*` scripts

### CI
- File: `.github/workflows/ci.yml` (77 lines)
- Runs: cargo fmt, clippy, test (stable+beta), pnpm lint, typecheck
- `cargo clippy --all-targets` will FAIL until workspace dep is fixed

### Editor React Apps (all passing)
- `apps/web/apps/{pdf,visio,presentation,document,spreadsheet}editor-react/`
- All: `tsc --noEmit` ✅, `vite build` ✅, `biome lint` ✅

---

### Task 1: Fix Rust workspace dependencies

**Files:**
- Modify: `Cargo.toml:62-82` (root)

- [x] **Step 1: Add missing `unicode-normalization` to root workspace dependencies**

Add `unicode-normalization = "0.1"` to `[workspace.dependencies]` in root `Cargo.toml` (line 82, after `image`).

The new end of file should be:
```toml
image = { version = "0.25", default-features = false, features = ["png", "jpeg", "tiff", "gif", "bmp"] }
unicode-normalization = "0.1"
```

- [x] **Step 2: Run `cargo check --workspace` from project root**

Expected: SUCCESS (zero errors). If it fails with a different missing dep, add that too.

- [x] **Step 3: Run `cargo clippy --workspace -- -D warnings`**

Expected: Many warnings (clippy pedantic on stubs) but zero errors. If clippy errors exist, fix them.

- [x] **Step 4: Commit**

```bash
git add Cargo.toml
git commit -m "fix: add missing unicode-normalization to workspace deps"
```

---

### Task 2: Add Rust tasks to turbo.json

**Files:**
- Modify: `turbo.json`
- Modify: `package.json`

- [x] **Step 1: Add Rust tasks to turbo.json**

Add `cargo:check`, `cargo:build`, `cargo:test`, `cargo:lint`, `cargo:fmt` tasks to `turbo.json`:

```json
{
  "$schema": "https://turbo.build/schema.json",
  "globalDependencies": ["**/.env.*local"],
  "tasks": {
    "build": {
      "dependsOn": ["^build"],
      "outputs": ["dist/**", ".next/**", "!.next/cache/**"]
    },
    "dev": {
      "dependsOn": ["^dev"],
      "cache": false,
      "persistent": true
    },
    "test": {
      "dependsOn": ["^build"],
      "outputs": ["coverage/**"]
    },
    "lint": {
      "dependsOn": ["^build"],
      "outputs": []
    },
    "typecheck": {
      "dependsOn": ["^build"],
      "outputs": []
    },
    "clean": {
      "cache": false
    },
    "cargo:check": {
      "outputs": ["target/**"]
    },
    "cargo:build": {
      "dependsOn": ["cargo:check"],
      "outputs": ["target/**"]
    },
    "cargo:test": {
      "dependsOn": ["cargo:check"],
      "outputs": ["target/**"]
    },
    "cargo:lint": {
      "outputs": []
    },
    "cargo:fmt": {
      "outputs": []
    }
  }
}
```

- [x] **Step 2: Add Rust scripts to root package.json**

Add `cargo:*` scripts to the `"scripts"` block in `package.json`:

```json
"scripts": {
    "build": "turbo run build",
    "dev": "turbo run dev",
    "test": "turbo run test",
    "lint": "turbo run lint",
    "format": "biome check --write .",
    "typecheck": "turbo run typecheck",
    "clean": "turbo run clean",
    "cargo:check": "cargo check --workspace",
    "cargo:build": "cargo build --workspace",
    "cargo:test": "cargo test --workspace",
    "cargo:lint": "cargo clippy --workspace -- -D warnings",
    "cargo:fmt": "cargo fmt --all -- --check",
    "cargo:fmt:fix": "cargo fmt --all"
}
```

- [x] **Step 3: Run `pnpm cargo:check` to verify**

Expected: SUCCESS (zero errors).

- [x] **Step 4: Run `pnpm cargo:lint` to verify**

Expected: Clippy runs (warnings ok for stubs, no errors).

- [x] **Step 5: Commit**

```bash
git add turbo.json package.json
git commit -m "feat: add Rust build tasks to turbo and npm scripts"
```

---

### Task 3: Harden CI to include all workspace members

**Files:**
- Modify: `.github/workflows/ci.yml`

- [x] **Step 1: Add pnpm build step to CI**

The current CI runs `pnpm lint` and `pnpm typecheck` but NOT `pnpm build`. Add a build step. Also add `pnpm cargo:check` to the Rust lint job (so we catch workspace dep issues early).

Replace the entire ci.yml with:

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  lint-rust:
    name: Rust Lint
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - name: Check formatting
        run: cargo fmt --all -- --check
      - name: Check workspace
        run: cargo check --workspace
      - name: Clippy
        run: cargo clippy --all-targets -- -D warnings

  test-rust:
    name: Rust Test (${{ matrix.rust }})
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        rust: [stable, beta]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: ${{ matrix.rust }}
      - uses: Swatinem/rust-cache@v2
      - name: Run tests
        run: cargo test --workspace

  lint-ts:
    name: TypeScript Lint
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with:
          version: 10
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: pnpm
      - run: pnpm install --frozen-lockfile
      - run: pnpm lint

  typecheck-ts:
    name: TypeScript Typecheck
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with:
          version: 10
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: pnpm
      - run: pnpm install --frozen-lockfile
      - run: pnpm typecheck

  build-ts:
    name: TypeScript Build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with:
          version: 10
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: pnpm
      - run: pnpm install --frozen-lockfile
      - run: pnpm build

  build-rust:
    name: Rust Build Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Check all targets
        run: cargo check --all-targets
```

Key changes:
- Added `cargo check --workspace` step before clippy (catches missing workspace deps)
- Added `build-ts` job (runs `pnpm build`)
- Everything else unchanged

- [x] **Step 2: Validate YAML syntax**

Run: `python -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"`
Expected: No output (valid YAML)

- [x] **Step 3: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: add workspace check and TypeScript build to CI pipeline"
```

---

### Task 4: Verify full monorepo build end-to-end

**Files:**
- No files modified — verification only

- [x] **Step 1: Run `cargo check --workspace`**

Expected: SUCCESS (zero errors)

- [x] **Step 2: Run `cargo test --workspace`**

Expected: Tests run (mostly stubs, some may fail on enterprise stubs — that's ok for now)

- [x] **Step 3: Run `pnpm build`**

Expected: All 6 editor apps + packages build successfully

- [x] **Step 4: Run `pnpm lint`**

Expected: Zero biome errors

- [x] **Step 5: Run `pnpm typecheck`**

Expected: Zero typecheck errors

- [x] **Step 6: Mark plan complete**

All tasks done. Record results in notepad.

---

## Final Verification Wave

### F1: cargo check --workspace passes
Run: `cargo check --workspace`
Expected: zero errors

### F2: pnpm build passes
Run: `pnpm build`
Expected: all apps build, zero errors

### F3: pnpm lint passes
Run: `pnpm lint`
Expected: zero biome errors

### F4: CI YAML is valid
Run: `python -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"`
Expected: no output (valid)
