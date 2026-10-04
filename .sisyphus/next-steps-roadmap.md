# World-Office Next Steps Roadmap

## TL;DR

> **Quick Summary**: Implement 10 prioritized improvements across the World-Office codebase — from quick wins (clippy, converters) to critical infrastructure (font rendering, SQLite persistence) to frontend advancement (React editor rewrites).
> 
> **Deliverables**:
> - Zero clippy warnings across workspace
> - Real font/glyph rendering (fontdb + swash + rustybuzz)
> - 33 format converters (6 new cross-format converters)
> - 4 niche format serializers (XPS, OFD, HWP, DjVu)
> - SQLite persistence for 5 microservices
> - diamond-types OT/CRDT for coauthoring
> - Real PDF output via pdf-writer
> - Functional wo-renderer-wasm render_page()
> - React editor Viewport wired to wo-renderer-wasm
> 
> **Estimated Effort**: XL (22 tasks across 4 waves)
> **Parallel Execution**: YES - 4 waves
> **Critical Path**: T2 (dep validation) → T7 (wire fonts) → T10 (draw_text) → T11/T12 (replace hacks) → T17 (PDF) → T18 (WASM render_page) → T21 (React)

---

## Context

### Original Request
User asked "what are good next implementation steps?" after completing EPUB/FB2 serializers and 4 new converters (767 tests). After presenting a prioritized 10-item list, user requested a detailed plan covering all items.

### Interview Summary
**Key Decisions**:
- Font rendering stack: fontdb + swash + rustybuzz (full Rust, WASM-friendly)
- Database: SQLite via rusqlite (embedded, zero-config)
- OT/CRDT: diamond-types (Rust-native, text-focused)
- Scope: All 10 items in one plan
- Edition: 2024 (all new deps must compile)

**Metis Review Findings**:
- Canvas has NO `draw_text()` method — must add entirely new API
- Rectangle glyph hack exists in TWO places: wo-docx-renderer + wo-renderer-wasm
- Only 5 services need SQLite (not 6 — api-gateway is pure proxy)
- All 5 services follow identical `Arc<Mutex<HashMap<String, T>>>` pattern
- EditOperation struct exists but is `#[allow(dead_code)]` — ready for diamond-types
- No font rendering deps exist — all must be added fresh
- Two Cargo.toml files: root (99 lines, services) + core (61 lines, parsers)
- wo-pdf has known ICE — skip entirely

### Auto-Resolved Decisions
- **5 services** need SQLite (not 6): api-gateway is pure proxy, server is JS, admin-panel is TS
- **Font rendering v1**: Latin-only, no kerning/ligatures/bidi/variable fonts
- **PDF output**: pdf-writer crate (lightweight, WASM-compatible, low-level)
- **diamond-types v1**: Plain text OT only (insert/delete, no rich text)
- **Repository pattern**: Per-service implementation (no shared crate, simpler)
- **WASM requirement**: All new deps must compile to wasm32-unknown-unknown
- **React scope**: Wire Viewport to wo-renderer-wasm for documenteditor-react; scaffold other editors

---

## Work Objectives

### Core Objective
Transform World-Office from a format-parsing codebase into a rendering-capable, persistent, collaborative document platform.

### Concrete Deliverables
- `core/crates/wo-renderer/src/text.rs` — real glyph rendering via swash
- `core/crates/wo-renderer/src/canvas.rs` — new `draw_text()` method
- `core/crates/wo-docx-renderer/src/pipeline.rs` — rectangle hacks eliminated
- `core/crates/wo-renderer-wasm/src/canvas_bridge.rs` — rectangle hack eliminated
- `core/crates/wo-xps/src/serializer.rs` — XPS serializer
- `core/crates/wo-ofd/src/serializer.rs` — OFD serializer
- `core/crates/wo-hwp/src/serializer.rs` — HWP serializer
- `core/crates/wo-djvu/src/serializer.rs` — DjVu serializer
- `services/*/src/` — SQLite persistence for 5 services
- `services/coauthoring-service/` — diamond-types integration

### Definition of Done
- [x] `cargo clippy --workspace` produces 0 warnings (excluding wo-pdf)
- [x] `cargo test --workspace --lib -- --test-threads=1` passes with 900+ tests
- [x] Rectangle glyph hacks eliminated from wo-docx-renderer and wo-renderer-wasm
- [x] PDF output starts with `%PDF-` header
- [x] 5 services persist data across restarts (SQLite)
- [x] Concurrent edits in coauthoring preserve both insertions

### Must Have
- All new deps compile on nightly + edition 2024
- All new deps compile to wasm32-unknown-unknown (font stack, pdf-writer)
- Zero regression in existing 767 tests
- TDD: failing test before implementation for all new functionality

### Must NOT Have (Guardrails)
- NO modification to enterprise crates (core-enterprise/*, services-enterprise/*)
- NO touching wo-pdf (known ICE)
- NO kerning, ligatures, OpenType features, variable fonts, complex scripts, bidi
- NO rich text in diamond-types v1 (plain text insert/delete only)
- NO encryption, connection pooling, multi-tenancy in SQLite
- NO modifications to server/ directory (JavaScript — out of scope)
- NO features added to converter list mid-plan

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** - ALL verification is agent-executed.

### Test Decision
- **Infrastructure exists**: YES (cargo test, 767 tests)
- **Automated tests**: YES (TDD — RED-GREEN-REFACTOR)
- **Framework**: cargo test (Rust native)
- **WSL Required**: All cargo commands via `wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo ..."`

### QA Policy
Every task MUST include agent-executed QA scenarios.
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

- **Rust crates**: Bash (WSL cargo test) — run tests, assert pass count
- **WASM**: Bash (WSL cargo build --target wasm32) — compilation check
- **Services**: Bash (WSL cargo test) — unit tests with :memory: SQLite
- **Rendering**: Visual QA not required — assert no rectangle hacks remain via grep

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately — foundation + quick wins, 8 tasks):
├── T1:  Fix clippy warnings across workspace [quick]
├── T2:  Validate new dependencies (fontdb/swash/rustybuzz/rusqlite/diamond-types/pdf-writer) [quick]
├── T3:  SQLite for storage-service [medium]
├── T4:  SQLite for identity-service [medium]
├── T5:  SQLite for conversion-service [medium]
├── T6:  SQLite for session-service [medium]
├── T7:  Wire wo-fonts into wo-renderer (dependency + FontMetrics) [deep]
└── T8:  Add XPS + OFD serializers [deep]

Wave 2 (After Wave 1 — core rendering + more converters, 7 tasks):
├── T9:  Add Canvas::draw_text() with swash glyph rasterization [deep]
├── T10: Replace rectangle-glyph hacks in wo-docx-renderer + wo-renderer-wasm [deep]
├── T11: Add reverse converters batch 1 (DOCX→ODT, ODT→DOCX, RTF→DOCX) [deep]
├── T12: Add reverse converters batch 2 (EPUB→DOCX, FB2→DOCX, DOCX→EPUB) [deep]
├── T13: Add HWP + DjVu serializers [deep]
├── T14: SQLite for coauthoring-service [medium]
└── T15: Wire diamond-types OT into coauthoring-service [deep]

Wave 3 (After Wave 2 — downstream rendering, 3 tasks):
├── T16: Real PDF output via pdf-writer crate [deep]
├── T17: Fix wo-renderer-wasm render_page() stub [deep]
└── T18: Add remaining reverse converters for new serializers (XPS→DOCX, etc.) [medium]

Wave 4 (After Wave 3 — frontend, 2 tasks):
├── T19: Wire documenteditor-react Viewport to wo-renderer-wasm [unspecified-high]
└── T20: Advance remaining React editors (spreadsheet/presentation/pdf/visio) [unspecified-high]

Wave FINAL (After ALL tasks — 4 parallel reviews):
├── F1: Plan compliance audit (oracle)
├── F2: Code quality review (unspecified-high)
├── F3: Real manual QA (unspecified-high)
└── F4: Scope fidelity check (deep)

Critical Path: T2 → T7 → T9 → T10 → T16 → T17 → T19
Max Concurrent: 8 (Wave 1)
```

### Dependency Matrix

| Task | Depends On | Blocks | Wave |
|------|-----------|--------|------|
| T1 | — | T9, T16, T17 | 1 |
| T2 | — | T7, T9, T10, T14, T15, T16, T17 | 1 |
| T3 | — | — | 1 |
| T4 | — | — | 1 |
| T5 | — | — | 1 |
| T6 | — | — | 1 |
| T7 | T2 | T9, T10, T11, T12 | 1 |
| T8 | — | T18 | 1 |
| T9 | T2, T7 | T10, T16, T17 | 2 |
| T10 | T9 | T16, T17 | 2 |
| T11 | T7 | — | 2 |
| T12 | T7 | — | 2 |
| T13 | — | T18 | 2 |
| T14 | T2 | T15 | 2 |
| T15 | T14 | — | 2 |
| T16 | T9, T10 | — | 3 |
| T17 | T9, T10 | T19 | 3 |
| T18 | T8, T13 | — | 3 |
| T19 | T17 | — | 4 |
| T20 | T17 | — | 4 |

### Agent Dispatch Summary

- **Wave 1**: 8 tasks — T1→`quick`, T2→`quick`, T3-T6→`medium`, T7→`deep`, T8→`deep`
- **Wave 2**: 7 tasks — T9→`deep`, T10→`deep`, T11-T12→`deep`, T13→`deep`, T14→`medium`, T15→`deep`
- **Wave 3**: 3 tasks — T16→`deep`, T17→`deep`, T18→`medium`
- **Wave 4**: 2 tasks — T19→`unspecified-high`, T20→`unspecified-high`
- **FINAL**: 4 tasks — F1→`oracle`, F2→`unspecified-high`, F3→`unspecified-high`, F4→`deep`

---

## TODOs

- [x] 1. Fix clippy warnings across workspace

  **What to do**:
  - Fix all pre-existing clippy warnings in: wo-common (2), wo-txt (1), wo-html (1), wo-rtf (9), wo-odf (4), wo-ofd (1), wo-djvu (1), wo-hwp (3)
  - Use `cargo clippy --fix --allow-dirty` for auto-fixable warnings
  - Manually fix remaining warnings (match collapses, impl derives, etc.)
  - Do NOT touch wo-pdf (known ICE)
  - Do NOT touch enterprise crates

  **Must NOT do**:
  - Change any public API or behavior
  - Remove or skip any tests
  - Touch wo-pdf or enterprise crates

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with T2-T8)
  - **Blocks**: T9, T16, T17
  - **Blocked By**: None

  **References**:
  - `core/crates/wo-common/src/encoding.rs` — 2 warnings (empty line after doc comment, impl can be derived)
  - `core/crates/wo-txt/src/parser.rs:63` — 1 warning
  - `core/crates/wo-html/src/parser.rs:162` — 1 warning
  - `core/crates/wo-rtf/src/parser.rs` — 9 warnings (match collapses, impl derives, clamp patterns, etc.)
  - `core/crates/wo-odf/src/parser.rs:795`, `core/crates/wo-odf/src/serializer.rs:131,291,373` — 4 warnings
  - `core/crates/wo-ofd/src/parser.rs:138,139` — 1 warning
  - `core/crates/wo-djvu/src/parser.rs:132` — 1 warning
  - `core/crates/wo-hwp/src/model.rs:100,115,129` — 3 warnings

  **Acceptance Criteria**:
  - [ ] `cargo clippy -p wo-common -p wo-txt -p wo-html -p wo-rtf -p wo-odf -p wo-ofd -p wo-djvu -p wo-hwp` → 0 warnings
  - [ ] `cargo test --workspace --lib -- --test-threads=1` → 767 tests pass

  **QA Scenarios**:
  ```
  Scenario: Clippy clean for all target crates
    Tool: Bash (WSL)
    Steps:
      1. wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo clippy -p wo-common -p wo-txt -p wo-html -p wo-rtf -p wo-odf -p wo-ofd -p wo-djvu -p wo-hwp 2>&1 | grep -c 'warning\['"
      2. Assert count is 0
    Expected Result: 0 warnings
    Evidence: .sisyphus/evidence/task-1-clippy-clean.txt

  Scenario: No test regressions
    Tool: Bash (WSL)
    Steps:
      1. wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test --workspace --lib -- --test-threads=1 2>&1 | grep 'FAILED'"
      2. Assert no FAILED lines
    Expected Result: 0 failures, 767 tests pass
    Evidence: .sisyphus/evidence/task-1-tests-pass.txt
  ```

  **Commit**: YES
  - Message: `chore: fix clippy warnings across workspace`
  - Pre-commit: `cargo test --workspace --lib -- --test-threads=1`

- [x] 2. Validate new dependencies

  **What to do**:
  - Add to `core/Cargo.toml` workspace deps: `fontdb`, `swash`, `rustybuzz` (with `rustybuzz-capi` feature if needed)
  - Add to root `Cargo.toml` workspace deps: `rusqlite` (with `"bundled"` feature), `diamond-types`
  - Add to `core/crates/wo-renderer/Cargo.toml`: `fontdb`, `swash`, `rustybuzz`, `wo-fonts` (workspace)
  - Add to `core/crates/wo-docx-renderer/Cargo.toml`: `pdf-writer`
  - Add to each service's Cargo.toml: `rusqlite` (workspace)
  - Verify ALL new deps compile on nightly + edition 2024
  - Verify font stack compiles to wasm32-unknown-unknown: `cargo build --target wasm32-unknown-unknown -p wo-renderer-wasm`
  - Verify pdf-writer compiles to wasm32 if needed
  - If any dep fails, find WASM-compatible alternative and document in plan
  - Do NOT implement any functionality — only add deps and verify compilation

  **Must NOT do**:
  - Implement any features using new deps
  - Modify any source code (.rs files) except Cargo.toml
  - Add deps that don't compile on nightly/edition 2024

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with T1, T3-T8)
  - **Blocks**: T7, T9, T14, T15, T16, T17
  - **Blocked By**: None

  **References**:
  - `C:\Users\Tobias\git\World-Office\Cargo.toml` — root workspace deps (99 lines)
  - `C:\Users\Tobias\git\World-Office\core\Cargo.toml` — core workspace deps (61 lines)
  - `core/crates/wo-renderer/Cargo.toml` — current renderer deps
  - `core/crates/wo-renderer-wasm/Cargo.toml` — current WASM deps
  - `core/crates/wo-docx-renderer/Cargo.toml` — current docx renderer deps
  - `services/storage-service/Cargo.toml` — example service deps

  **Acceptance Criteria**:
  - [ ] `cargo check --workspace` succeeds (excluding wo-pdf)
  - [ ] `cargo build --target wasm32-unknown-unknown -p wo-renderer-wasm` succeeds
  - [ ] All new deps are workspace-level in appropriate Cargo.toml

  **QA Scenarios**:
  ```
  Scenario: Workspace compiles with new deps
    Tool: Bash (WSL)
    Steps:
      1. wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo check --workspace 2>&1 | grep -E 'error\['"
      2. Assert no errors
    Expected Result: Clean compilation
    Evidence: .sisyphus/evidence/task-2-workspace-check.txt

  Scenario: WASM compilation succeeds
    Tool: Bash (WSL)
    Steps:
      1. wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo build --target wasm32-unknown-unknown -p wo-renderer-wasm 2>&1 | tail -5"
      2. Assert "Finished" in output
    Expected Result: WASM build succeeds
    Evidence: .sisyphus/evidence/task-2-wasm-build.txt

  Scenario: No test regressions from dep additions
    Tool: Bash (WSL)
    Steps:
      1. wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test --workspace --lib -- --test-threads=1 2>&1 | grep 'FAILED'"
      2. Assert no FAILED lines
    Expected Result: 767 tests still pass
    Evidence: .sisyphus/evidence/task-2-tests-pass.txt
  ```

  **Commit**: YES
  - Message: `chore: add fontdb, swash, rustybuzz, rusqlite, diamond-types, pdf-writer dependencies`
  - Pre-commit: `cargo check --workspace`

- [x] 3. SQLite persistence for storage-service

  **What to do**:
  - Add `rusqlite` dependency (from T2) to storage-service Cargo.toml
  - Create `StorageRepository` struct wrapping `rusqlite::Connection`
  - Implement `new()` with `Connection::open_in_memory()` for now (file path configurable via env)
  - Create `files` table: `id TEXT PRIMARY KEY, name TEXT, content_type TEXT, size INTEGER, path TEXT, created_at TEXT, updated_at TEXT`
  - Migrate `Arc<Mutex<HashMap<String, StoredFile>>>` to `Arc<Mutex<StorageRepository>>`
  - Implement CRUD methods: `insert_file()`, `get_file()`, `list_files()`, `delete_file()`, `get_file_content()`
  - Keep all endpoint handlers unchanged — only swap the backing store
  - Add tests using `:memory:` database that verify: insert, read, list, delete, persistence across "restarts"
  - Follow existing test pattern in storage-service (look for `#[cfg(test)]` blocks)

  **Must NOT do**:
  - Change any endpoint signatures or API behavior
  - Add connection pooling or encryption
  - Modify the actual file blob storage (keep disk-based, only metadata in SQLite)

  **Recommended Agent Profile**:
  - **Category**: `medium`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with T1, T2, T4-T8)
  - **Blocks**: None
  - **Blocked By**: T2 (for rusqlite dep)

  **References**:
  - `services/storage-service/src/main.rs` — current in-memory HashMap implementation, endpoint handlers
  - `services/storage-service/Cargo.toml` — current deps

  **Acceptance Criteria**:
  - [ ] `cargo test -p storage-service --lib -- --test-threads=1` → all tests pass
  - [ ] `grep -r 'HashMap' services/storage-service/src/` → 0 matches (fully migrated)
  - [ ] Tests verify data persists across "restarts" (drop connection, recreate, read)

  **QA Scenarios**:
  ```
  Scenario: Storage service tests pass with SQLite
    Tool: Bash (WSL)
    Steps:
      1. wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test -p storage-service --lib -- --test-threads=1 2>&1 | grep 'test result'"
      2. Assert all pass
    Expected Result: All storage-service tests pass
    Evidence: .sisyphus/evidence/task-3-storage-tests.txt

  Scenario: No HashMap references remain
    Tool: Bash
    Steps:
      1. grep -r 'HashMap' services/storage-service/src/
      2. Assert 0 matches
    Expected Result: Clean migration
    Evidence: .sisyphus/evidence/task-3-no-hashmap.txt
  ```

  **Commit**: YES
  - Message: `feat(storage-service): add SQLite persistence`
  - Pre-commit: `cargo test -p storage-service --lib -- --test-threads=1`

- [x] 4. SQLite persistence for identity-service

  **What to do**:
  - Add `rusqlite` dependency to identity-service Cargo.toml
  - Create `UserRepository` struct wrapping `rusqlite::Connection`
  - Create `users` table: `id TEXT PRIMARY KEY, username TEXT UNIQUE, email TEXT UNIQUE, password_hash TEXT, role TEXT, created_at TEXT`
  - Migrate `Arc<Mutex<HashMap<String, User>>>` to `Arc<Mutex<UserRepository>>`
  - Implement CRUD methods matching existing HashMap operations
  - Add persistence tests with `:memory:` database
  - Follow exact same pattern as T3

  **Must NOT do**:
  - Change endpoint signatures
  - Modify password hashing logic
  - Add connection pooling

  **Recommended Agent Profile**:
  - **Category**: `medium`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: None
  - **Blocked By**: T2

  **References**:
  - `services/identity-service/src/main.rs` — current implementation with 4 endpoints + tests
  - `services/identity-service/Cargo.toml`

  **Acceptance Criteria**:
  - [ ] `cargo test -p identity-service --lib -- --test-threads=1` → all tests pass
  - [ ] `grep -r 'HashMap' services/identity-service/src/` → 0 matches

  **QA Scenarios**:
  ```
  Scenario: Identity service tests pass with SQLite
    Tool: Bash (WSL)
    Steps:
      1. wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test -p identity-service --lib -- --test-threads=1 2>&1 | grep 'test result'"
    Expected Result: All pass
    Evidence: .sisyphus/evidence/task-4-identity-tests.txt
  ```

  **Commit**: YES
  - Message: `feat(identity-service): add SQLite persistence`
  - Pre-commit: `cargo test -p identity-service --lib -- --test-threads=1`

- [x] 5. SQLite persistence for conversion-service

  **What to do**:
  - Add `rusqlite` dependency to conversion-service Cargo.toml
  - Create `JobRepository` struct wrapping `rusqlite::Connection`
  - Create `conversion_jobs` table: `id TEXT PRIMARY KEY, source_format TEXT, target_format TEXT, status TEXT, input_size INTEGER, output_size INTEGER, created_at TEXT, completed_at TEXT, error TEXT`
  - Migrate `Arc<Mutex<HashMap<String, ConversionJob>>>` to `Arc<Mutex<JobRepository>>`
  - Add persistence tests with `:memory:` database

  **Must NOT do**:
  - Change endpoint signatures or conversion logic
  - Store actual file content in SQLite (only job metadata)

  **Recommended Agent Profile**:
  - **Category**: `medium`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T3, T4, T6)
  - **Parallel Group**: Wave 1
  - **Blocks**: None
  - **Blocked By**: T2

  **References**:
  - `services/conversion-service/src/main.rs` — current implementation with 5 endpoints

  **Acceptance Criteria**:
  - [ ] `cargo test -p conversion-service --lib -- --test-threads=1` → all tests pass
  - [ ] `grep -r 'HashMap' services/conversion-service/src/` → 0 matches

  **QA Scenarios**:
  ```
  Scenario: Conversion service tests pass with SQLite
    Tool: Bash (WSL)
    Steps:
      1. wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test -p conversion-service --lib -- --test-threads=1 2>&1 | grep 'test result'"
    Expected Result: All pass
    Evidence: .sisyphus/evidence/task-5-conversion-tests.txt
  ```

  **Commit**: YES
  - Message: `feat(conversion-service): add SQLite persistence`
  - Pre-commit: `cargo test -p conversion-service --lib -- --test-threads=1`

- [x] 6. SQLite persistence for session-service

  **What to do**:
  - Add `rusqlite` dependency to session-service Cargo.toml
  - Create `SessionRepository` struct wrapping `rusqlite::Connection`
  - Create `sessions` table: `id TEXT PRIMARY KEY, user_id TEXT, access_token TEXT, refresh_token TEXT, expires_at TEXT, created_at TEXT, revoked INTEGER DEFAULT 0`
  - Migrate `Arc<Mutex<HashMap<String, Session>>>` to `Arc<Mutex<SessionRepository>>`
  - Add persistence tests with `:memory:` database

  **Must NOT do**:
  - Change endpoint signatures or token generation logic

  **Recommended Agent Profile**:
  - **Category**: `medium`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T3, T4, T5)
  - **Parallel Group**: Wave 1
  - **Blocks**: None
  - **Blocked By**: T2

  **References**:
  - `services/session-service/src/main.rs` — current implementation with 5 endpoints

  **Acceptance Criteria**:
  - [ ] `cargo test -p session-service --lib -- --test-threads=1` → all tests pass
  - [ ] `grep -r 'HashMap' services/session-service/src/` → 0 matches

  **QA Scenarios**:
  ```
  Scenario: Session service tests pass with SQLite
    Tool: Bash (WSL)
    Steps:
      1. wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test -p session-service --lib -- --test-threads=1 2>&1 | grep 'test result'"
    Expected Result: All pass
    Evidence: .sisyphus/evidence/task-6-session-tests.txt
  ```

  **Commit**: YES
  - Message: `feat(session-service): add SQLite persistence`
  - Pre-commit: `cargo test -p session-service --lib -- --test-threads=1`

- [x] 7. Wire wo-fonts into wo-renderer

  **What to do**:
  - Add `wo-fonts` as a dependency to `core/crates/wo-renderer/Cargo.toml`
  - Add `fontdb` as a dependency (from T2)
  - Create `FontLibrary` struct in wo-renderer that wraps `fontdb::Database`
  - Implement `FontLibrary::new()` that loads system fonts via `fontdb::Database::new()`
  - Implement `FontLibrary::load_font(data: &[u8])` for loading custom fonts
  - Replace the `MetricsEstimator` in `wo-renderer/src/text.rs` with real font metrics from `fontdb`
  - `TextLayoutEngine` should accept a `&FontLibrary` reference
  - Character width should come from `fontdb::Database::face()` → glyph advances, not hardcoded `0.6 * font_size`
  - Do NOT implement actual glyph rasterization yet (that's T9) — only fix the metrics
  - Update existing tests to use `FontLibrary` with a test font (embed minimal TTF or use fontdb's built-in)
  - All 125 existing wo-renderer tests must still pass

  **Must NOT do**:
  - Implement `Canvas::draw_text()` (that's T9)
  - Add swash or rustybuzz integration (that's T9)
  - Change public Canvas API (only TextLayoutEngine internals)
  - Break any existing test

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T3-T6, T8)
  - **Parallel Group**: Wave 1
  - **Blocks**: T9, T10, T11, T12
  - **Blocked By**: T2

  **References**:
  - `core/crates/wo-renderer/src/text.rs` — TextLayoutEngine, MetricsEstimator, hardcoded 0.6 char width
  - `core/crates/wo-renderer/src/lib.rs` — module structure
  - `core/crates/wo-renderer/src/canvas.rs` — Canvas struct (no draw_text method yet)
  - `core/crates/wo-fonts/src/lib.rs` — font loading, caching, CSS matching
  - `core/crates/wo-fonts/Cargo.toml` — current deps (has ttf-parser)

  **Acceptance Criteria**:
  - [ ] `cargo test -p wo-renderer --lib -- --test-threads=1` → 125+ tests pass
  - [ ] `grep -r '0\.6.*font_size\|avg_char_width' core/crates/wo-renderer/src/` → 0 matches (hardcoded metrics eliminated)
  - [ ] `grep -r 'fontdb' core/crates/wo-renderer/src/` → found in text.rs

  **QA Scenarios**:
  ```
  Scenario: Renderer tests pass with real font metrics
    Tool: Bash (WSL)
    Steps:
      1. wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test -p wo-renderer --lib -- --test-threads=1 2>&1 | grep 'test result'"
    Expected Result: 125+ tests pass
    Evidence: .sisyphus/evidence/task-7-renderer-tests.txt

  Scenario: Hardcoded metrics eliminated
    Tool: Bash
    Steps:
      1. grep -rn '0\.6.*font_size\|avg_char_width' core/crates/wo-renderer/src/
    Expected Result: 0 matches
    Evidence: .sisyphus/evidence/task-7-no-hardcoded-metrics.txt
  ```

  **Commit**: YES
  - Message: `feat(renderer): wire wo-fonts and fontdb for real font metrics`
  - Pre-commit: `cargo test -p wo-renderer --lib -- --test-threads=1`

- [x] 8. Add XPS and OFD serializers

  **What to do**:
  - Create `core/crates/wo-xps/src/serializer.rs` — XpsSerializer
  - Create `core/crates/wo-ofd/src/serializer.rs` — OfdSerializer
  - Follow the same pattern as OdfSerializer: `new()`, `serialize(&XpsDocument) -> Result<Vec<u8>>`, `ArchiveWriter`
  - Update each crate's `lib.rs` to add `pub mod serializer; pub use serializer::XpsSerializer;`
  - Add comprehensive tests (10+ per serializer): minimal document, full document, roundtrip
  - Study XPS model types in `core/crates/wo-xps/src/model.rs` before implementing
  - Study OFD model types in `core/crates/wo-ofd/src/model.rs` before implementing
  - Follow ODF serializer pattern: `core/crates/wo-odf/src/serializer.rs`

  **Must NOT do**:
  - Modify parser code
  - Change model types
  - Skip roundtrip tests

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: T18
  - **Blocked By**: None

  **References**:
  - `core/crates/wo-xps/src/model.rs` — XPS document model types
  - `core/crates/wo-xps/src/lib.rs` — current modules
  - `core/crates/wo-ofd/src/model.rs` — OFD document model types
  - `core/crates/wo-ofd/src/lib.rs` — current modules
  - `core/crates/wo-odf/src/serializer.rs` — reference serializer pattern (ArchiveWriter usage)
  - `core/crates/wo-epub/src/serializer.rs` — another reference (EPUB ZIP structure)

  **Acceptance Criteria**:
  - [ ] `cargo test -p wo-xps --lib -- --test-threads=1` → 14+ tests pass (was 14, add 10+ serializer tests)
  - [ ] `cargo test -p wo-ofd --lib -- --test-threads=1` → 10+ tests pass (was 10, add 10+ serializer tests)
  - [ ] Both crates have `pub use serializer::{XpsSerializer, OfdSerializer};` in lib.rs

  **QA Scenarios**:
  ```
  Scenario: XPS serializer roundtrip
    Tool: Bash (WSL)
    Steps:
      1. wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test -p wo-xps --lib -- --test-threads=1 2>&1 | grep 'test result'"
    Expected Result: 24+ tests pass (14 parser + 10+ serializer)
    Evidence: .sisyphus/evidence/task-8-xps-tests.txt

  Scenario: OFD serializer roundtrip
    Tool: Bash (WSL)
    Steps:
      1. wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test -p wo-ofd --lib -- --test-threads=1 2>&1 | grep 'test result'"
    Expected Result: 20+ tests pass (10 parser + 10+ serializer)
    Evidence: .sisyphus/evidence/task-8-ofd-tests.txt
  ```

  **Commit**: YES
  - Message: `feat(xps,ofd): add XPS and OFD serializers`
  - Pre-commit: `cargo test -p wo-xps -p wo-ofd --lib -- --test-threads=1`

- [x] 9. Add Canvas::draw_text() with swash glyph rasterization

  **What to do**:
  - Add `swash` dependency to wo-renderer (from T2)
  - Add `rustybuzz` dependency for text shaping (from T2)
  - Add `Canvas::draw_text(&mut self, text: &str, x: f64, y: f64, font_size: f64, font: &str, color: Color)` method
  - Internally: use `rustybuzz::shape()` to get glyph positions, then `swash::Rasterizer` to rasterize each glyph to bitmap
  - Composite glyph bitmaps onto the Canvas pixel buffer using existing alpha blending
  - Wire through `FontLibrary` (from T7) to look up fonts
  - Latin-only for v1: no kerning, no ligatures, no bidi, no complex scripts
  - Add tests: render a single character, verify pixel data changes; render "Hello", verify output non-empty
  - Keep existing `fill_rect`/`stroke_rect` working unchanged

  **Must NOT do**:
  - Implement kerning, ligatures, OpenType features, variable fonts, complex scripts, bidi
  - Remove or change existing Canvas methods
  - Break any existing test

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 2
  - **Blocks**: T10, T16, T17
  - **Blocked By**: T2, T7

  **References**:
  - `core/crates/wo-renderer/src/canvas.rs` — Canvas struct, pixel buffer, alpha blending
  - `core/crates/wo-renderer/src/text.rs` — TextLayoutEngine (updated in T7 with FontLibrary)
  - `core/crates/wo-renderer/src/color.rs` — Color type
  - `core/crates/wo-renderer/src/model.rs` — Render types
  - swash docs: glyph rasterization API — `swash::scale::ScaleContext`, `swash::scale::ScaleImage`
  - rustybuzz docs: text shaping — `rustybuzz::shape()`, `rustybuzz::GlyphBuffer`

  **Acceptance Criteria**:
  - [ ] `cargo test -p wo-renderer --lib -- --test-threads=1` → 130+ tests pass (125 existing + 5+ new)
  - [ ] `grep -r 'draw_text' core/crates/wo-renderer/src/canvas.rs` → found
  - [ ] `grep -r 'swash' core/crates/wo-renderer/src/` → found in canvas.rs or text.rs

  **QA Scenarios**:
  ```
  Scenario: draw_text renders visible pixels
    Tool: Bash (WSL)
    Steps:
      1. wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test -p wo-renderer --lib draw_text -- --test-threads=1 2>&1 | grep 'test result'"
    Expected Result: New draw_text tests pass
    Evidence: .sisyphus/evidence/task-9-draw-text.txt

  Scenario: All existing renderer tests still pass
    Tool: Bash (WSL)
    Steps:
      1. wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test -p wo-renderer --lib -- --test-threads=1 2>&1 | grep 'test result'"
    Expected Result: 130+ tests pass
    Evidence: .sisyphus/evidence/task-9-renderer-regression.txt
  ```

  **Commit**: YES
  - Message: `feat(renderer): add Canvas::draw_text() with swash glyph rasterization`
  - Pre-commit: `cargo test -p wo-renderer --lib -- --test-threads=1`

- [x] 10. Replace rectangle glyph hacks in wo-docx-renderer and wo-renderer-wasm

  **What to do**:
  - In `core/crates/wo-docx-renderer/src/pipeline.rs`: replace `render_line()` (lines 219-233) and `render_table_cell()` (lines 276-289) rectangle drawing with `Canvas::draw_text()` calls
  - In `core/crates/wo-renderer-wasm/src/canvas_bridge.rs`: replace `render_text()` (lines 145-148) rectangle drawing with real text rendering
  - Both call sites currently use `font_size * 0.5` width rectangles per character — replace with actual glyph rendering
  - Wire `FontLibrary` initialization into both crates
  - Update tests to verify real text output (not just non-empty)
  - Ensure wo-renderer-wasm still compiles to wasm32-unknown-unknown

  **Must NOT do**:
  - Change any Canvas API (use T9's draw_text as-is)
  - Add new rendering features beyond what T9 provides
  - Break WASM compilation

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 2
  - **Blocks**: T16, T17
  - **Blocked By**: T9

  **References**:
  - `core/crates/wo-docx-renderer/src/pipeline.rs:219-233` — render_line() with rectangle hack
  - `core/crates/wo-docx-renderer/src/pipeline.rs:276-289` — render_table_cell() with rectangle hack
  - `core/crates/wo-renderer-wasm/src/canvas_bridge.rs:145-148` — render_text() with rectangle hack
  - `core/crates/wo-renderer/src/canvas.rs` — draw_text() method (from T9)

  **Acceptance Criteria**:
  - [ ] `grep -rn 'fill_rect.*char\|font_size.*0\.\(5\|6\).*width' core/crates/wo-docx-renderer/ core/crates/wo-renderer-wasm/` → 0 matches
  - [ ] `cargo test -p wo-docx-renderer --lib -- --test-threads=1` → 13+ tests pass
  - [ ] `cargo build --target wasm32-unknown-unknown -p wo-renderer-wasm` → succeeds

  **QA Scenarios**:
  ```
  Scenario: Rectangle hacks eliminated
    Tool: Bash
    Steps:
      1. grep -rn 'fill_rect.*char\|font_size.*0\.\(5\|6\).*width' core/crates/wo-docx-renderer/ core/crates/wo-renderer-wasm/
    Expected Result: 0 matches
    Evidence: .sisyphus/evidence/task-10-no-rectangles.txt

  Scenario: WASM still compiles
    Tool: Bash (WSL)
    Steps:
      1. wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo build --target wasm32-unknown-unknown -p wo-renderer-wasm 2>&1 | tail -3"
    Expected Result: "Finished" in output
    Evidence: .sisyphus/evidence/task-10-wasm-build.txt
  ```

  **Commit**: YES
  - Message: `refactor: replace rectangle glyph hacks with real font rendering`
  - Pre-commit: `cargo test -p wo-docx-renderer -p wo-renderer --lib -- --test-threads=1`

- [x] 11. Add reverse converters batch 1 (DOCX↔ODT, RTF→DOCX)

  **What to do**:
  - Add 3 new converters to `core/crates/wo-x2t/src/converters.rs`:
    - `DocxToOdtConverter`: Parse DOCX (OoxmlParser) → convert to OdfDocument model → serialize (OdfSerializer)
    - `OdtToDocxConverter`: Parse ODF (OdfParser) → convert to OoxmlDocument model → serialize (OoxmlSerializer)
    - `RtfToDocxConverter`: Parse RTF (RtfParser) → convert to OoxmlDocument model → serialize (OoxmlSerializer)
  - Register in `core/crates/wo-x2t/src/router.rs` (count 27→30)
  - Add roundtrip tests for each (parse source → convert → parse target → assert content)
  - The model conversion (e.g., OoxmlDocument → OdfDocument) is the hard part — map paragraphs, text runs, basic formatting
  - For v1: preserve text content, basic bold/italic/underline. Skip: tables, images, headers/footers, styles.

  **Must NOT do**:
  - Attempt full fidelity conversion (complex formatting, embedded objects)
  - Modify existing converters

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T12, T13, T14, T15)
  - **Parallel Group**: Wave 2
  - **Blocks**: None
  - **Blocked By**: T7

  **References**:
  - `core/crates/wo-x2t/src/converters.rs` — existing 27 converter implementations
  - `core/crates/wo-x2t/src/router.rs` — converter registration (count at line ~83)
  - `core/crates/wo-ooxml/src/model.rs` — OoxmlDocument type (source/target for DOCX converters)
  - `core/crates/wo-odf/src/model.rs` — OdfDocument type (source/target for ODT converters)
  - `core/crates/wo-rtf/src/model.rs` — RtfDocument type (source for RTF converter)
  - `core/crates/wo-ooxml/src/serializer.rs` — OoxmlSerializer for DOCX output
  - `core/crates/wo-odf/src/serializer.rs` — OdfSerializer for ODT output

  **Acceptance Criteria**:
  - [ ] `cargo test -p wo-x2t --lib -- --test-threads=1` → 130+ tests pass (was 120, add 10+)
  - [ ] Router count updated to 30
  - [ ] Roundtrip tests for all 3 converters

  **QA Scenarios**:
  ```
  Scenario: All x2t tests pass with new converters
    Tool: Bash (WSL)
    Steps:
      1. wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test -p wo-x2t --lib -- --test-threads=1 2>&1 | grep 'test result'"
    Expected Result: 130+ tests pass
    Evidence: .sisyphus/evidence/task-11-x2t-tests.txt
  ```

  **Commit**: YES
  - Message: `feat(x2t): add DOCX↔ODT and RTF→DOCX converters`
  - Pre-commit: `cargo test -p wo-x2t --lib -- --test-threads=1`

- [x] 12. Add reverse converters batch 2 (EPUB→DOCX, FB2→DOCX, DOCX→EPUB)

  **What to do**:
  - Add 3 new converters to `core/crates/wo-x2t/src/converters.rs`:
    - `EpubToDocxConverter`: Parse EPUB → extract chapters/text → build OoxmlDocument → serialize DOCX
    - `Fb2ToDocxConverter`: Parse FB2 → extract body/sections → build OoxmlDocument → serialize DOCX
    - `DocxToEpubConverter`: Parse DOCX → extract text → build EpubDocument with chapters → serialize EPUB
  - Register in router.rs (count 30→33)
  - Add roundtrip tests
  - Chapter splitting: use document structure (headings) or fixed chapter size as fallback

  **Must NOT do**:
  - Attempt full fidelity (images, styles, metadata beyond basic)
  - Modify existing converters

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T11, T13, T14, T15)
  - **Parallel Group**: Wave 2
  - **Blocks**: None
  - **Blocked By**: T7

  **References**:
  - `core/crates/wo-epub/src/model.rs` — EpubDocument type
  - `core/crates/wo-epub/src/serializer.rs` — EpubSerializer
  - `core/crates/wo-fb2/src/model.rs` — Fb2Document type
  - `core/crates/wo-fb2/src/serializer.rs` — Fb2Serializer
  - `core/crates/wo-x2t/src/converters.rs` — existing converter patterns

  **Acceptance Criteria**:
  - [ ] `cargo test -p wo-x2t --lib -- --test-threads=1` → 140+ tests pass
  - [ ] Router count updated to 33
  - [ ] Roundtrip tests for all 3 converters

  **QA Scenarios**:
  ```
  Scenario: EPUB→DOCX roundtrip preserves text
    Tool: Bash (WSL)
    Steps:
      1. wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test -p wo-x2t --lib epub_to_docx -- --test-threads=1 2>&1"
    Expected Result: Tests pass
    Evidence: .sisyphus/evidence/task-12-epub-docx.txt
  ```

  **Commit**: YES
  - Message: `feat(x2t): add EPUB↔DOCX and FB2→DOCX converters`
  - Pre-commit: `cargo test -p wo-x2t --lib -- --test-threads=1`

- [x] 13. Add HWP and DjVu serializers

  **What to do**:
  - Create `core/crates/wo-hwp/src/serializer.rs` — HwpSerializer
  - Create `core/crates/wo-djvu/src/serializer.rs` — DjVuSerializer
  - Follow same pattern as other serializers
  - Study model types before implementing
  - Add 10+ tests per serializer
  - Update lib.rs files

  **Must NOT do**:
  - Modify parser code or model types

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2
  - **Blocks**: T18
  - **Blocked By**: None

  **References**:
  - `core/crates/wo-hwp/src/model.rs` — HWP document model
  - `core/crates/wo-hwp/src/lib.rs` — current modules
  - `core/crates/wo-djvu/src/model.rs` — DjVu document model
  - `core/crates/wo-djvu/src/lib.rs` — current modules
  - `core/crates/wo-odf/src/serializer.rs` — reference pattern

  **Acceptance Criteria**:
  - [ ] `cargo test -p wo-hwp --lib -- --test-threads=1` → 22+ tests (was 12, add 10+)
  - [ ] `cargo test -p wo-djvu --lib -- --test-threads=1` → 18+ tests (was 8, add 10+)

  **Commit**: YES
  - Message: `feat(hwp,djvu): add HWP and DjVu serializers`
  - Pre-commit: `cargo test -p wo-hwp -p wo-djvu --lib -- --test-threads=1`

- [x] 14. SQLite persistence for coauthoring-service

  **What to do**:
  - Add `rusqlite` dependency to coauthoring-service Cargo.toml
  - Create `SessionRepository` struct
  - Create `editor_sessions` table: `id TEXT PRIMARY KEY, document_id TEXT, created_at TEXT, max_participants INTEGER`
  - Create `session_participants` table: `session_id TEXT, user_id TEXT, joined_at TEXT`
  - Persist session metadata in SQLite; broadcast channels remain ephemeral (runtime-only)
  - Add persistence tests

  **Must NOT do**:
  - Persist broadcast channel state (impossible — runtime-only)
  - Change WebSocket handling logic

  **Recommended Agent Profile**:
  - **Category**: `medium`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T11-T13)
  - **Parallel Group**: Wave 2
  - **Blocks**: T15
  - **Blocked By**: T2

  **References**:
  - `services/coauthoring-service/src/main.rs` — current implementation, EditOperation struct at line 46

  **Acceptance Criteria**:
  - [ ] `cargo test -p coauthoring-service --lib -- --test-threads=1` → all tests pass
  - [ ] `grep -r 'HashMap' services/coauthoring-service/src/` → 0 matches

  **Commit**: YES
  - Message: `feat(coauthoring): add SQLite session persistence`
  - Pre-commit: `cargo test -p coauthoring-service --lib -- --test-threads=1`

- [x] 15. Wire diamond-types OT into coauthoring-service

  **What to do**:
  - Add `diamond-types` dependency (from T2)
  - Import `diamond_types::list::ListCRDT`
  - Replace the raw broadcast at the TODO (line 285) with OT-validated operations
  - Each `EditorSession` gets a `ListCRDT` instance
  - On incoming edit: apply to local ListCRDT, broadcast transformed operation
  - The existing `EditOperation` struct (currently `#[allow(dead_code)]`) becomes the message format
  - Plain text only: "insert" and "delete" op_types
  - Add concurrent edit test: two simulated clients insert at same position, both preserved
  - Add tests for: single client insert, single client delete, concurrent inserts, concurrent delete+insert

  **Must NOT do**:
  - Implement rich text OT (no formatting operations in v1)
  - Change WebSocket protocol format (keep existing message structure)

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on T14's repository pattern)
  - **Parallel Group**: Wave 2
  - **Blocks**: None
  - **Blocked By**: T14

  **References**:
  - `services/coauthoring-service/src/main.rs:46-58` — EditOperation struct
  - `services/coauthoring-service/src/main.rs:285` — TODO insertion point
  - diamond-types docs: `diamond_types::list::ListCRDT` API

  **Acceptance Criteria**:
  - [ ] `cargo test -p coauthoring-service --lib -- --test-threads=1` → all tests pass
  - [ ] Concurrent edit test passes (both insertions preserved)
  - [ ] `grep -r 'ListCRDT\|diamond_types' services/coauthoring-service/src/` → found

  **QA Scenarios**:
  ```
  Scenario: Concurrent edits preserved
    Tool: Bash (WSL)
    Steps:
      1. wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test -p coauthoring-service --lib concurrent -- --test-threads=1 2>&1 | grep 'test result'"
    Expected Result: Concurrent edit tests pass
    Evidence: .sisyphus/evidence/task-15-concurrent-ot.txt
  ```

  **Commit**: YES
  - Message: `feat(coauthoring): integrate diamond-types OT engine`
  - Pre-commit: `cargo test -p coauthoring-service --lib -- --test-threads=1`

- [x] 16. Real PDF output via pdf-writer crate

  **What to do**:
  - Add `pdf-writer` dependency to `core/crates/wo-docx-renderer/Cargo.toml` (from T2)
  - Replace the fake PDF output in `pipeline.rs:73-76` (raw RGBA concatenation) with real PDF generation
  - Use `pdf-writer` to create: page objects, content streams with text operators, font resources
  - For each rendered page: create a PDF page with the canvas content embedded
  - At minimum: text content as PDF text operators (Tj, TJ). Image content as embedded XObject if time permits.
  - Update the roundtrip test to assert output starts with `%PDF-`
  - pdf-writer is low-level: you write PDF objects manually. Use `PdfWriter::new()`, `doc.pages()`, `doc.page()`, etc.

  **Must NOT do**:
  - Use any other PDF library (no printpdf, no genpdf)
  - Touch wo-pdf crate (known ICE)
  - Attempt complex PDF features (forms, annotations, bookmarks)

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 3
  - **Blocks**: None
  - **Blocked By**: T9, T10

  **References**:
  - `core/crates/wo-docx-renderer/src/pipeline.rs:73-76` — fake PDF output (concatenates RGBA bytes)
  - `core/crates/wo-docx-renderer/src/pipeline.rs` — RenderOutput::Pdf variant
  - `core/crates/wo-docx-renderer/src/roundtrip.rs:136` — test that asserts `!output.is_empty()`
  - pdf-writer crate docs: `pdf_writer::PdfWriter`, page/content stream API

  **Acceptance Criteria**:
  - [ ] `cargo test -p wo-docx-renderer --lib -- --test-threads=1` → 13+ tests pass
  - [ ] PDF output starts with `%PDF-` header
  - [ ] `grep -r 'flatten\|collect.*page_bytes' core/crates/wo-docx-renderer/src/pipeline.rs` → 0 matches (fake code eliminated)

  **QA Scenarios**:
  ```
  Scenario: PDF output is valid
    Tool: Bash (WSL)
    Steps:
      1. wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test -p wo-docx-renderer --lib pdf -- --test-threads=1 2>&1 | grep 'test result'"
    Expected Result: PDF tests pass
    Evidence: .sisyphus/evidence/task-16-pdf-output.txt

  Scenario: PDF starts with valid header
    Tool: Bash
    Steps:
      1. grep -n '%PDF' core/crates/wo-docx-renderer/src/pipeline.rs
    Expected Result: Found in PDF generation code
    Evidence: .sisyphus/evidence/task-16-pdf-header.txt
  ```

  **Commit**: YES
  - Message: `feat(docx-renderer): real PDF output via pdf-writer`
  - Pre-commit: `cargo test -p wo-docx-renderer --lib -- --test-threads=1`

- [x] 17. Fix wo-renderer-wasm render_page() stub

  **What to do**:
  - In `core/crates/wo-renderer-wasm/src/lib.rs`, replace the placeholder `render_page()` function
  - Current stub (TODO comment): draws white background + border + format label + test rectangles
  - New implementation:
    1. Parse input bytes based on format parameter (use wo-ooxml for DOCX, wo-odf for ODT, etc.)
    2. Create Canvas, render parsed document using real draw_text() (from T9/T10)
    3. Return rendered pixel data for the requested page
  - For v1: support DOCX format only. Other formats return error "unsupported format".
  - Ensure WASM compilation succeeds

  **Must NOT do**:
  - Support all formats (DOCX only for v1)
  - Change the JS API signature (keep existing `render_page(doc_bytes, format, width, height)`)

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T16, T18)
  - **Parallel Group**: Wave 3
  - **Blocks**: T19, T20
  - **Blocked By**: T9, T10

  **References**:
  - `core/crates/wo-renderer-wasm/src/lib.rs` — render_page() stub with TODO comment
  - `core/crates/wo-renderer-wasm/src/canvas_bridge.rs` — Canvas bridge (updated in T10)
  - `core/crates/wo-renderer/src/canvas.rs` — Canvas API (with draw_text from T9)
  - `core/crates/wo-docx-renderer/src/pipeline.rs` — DOCX rendering pipeline (reference for parsing + rendering flow)

  **Acceptance Criteria**:
  - [ ] `cargo build --target wasm32-unknown-unknown -p wo-renderer-wasm` → succeeds
  - [ ] `grep -r 'TODO.*render_page\|placeholder' core/crates/wo-renderer-wasm/src/` → 0 matches
  - [ ] 15+ WASM tests pass

  **QA Scenarios**:
  ```
  Scenario: WASM build succeeds
    Tool: Bash (WSL)
    Steps:
      1. wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo build --target wasm32-unknown-unknown -p wo-renderer-wasm 2>&1 | tail -3"
    Expected Result: "Finished" in output
    Evidence: .sisyphus/evidence/task-17-wasm-build.txt

  Scenario: No TODO/placeholder remains
    Tool: Bash
    Steps:
      1. grep -rn 'TODO.*render_page\|placeholder' core/crates/wo-renderer-wasm/src/
    Expected Result: 0 matches
    Evidence: .sisyphus/evidence/task-17-no-stubs.txt
  ```

  **Commit**: YES
  - Message: `feat(renderer-wasm): implement render_page() with real DOCX rendering`
  - Pre-commit: `cargo build --target wasm32-unknown-unknown -p wo-renderer-wasm`

- [x] 18. Add converters for new niche format serializers

  **What to do**:
  - Add converters that use the new serializers from T8 and T13:
    - `XpsToDocxConverter`: XPS → DOCX (parse XPS, convert to OoxmlDocument, serialize)
    - `OfdToDocxConverter`: OFD → DOCX
    - `HwpToDocxConverter`: HWP → DOCX (if model allows conversion)
    - `DjvuToDocxConverter`: DjVu → DOCX (if model allows conversion)
    - `DocxToXpsConverter`: DOCX → XPS
  - Register in router.rs (count 33→38)
  - Add roundtrip tests
  - Only add converters where the source model has enough structure to build a target model

  **Must NOT do**:
  - Add converters that require information not in the source model
  - Modify existing converters

  **Recommended Agent Profile**:
  - **Category**: `medium`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T16, T17)
  - **Parallel Group**: Wave 3
  - **Blocks**: None
  - **Blocked By**: T8, T13

  **References**:
  - `core/crates/wo-xps/src/serializer.rs` — XpsSerializer (from T8)
  - `core/crates/wo-ofd/src/serializer.rs` — OfdSerializer (from T8)
  - `core/crates/wo-hwp/src/serializer.rs` — HwpSerializer (from T13)
  - `core/crates/wo-djvu/src/serializer.rs` — DjVuSerializer (from T13)
  - `core/crates/wo-x2t/src/converters.rs` — existing converter patterns

  **Acceptance Criteria**:
  - [ ] `cargo test -p wo-x2t --lib -- --test-threads=1` → 150+ tests pass
  - [ ] Router count updated to 38

  **Commit**: YES
  - Message: `feat(x2t): add converters for XPS, OFD, HWP, DjVu formats`
  - Pre-commit: `cargo test -p wo-x2t --lib -- --test-threads=1`

- [x] 19. Wire documenteditor-react Viewport to wo-renderer-wasm

  **What to do**:
  - In `apps/web/apps/documenteditor-react/src/components/Viewport.tsx`: replace placeholder with real canvas rendering
  - Use the `sdk-bridge` package (`packages/sdk-bridge/`) to call `render_page()` from wo-renderer-wasm
  - Load a sample DOCX document (base64 embedded or fetched from conversion-service)
  - Render to HTML5 Canvas element using the pixel data from render_page()
  - Add basic zoom (scale canvas) and page navigation (prev/next)
  - Ensure `pnpm build` succeeds for documenteditor-react

  **Must NOT do**:
  - Add text editing capabilities (view-only for v1)
  - Modify the sdk-bridge package API
  - Touch other React apps

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T20)
  - **Parallel Group**: Wave 4
  - **Blocks**: None
  - **Blocked By**: T17

  **References**:
  - `apps/web/apps/documenteditor-react/src/components/Viewport.tsx` — current placeholder
  - `apps/web/apps/documenteditor-react/src/hooks/` — existing hooks
  - `apps/web/apps/documenteditor-react/src/stores/` — MobX stores
  - `apps/web/apps/documenteditor-react/vite.config.ts` — build config
  - `apps/web/apps/documenteditor-react/package.json` — dependencies

  **Acceptance Criteria**:
  - [ ] `pnpm --filter documenteditor-react build` → succeeds
  - [ ] Viewport.tsx calls render_page() from sdk-bridge
  - [ ] No placeholder/TODO comments in Viewport.tsx

  **QA Scenarios**:
  ```
  Scenario: documenteditor-react builds successfully
    Tool: Bash
    Steps:
      1. pnpm --filter documenteditor-react build
    Expected Result: Exit 0
    Evidence: .sisyphus/evidence/task-19-react-build.txt

  Scenario: Viewport uses sdk-bridge
    Tool: Bash
    Steps:
      1. grep -r 'render_page\|sdk-bridge\|init()' apps/web/apps/documenteditor-react/src/components/Viewport.tsx
    Expected Result: Found references to WASM bridge
    Evidence: .sisyphus/evidence/task-19-viewport-wired.txt
  ```

  **Commit**: YES
  - Message: `feat(web): wire documenteditor-react Viewport to wo-renderer-wasm`
  - Pre-commit: `pnpm --filter documenteditor-react build`

- [x] 20. Advance remaining React editors

  **What to do**:
  - For each scaffold app (spreadsheet, presentation, pdf, visio, editor-shell):
    - Wire Viewport component to wo-renderer-wasm (same pattern as T19)
    - Ensure `pnpm build` succeeds for each app
    - Add basic document loading (fetch from conversion-service)
    - Keep it view-only (no editing)
  - For editor-shell: wire the main Canvas placeholder to render_page()
  - Each app should at minimum: build without errors, render a test document on the canvas

  **Must NOT do**:
  - Add editing capabilities
  - Create app-specific features (formulas for spreadsheet, slides for presentation)
  - Modify the design-system or editor-common packages

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T19)
  - **Parallel Group**: Wave 4
  - **Blocks**: None
  - **Blocked By**: T17

  **References**:
  - `apps/web/apps/spreadsheeteditor-react/src/components/Viewport.tsx` — placeholder
  - `apps/web/apps/presentationeditor-react/src/components/Viewport.tsx` — placeholder
  - `apps/web/apps/pdfeditor-react/src/components/Viewport.tsx` — placeholder
  - `apps/web/apps/visioeditor-react/src/components/Viewport.tsx` — placeholder
  - `apps/web/apps/editor-shell/src/` — shell with Canvas placeholder
  - `packages/sdk-bridge/` — WASM bridge (same as T19)

  **Acceptance Criteria**:
  - [ ] `pnpm build` succeeds for all 5 apps
  - [ ] Each app's Viewport calls render_page()
  - [ ] No build errors or type errors

  **QA Scenarios**:
  ```
  Scenario: All React apps build
    Tool: Bash
    Steps:
      1. pnpm build (from apps/web/ directory or workspace root)
    Expected Result: All apps build successfully
    Evidence: .sisyphus/evidence/task-20-all-build.txt
  ```

  **Commit**: YES
  - Message: `feat(web): wire remaining React editors to wo-renderer-wasm`
  - Pre-commit: `pnpm build`

---

## Final Verification Wave

> 4 review agents run in PARALLEL. ALL must APPROVE.

- [x] F1. **Plan Compliance Audit** — `oracle` (CONDITIONAL APPROVE — clippy 12+ warnings, tests 898/900+) → **RESOLVED: 0 clippy warnings, 902 tests**
  Read the plan end-to-end. For each "Must Have": verify implementation exists. For each "Must NOT Have": search codebase for forbidden patterns. Check evidence files exist. Compare deliverables against plan.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [x] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo clippy --workspace`, `cargo test --workspace --lib`, review all changed files for anti-patterns. Check AI slop patterns.
  Output: `Build [PASS/FAIL] | Lint [PASS/FAIL] | Tests [N pass/N fail] | Files [N clean/N issues] | VERDICT`

- [x] F3. **Real QA** — `unspecified-high`
  Execute every QA scenario from every task. Test cross-task integration. Test edge cases. Save evidence to `.sisyphus/evidence/final-qa/`.
  Output: `Scenarios [N/N pass] | Integration [N/N] | Edge Cases [N tested] | VERDICT`

- [x] F4. **Scope Fidelity Check** — `deep`
  For each task: read spec, read actual diff. Verify 1:1 — everything built, nothing beyond spec. Check contamination. Flag unaccounted changes.
  Output: `Tasks [N/N compliant] | Contamination [CLEAN/N issues] | Unaccounted [CLEAN/N files] | VERDICT`

---

## Commit Strategy

- **T1**: `chore: fix clippy warnings across workspace`
- **T2**: `chore: validate and add new dependencies`
- **T3-T6**: `feat(storage-service): add SQLite persistence` (one per service)
- **T7**: `feat(renderer): wire wo-fonts for real font metrics`
- **T8**: `feat(xps,ofd): add XPS and OFD serializers`
- **T9**: `feat(renderer): add Canvas::draw_text() with swash`
- **T10**: `refactor: replace rectangle glyph hacks with real rendering`
- **T11-T12**: `feat(x2t): add cross-format converters (DOCX↔ODT, RTF→DOCX, ...)`
- **T13**: `feat(hwp,djvu): add HWP and DjVu serializers`
- **T14**: `feat(coauthoring): add SQLite session persistence`
- **T15**: `feat(coauthoring): integrate diamond-types OT engine`
- **T16**: `feat(docx-renderer): real PDF output via pdf-writer`
- **T17**: `feat(renderer-wasm): implement render_page() with real doc parsing`
- **T18**: `feat(x2t): add converters for new niche format serializers`
- **T19-T20**: `feat(web): wire React editors to wo-renderer-wasm`

---

## Success Criteria

### Verification Commands
```bash
# Clippy clean (excluding wo-pdf)
wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo clippy --workspace 2>&1 | grep -E 'warning.*generated' | grep -v wo-pdf"
# Expected: 0 warnings from target crates

# All tests pass
wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test --workspace --lib -- --test-threads=1 2>&1 | grep 'test result'"
# Expected: all ok, 900+ tests total

# No rectangle glyph hacks remain
grep -r 'fill_rect.*char\|font_size.*0\.\(5\|6\).*width' core/crates/wo-docx-renderer/ core/crates/wo-renderer-wasm/
# Expected: 0 matches

# PDF output is valid
wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test -p wo-docx-renderer --lib -- --test-threads=1"
# Expected: tests pass, PDF output starts with %PDF-

# WASM compilation
wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo build --target wasm32-unknown-unknown -p wo-renderer-wasm"
# Expected: exit 0

# Services persist data
wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test -p storage-service -p identity-service -p conversion-service -p session-service -p coauthoring-service --lib -- --test-threads=1"
# Expected: all pass, :memory: DB tests verify persistence

# Concurrent OT test
wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test -p coauthoring-service --lib -- --test-threads=1"
# Expected: concurrent edit test passes
```

### Final Checklist
- [x] All "Must Have" present
- [x] All "Must NOT Have" absent
- [x] All 900+ tests pass
- [x] Rectangle glyph hacks eliminated
- [x] PDF output is valid PDF
- [x] WASM builds successfully
- [x] 5 services persist data via SQLite
- [x] OT/CRDT preserves concurrent edits
- [x] 33+ format converters registered
- [x] 4 niche format serializers created
