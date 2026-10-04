# World-Office Full Rewrite: Modernization Plan v3

**Created:** 2026-04-06  
**Updated:** 2026-04-06 (v3: Oracle-reviewed, enterprise plan added)  
**Brand:** World-Office (successor to Word Office, fork of Word Office)  
**License:** Dual — AGPL-3.0 (Community) + Commercial (Enterprise)  
**Status:** Revised — Oracle-verified, enterprise architecture added  
**Scope:** 22 repos, ~100k files, 5 languages → Rust + TypeScript monorepo  
**Timeline:** ~180 weeks (3.5 years), 6-8 person team

---

## 1. Current State Audit

### 1.1 Repository Inventory

| Tier | Repo | Language | Files | Tech Stack | Health |
|------|------|----------|-------|------------|--------|
| **Engine** | `core/` | C++17 | 10,385 | CMake, boost, freetype, harfbuzz, icu, hunspell, openssl, v8 | Stable, no TS |
| **JS SDK** | `sdkjs/` | JavaScript | — | (submodule, empty locally) | Unknown |
| **Web UI** | `web-apps/` | JS/HTML/CSS | 28,066 | Grunt, RequireJS, Backbone, jQuery, Underscore, LESS | **Critical** |
| **Server** | `server/` | Node.js | 25 | Express 4, CommonJS, `co`, socket.io | **Critical** |
| **Admin** | `server/AdminPanel/` | React+Node | — | React 18, Redux Toolkit, Webpack, SCSS | Healthy |
| **Desktop SDK** | `desktop-sdk/` | C++ | 6,605 | CEF, Qt 5.15, CMake | Concerns (CEF) |
| **Desktop Apps** | `desktop-apps/` | C++/JS | ~2,900 | CMake, QMake | Stable |
| **Desktop Main** | `DesktopEditors/` | C++ | — | CMake (CI config only) | Shell |
| **Assembly** | `DocumentServer/` | Shell | — | Makefiles, Nginx, systemd | Stable |
| **CI** | `docker-ci/` | Docker | — | Ubuntu 24.04, Node 20, JDK 21 | Healthy |
| **Packaging** | `document-server-package/` | Shell | — | Inno Setup, deb/rpm | Stable |
| **Nextcloud** | `word-office-nextcloud/` | PHP+Vue3 | 46 PHP | Vue 3, Vite, Nextcloud SDK | Healthy |
| **OpenCloud** | `word-office-opencloud/` | Node.js | — | EJS, Express | WIP |
| **Android** | `documents-app-android/` | Kotlin | — | Framework7, React | WIP |
| **Integration** | `document-server-integration/` | Multi | — | Go, Python, PHP, Java, C#, Node, Ruby | Reference |
| **Assets** | fonts, dictionaries, artwork, templates, plugins, forms, formats | Various | — | Static assets | Stable |
| **Org** | `.github/` | Markdown | — | — | Healthy |

### 1.2 Core Engine Module Breakdown (10,385 C++ files)

| Module | Files | Purpose | Rewrite Difficulty |
|--------|-------|---------|-------------------|
| OOXML/ | 3,387 | DOCX/XLSX/PPTX parsing (modern Office formats) | **High** — heavy OOP, virtual dispatch, multiple inheritance |
| DesktopEditor/ | 3,152 | Canvas rendering, fonts, raster, V8/JS bridges | **Critical** — GPU code, V8 integration, platform-specific |
| MsBinaryFile/ | 2,954 | Legacy .doc/.xls/.ppt binary format parsers | **High** — complex binary formats, little documentation |
| Common/ | 1,030 | Shared utils, network, 29 third-party deps | **Medium** — mostly wrappers around ICU, OpenSSL, etc. |
| UnicodeConverter/ | 998 | ICU-based text encoding conversion | **Low** — mostly ICU C API wrappers |
| OdfFile/ | 841 | ODF (OpenDocument) XML format support | **Medium** — XML parsing, well-documented spec |
| PdfFile/ | 301 | PDF reading/writing (based on xpdf) | **High** — complex spec, embedded in xpdf codebase |
| RtfFile/ | 168 | Rich Text Format parser | **Medium** — well-documented format |
| HwpFile/ | 165 | Korean HWP word processor format | **Medium** — obscure but self-contained |
| OfficeUtils/ | 90 | ZIP/archive manipulation | **Low** — thin wrapper |
| DjVuFile/ | 117 | DjVu image/document format | **Low** — self-contained, well-known library |
| EpubFile/ | 86 | EPUB ebook format | **Low** — ZIP + XHTML, straightforward |
| OFDFile/ | 58 | Chinese OFD document format | **Low** — self-contained |
| DocxRenderer/ | 46 | DOCX → PDF rendering pipeline | **Medium** — connects OOXML + PDF |
| TxtFile/ | 16 | Plain text import/export | **Trivial** |
| HtmlFile2/ | 22 | HTML import/export | **Low** |
| X2tConverter/ | 38 | Format conversion orchestrator | **Medium** — routing logic between format modules |
| Fb2File/ | 7 | FictionBook 2.0 format | **Trivial** |
| XpsFile/ | 19 | XPS document format | **Low** |
| Other | ~100 | Test apps, build configs, misc | — |

### 1.3 C++ Complexity Profile

| Feature | Prevalence | Rust Impact |
|---------|-----------|-------------|
| **C++17 standard** | Confirmed (common.cmake) | Modern, well-understood |
| **Multiple inheritance** | 31+ classes (PPTX, raster) | Must redesign with traits/composition |
| **Virtual dispatch** | 263+ methods (DesktopEditor alone) | Use trait objects, acceptable perf |
| **Templates** | 29+ usages (Graphics, V8/JSC) | Use Rust generics, may need macros for complex cases |
| **Exceptions** | Minimal (12 try blocks) | Rust error handling maps naturally |
| **Raw pointers** | Extensive (C++ codebase norm) | Manual conversion to references/slices/lifetimes |
| **WASM build** | Already exists (Emscripten) | Rust compiles to WASM natively (advantage) |
| **Third-party deps** | 29 (boost, freetype, harfbuzz, icu, openssl, v8, etc.) | Most have Rust equivalents or FFI bindings |

### 1.4 Technical Debt Inventory

#### CRITICAL

| # | Problem | Location | Impact |
|---|---------|----------|--------|
| C1 | AMD modules (RequireJS), no tree-shaking | `web-apps/` | 28k files, massive bundle |
| C2 | Backbone+jQuery DOM manipulation | `web-apps/` | No component model, untestable |
| C3 | Grunt build system | `web-apps/` | Dead ecosystem, no HMR |
| C4 | CommonJS + `co` generators | `server/` | Callback hell, no type safety |
| C5 | No TypeScript (main codebase) | `web-apps/`, `server/` | Zero type safety |
| C6 | No design system | `web-apps/` | 7 apps, inconsistent UI |
| C7 | Monolithic server | `server/` | Cannot scale independently |
| C8 | CEF binary in desktop | `desktop-sdk/` | 200MB+, licensing friction |
| C9 | C++ memory safety | `core/` | Buffer overflows, use-after-free potential |

---

## 2. Rewrite Strategy

### 2.1 Guiding Principles

1. **Rust everywhere** — Core engine, desktop shell, microservices tooling. Memory safety is non-negotiable.
2. **Incremental migration, never Big Bang** — Ship continuously. Old and new coexist.
3. **100% format parity before any format module is "done"** — Every format test corpus must pass.
4. **TypeScript for all JavaScript** — Zero new JS without types.
5. **Monorepo** — One workspace, shared configs, single source of truth.
6. **Microservices where domain boundaries justify it** — Conversion, co-authoring, storage, auth are separate services.
7. **Design system from Day 1** — Build the component library before the apps.
8. **Test first** — Format roundtrip tests before any Rust rewrite begins.

### 2.2 Target Architecture

```
world-office/                             (monorepo)
├── turbo.json                           # Turborepo orchestration
├── pnpm-workspace.yaml                  # JS workspace
├── Cargo.toml                           # Rust workspace root
├── package.json                         # Root scripts
├── LICENSE-AGPL                         # Community license
├── LICENSE-COMMERCIAL                   # Enterprise license (customer-facing)
├── LICENSE.md                           # Dual license explainer
│
├── core/                                # Rust document engine (AGPL)
│   ├── Cargo.toml                       # Workspace for all core crates
│   ├── crates/
│   │   ├── wo-common/                   # Shared types, errors, utils
│   │   ├── wo-unicode/                  # ICU-based text conversion
│   │   ├── wo-txt/                      # Plain text format
│   │   ├── wo-fb2/                      # FictionBook 2.0
│   │   ├── wo-html/                     # HTML import/export
│   │   ├── wo-xps/                      # XPS format
│   │   ├── wo-ofd/                      # Chinese OFD
│   │   ├── wo-djvu/                     # DjVu format
│   │   ├── wo-epub/                     # EPUB format
│   │   ├── wo-office-utils/             # ZIP/archive manipulation
│   │   ├── wo-rtf/                      # RTF format
│   │   ├── wo-hwp/                      # Korean HWP
│   │   ├── wo-pdf/                      # PDF read/write
│   │   ├── wo-odf/                      # ODF (ODT/ODS/ODP)
│   │   ├── wo-ooxml/                    # OOXML (DOCX/XLSX/PPTX)
│   │   ├── wo-msbinary/                 # Legacy .doc/.xls/.ppt
│   │   ├── wo-docx-renderer/            # DOCX → PDF rendering
│   │   ├── wo-x2t/                      # Format conversion orchestrator
│   │   ├── wo-renderer/                 # Canvas rendering engine
│   │   ├── wo-fonts/                    # Font engine
│   │   └── wo-raster/                   # Raster image processing
│   └── tests/                           # Format roundtrip test corpus
│
├── core-enterprise/                     # Enterprise-only Rust crates (COMMERCIAL)
│   ├── Cargo.toml
│   └── crates/
│       ├── wo-digital-signature/       # PAdES, XAdES, document signing
│       ├── wo-redaction/                # Document redaction (black bars, remove content)
│       ├── wo-drm/                      # Document rights management, encryption
│       ├── wo-watermark/                # Dynamic watermarking (text, image)
│       ├── wo-comparison/               # Advanced document comparison (legal redline)
│       └── wo-converter-pro/            # Batch conversion, priority queue, advanced formats
│
├── services/                            # Rust microservices (AGPL core, commercial extensions)
│   ├── conversion-service/              # Format conversion (AGPL core)
│   ├── coauthoring-service/             # Real-time collaboration (AGPL core)
│   ├── storage-service/                 # File storage abstraction (AGPL core)
│   ├── session-service/                 # JWT/session management (AGPL core)
│   └── identity-service/                # OAuth2/OIDC/SAML (AGPL core, SAML=Enterprise)
│
├── apps/
│   ├── web/                             # React 19 frontend (AGPL)
│   │   ├── editor-shell/                # Shared editor layout
│   │   ├── document-editor/             # Word-like
│   │   ├── spreadsheet-editor/          # Excel-like
│   │   ├── presentation-editor/         # PowerPoint-like
│   │   ├── pdf-editor/                  # PDF viewer/editor
│   │   ├── visio-editor/                # Diagram editor
│   │   ├── forms-editor/                # Form builder
│   │   └── admin-panel/                 # Admin UI (AGPL base + Enterprise features)
│   ├── web-enterprise/                  # Enterprise frontend extensions (COMMERCIAL)
│   │   ├── compliance-dashboard/         # Audit logs, data retention, DLP dashboard
│   │   ├── analytics-dashboard/          # Usage analytics, document insights
│   │   ├── review-workflows/            # Approval chains, review tracking
│   │   └── admin-enterprise/            # SSO config, SCIM, branding, advanced settings
│   └── desktop/                         # Tauri 2.0 (AGPL core + Commercial features)
│       ├── src-tauri/                   # Rust shell
│       └── src/                         # React webview
│
├── packages/
│   ├── ui-kit/                          # Design system (AGPL)
│   ├── editor-core/                     # Shared editor logic (AGPL)
│   ├── types/                           # Shared TypeScript types
│   ├── eslint-config/                   # Shared lint rules
│   ├── tsconfig/                        # Shared TS configs
│   ├── api-client/                      # Typed API client
│   ├── enterprise-features/             # Feature flag runtime (COMMERCIAL)
│   │   └── src/                         # License validation, feature gating
│   └── document-builder/                # Template engine (COMMERCIAL)
│       └── src/                         # Mail merge, batch generation
│
├── integrations/                       # (AGPL)
│   ├── nextcloud/                       # Vue 3
│   ├── wopi/                            # WOPI protocol server (Rust)
│   └── opencloud/                       # Node.js → Rust migration
│
├── tools/
│   ├── docker/                          # Dev + CI images
│   ├── ci/                              # Forgejo Actions workflows
│   ├── license-check/                   # Validate dual license compliance
│   └── traffic-replay/                   # Production traffic replay tool
│
└── tests/
    ├── e2e/                             # Playwright
    ├── unit/                            # Vitest
    ├── integration/                     # API tests
    └── enterprise/                       # Enterprise-specific tests (commercial)
```

### 2.3 Key Architectural Decisions

#### Decision 1: Core Engine — Rust Rewrite (Manual, with cxx Bridge)

**Choice:** Manual Rust rewrite module-by-module, with `cxx` bridge for interim C++ interop  
**Motivation:** Memory safety (user requirement)  
**Honest assessment:** There is NO automated C++→Rust transpiler. c2rust handles C only — C++ semantics (classes, templates, inheritance, virtual dispatch) are fundamentally incompatible with direct translation. The "transpile then refactor" approach is adapted to "bridge then manually rewrite."

**Parallel development strategy (not just rewrite):**
- C++ codebase continues to be maintained and patched during the entire rewrite
- Rust modules are developed alongside C++, not as a replacement-in-place
- Both implementations run in parallel; CI tests both against the same format corpus
- No module is removed until its Rust equivalent passes 100% parity tests

**cxx bridge usage (temporary, enforced removal):**
- Each C++ module gets a `cxx` FFI wrapper when its Rust rewrite begins
- Rust calls C++ through the bridge during parallel development
- Bridge is removed ONLY when Rust module achieves 100% parity
- Lint rule enforces: if a Rust crate exists for a module, the cxx bridge for that module must be removed
- If bridge remains >6 months after Rust module ships, it triggers an escalation review

**Rust crate equivalents for C++ third-party deps:**
| C++ Dep | Rust Equivalent | Notes |
|---------|----------------|-------|
| Boost (filesystem, algorithm, etc.) | std::fs, itertools, etc. | Direct replacements |
| ICU | `icu4x` | Unicode Consortium's official Rust ICU |
| FreeType | `ttf-parser` + `fontdb` | Pure Rust, no FFI |
| HarfBuzz | `harfbuzz-sys` (FFI) | **No pure Rust alternative exists.** Text shaping stays as C FFI. This is acceptable — HarfBuzz is a thin C library, not a C++ complexity problem. |
| OpenSSL | `rustls` | Pure Rust TLS |
| Crypto++ | `ring` | Pure Rust crypto |
| zlib/brotli | `flate2` / `brotli` | Pure Rust |
| Hunspell | `hunspell-rs` (FFI) | FFI acceptable for spellcheck; evaluate `nlprule` as pure Rust alternative |
| libcurl | `reqwest` | Pure Rust HTTP |
| V8 | **V8 via `cxx` bridge** | **Boa is NOT production-ready** (no JIT, incomplete ES2020). JavaScript macro compatibility requires V8. Keep V8 as C++ dependency via FFI — this is a known permanent exception to the "pure Rust" goal. |

#### Decision 2: Microservices Architecture

**Choice:** Decompose server monolith into 5 Rust microservices + 1 API gateway  
**Rationale:** Each service has distinct scaling, latency, and resource profiles.

**Data layer (new — was missing):**
- **PostgreSQL** — Co-authoring state, document metadata, user sessions, audit logs
- **Redis** — Caching (document locks, session cache, rate limit counters), pub/sub for service events
- **RabbitMQ** — Job queue for conversion service (replaces current in-process queue)

```
                    ┌──────────────────┐
                    │  API Gateway     │  Rust (Axum)
                    │  Rate limit      │
                    │  JWT validation  │
                    │  Request routing │
                    └──────┬──────────┘
                           │
         ┌─────────────────┼─────────────────┐
         │                 │                 │
  ┌──────┴──────┐  ┌──────┴──────┐  ┌──────┴──────┐
  │ Co-authoring │  │ Conversion  │  │  Storage    │
  │ Service      │  │ Service     │  │  Service    │
  │ (WebSocket)  │  │ (CPU-heavy) │  │ (I/O-bound) │
  │ CRDT engine  │  │ Job queue   │  │ FS/S3/Azure │
  │ Tokio async  │  │ (RabbitMQ)  │  │             │
  └──────┬───────┘  └──────┬──────┘  └─────────────┘
         │                 │
  ┌──────┴──────┐  ┌──────┴──────┐
  │  Session    │  │  Identity   │
  │  Service    │  │  Service    │
  │  JWT issue  │  │  OAuth2     │
  │  Authz      │  │  OIDC       │
  │  (Postgres) │  │  SAML       │
  └─────────────┘  └─────────────┘
```

| Service | Language | Data Store | Why Separate |
|---------|----------|-----------|-------------|
| **API Gateway** | Rust (Axum) | Redis (rate limit cache) | Thin proxy, JWT validation, routing. Rust for performance — no Node.js bottleneck. |
| **Conversion Service** | Rust | RabbitMQ (jobs), PostgreSQL (job state), local disk (temp files) | CPU-intensive format conversion. Scales horizontally with job queue. Calls Rust core directly (no FFI overhead). |
| **Co-authoring Service** | Rust | PostgreSQL (document state, history), Redis (locks, cursors) | WebSocket state management, CRDT engine. Latency-sensitive, stateful. |
| **Storage Service** | Rust | FS / S3 / Azure Blob | File I/O abstraction. I/O-bound. Shared across other services. |
| **Session Service** | Rust | PostgreSQL, Redis (session cache) | JWT issuance, token refresh, session management. Security-critical, separate from identity. |
| **Identity Service** | Rust | PostgreSQL, external IdP connections | OAuth2/OIDC/SAML federation. Different lifecycle from sessions. |

**Inter-service communication:** HTTP+JSON with OpenAPI specs (not gRPC — premature optimization for a 6-person team). Redis pub/sub for events (document lock changes, conversion completion). RabbitMQ for async job submission.

#### Decision 3: Frontend — React 19

**Choice:** React 19 + Zustand + Radix UI + Tailwind CSS 4 + Vite  
**Rationale:** AdminPanel already React 18, largest talent pool, canvas rendering is framework-agnostic.

#### Decision 4: Desktop — Tauri 2.0

**Choice:** Tauri 2.0 (Rust shell + system WebView + React frontend)  
**Rationale:** Eliminates CEF (200MB→10MB), FOSS, Rust native bridge, mobile support.

#### Decision 5: Build System — Turborepo + Cargo + pnpm

**Choice:** Turborepo orchestrates everything. Cargo builds Rust. pnpm manages JS.  
**Rationale:** One command (`turbo run build`) builds the entire project.

#### Decision 6: Design System — Radix UI Primitives

**Choice:** Custom design system on Radix UI + OKLCH tokens + dark mode from Day 1.

---

## 3. Core Rust Rewrite — Detailed Strategy

### 3.1 Rewrite Order (by difficulty)

```
Wave 1: TRIVIAL + LOW (419 files) ──── Weeks 9-14
  Fb2File(7) → TxtFile(16) → HtmlFile2(22) → XpsFile(19) → 
  OFDFile(58) → DjVuFile(117) → EpubFile(86) → OfficeUtils(90)

Wave 2: MEDIUM (1,174 files) ──────── Weeks 15-24
  RtfFile(168) → HwpFile(165) → PdfFile(301) → OdfFile(841) → DocxRenderer(46)

Wave 3: HIGH (4,385 files) ────────── Weeks 25-48
  UnicodeConverter(998) → MsBinaryFile(2,954) → OOXML(3,387) → X2tConverter(38)

Wave 4: CRITICAL (4,182 files) ────── Weeks 49-72
  DesktopEditor(3,152) → Common(1,030)
```

### 3.2 cxx Bridge Strategy

During the rewrite, C++ modules that haven't been rewritten yet are accessed via `cxx`:

```rust
// core/crates/wo-ooxml/src/bridge.rs
#[cxx::bridge]
mod ffi {
    extern "C++" {
        include!("core/OOXML/DocxFormat/DocxFormat.h");
        
        type CDocxFile;
        
        fn open(path: &str) -> UniquePtr<CDocxFile>;
        fn read_content(&self) -> String;
        fn close(self: UniquePtr<CDocxFile>);
    }
}

// Rust wrapper with safe API
pub struct DocxFile {
    inner: cxx::UniquePtr<ffi::CDocxFile>,
}

impl DocxFile {
    pub fn open(path: &Path) -> Result<Self> {
        let inner = ffi::open(&path.to_string_lossy());
        Ok(Self { inner })
    }
    
    pub fn read_content(&self) -> Result<String> {
        Ok(self.inner.read_content())
    }
}
```

When a module is fully rewritten in Rust, the cxx bridge is removed and the Rust implementation is used directly.

### 3.3 Format Parity Testing

**Before ANY Rust rewrite begins:**

0. **Discovery phase (Weeks 5-8, NEW):**
   - Profile C++ codebase for undefined behavior (ASan, UBSan, MSan)
   - Document complex patterns (31 multiple inheritance hierarchies, 263 virtual dispatch chains)
   - Identify C++ bugs that documents may depend on (behavioral parity vs. spec parity decision)
   - Create detailed module specs for each of the 18 format crates

1. **Build format test corpus** — Collect 10,000+ documents covering:
   - All supported formats (DOCX, XLSX, PPTX, PDF, ODT, ODS, ODP, DOC, XLS, PPT, RTF, EPUB, etc.)
   - Edge cases: corrupted files, password-protected, embedded objects, macros, complex layouts
   - Real-world documents from bug reports
   - **Golden master documents:** Original Office files (not Word Office output) as ground truth

2. **Create roundtrip test harness:**
   ```
   For each document in corpus:
     1. Open with C++ core → extract content tree (XML/JSON)
     2. Save to same format with C++ core
     3. Open saved file with C++ core → extract content tree
     4. Diff: original tree vs roundtrip tree → MUST be identical
   ```

3. **Rust module acceptance criteria:**
   ```
   For each Rust module (e.g., wo-fb2):
     1. Run roundtrip test corpus for that format against Rust implementation
     2. Compare output with C++ implementation
     3. Compare BOTH against golden master (original Office documents)
     4. 100% of documents must produce identical output to C++
     5. Differences from C++ that match golden master = C++ bug, acceptable
     6. Differences from C++ that DON'T match golden master = Rust bug, blocker
     7. Performance within 20% of C++ (relaxed initially, tightened over time)
   ```

### 3.4 Extended Testing Strategy (beyond format parity)

| Test Type | Tool | Purpose |
|-----------|------|---------|
| **Format roundtrip** | cargo test + custom harness | 100% parity with C++ output |
| **Fuzz testing** | `cargo-fuzz` + AFL | Find parser crashes, memory issues in Rust code |
| **Property-based testing** | `proptest` | Generate random documents, test invariants (every valid DOCX must parse, etc.) |
| **Performance benchmarks** | `criterion` | Per-format benchmarks with statistical significance (not "within 20%") |
| **Accessibility testing** | axe-core + Playwright | WCAG 2.1 AA automated checks per PR |
| **Production traffic replay** | Custom tool (see Section 12) | Replay anonymized production API traffic against both C++ and Rust implementations |

### 3.5 Production Traffic Replay — Implementation Detail

**Purpose:** Catch regressions that synthetic tests miss by replaying real user traffic.

**Architecture:**
1. **Capture:** Deploy traffic capture proxy in production (logs requests/responses with PII stripped). Store in immutable append-only log (S3).
2. **Anonymize:** Automated pipeline that replaces user IDs, document names, file contents with hash tokens. Preserves request structure, headers, timing.
3. **Replay:** Standalone tool that replays captured requests against both C++ and Rust implementations, comparing responses.
4. **Metrics:** Per-endpoint diff rate, latency comparison (p50/p95/p99), error rate delta.

**Privacy requirements:**
- All PII stripped before storage (user IDs, emails, IPs, document contents)
- Retention: 90 days max, auto-purge
- Access: restricted to CI system, no human access to raw logs
- GDPR compliance: no personal data in replay corpus

**Success criteria:**
- Replay covers 95%+ of production API endpoints
- Response diff rate < 0.1% (accounting for timestamps, nonces)
- Automated run on every Rust core release candidate

---

## 4. Execution Phases

### Phase 0: Foundation (Weeks 1-8)

**Goal:** Monorepo, tooling, design system, format test corpus, discovery, legal review

| Task | Effort | Category | Parallel |
|------|--------|----------|----------|
| Consolidate 22 repos into pnpm + Cargo monorepo | 2 weeks | unspecified-high | No (blocks everything) |
| Set up Turborepo, shared ESLint flat config, tsconfig, Biome | 1 week | quick | After monorepo |
| **Discovery phase: C++ profiling** (ASan/UBSan, pattern docs, module specs) | 3 weeks | deep | Parallel with below |
| **Legal review** (license audit, patent assessment, liability framework) | 3 weeks | — | Parallel (Phase 0.5) |
| Build format test corpus (10k+ documents, golden masters) | 2 weeks | deep | Parallel |
| Build roundtrip test harness (C++ baseline) | 2 weeks | deep | After corpus |
| Create design-system package (tokens + Radix primitives) | 3 weeks | visual-engineering | Parallel |
| Set up Playwright E2E infrastructure | 1 week | quick | Parallel |
| Set up CI (Forgejo Actions + Woodpecker) | 1 week | unspecified-low | Parallel |
| Tauri WebView canvas POC | 1 week | deep | Parallel |
| Set up Rust workspace + cxx bridge scaffolding | 1 week | unspecified-high | After monorepo |
| Set up PostgreSQL + Redis + RabbitMQ for dev environment | 1 week | quick | Parallel |

**Exit Criteria:**
- `turbo run build` succeeds across entire monorepo
- Discovery report: UB profile, complex pattern documentation, module specs for all 18 crates
- Legal opinion: rewrite cleared to proceed (or specific blockers documented)
- Format roundtrip tests pass against C++ core (baseline established)
- Design system has 10+ components
- Tauri POC renders a canvas in system WebView

### Phase 1: Server → Microservices (Weeks 9-28)

**Goal:** Decompose monolithic Node.js server into microservices

**Runs in parallel with Phase 2 (Rust core rewrite starts).**

| Task | Effort | Category | Parallel |
|------|--------|----------|----------|
| TypeScript migration (server/, strict mode, ESM) | 3 weeks | unspecified-high | No |
| Build API Gateway (Fastify, JWT validation, routing) | 2 weeks | deep | After TS |
| Build Storage Service (Rust: FS/S3/Azure abstraction) | 3 weeks | deep | After Gateway |
| Build Conversion Service shell (Rust: job queue, API) | 2 weeks | deep | After Storage |
| Build Co-authoring Service shell (Rust: WebSocket, CRDT) | 4 weeks | deep | After Gateway |
| Build Auth Service (Rust: JWT/OAuth2) | 2 weeks | deep | After Gateway |
| Migrate AdminPanel to Vite + React 19 | 1 week | quick | Independent |
| Add OpenTelemetry + Grafana stack | 1 week | unspecified-low | Parallel |
| Integration tests for all services | 3 weeks | deep | Ongoing |
| Migrate co-authoring from socket.io to native WS | 2 weeks | deep | After coauth service |

**Exit Criteria:**
- All 4 Rust services start and respond to health checks
- API Gateway routes requests to correct services
- Co-authoring works through new WebSocket path
- Storage abstraction works for FS + S3
- Auth service issues valid JWTs
- Zero regressions vs current server responses

### Phase 2: Rust Core — Small Formats (Weeks 9-14)

**Goal:** Rewrite trivial/low-difficulty format modules (419 files total)

**Runs in parallel with Phase 1 (server microservices).**

| Task | Effort | Category | Parallel |
|------|--------|----------|----------|
| wo-common crate (shared types, errors, utils) | 1 week | deep | No |
| wo-txt (16 files) | 2 days | quick | After common |
| wo-fb2 (7 files) | 1 day | quick | After common |
| wo-html (22 files) | 3 days | quick | After common |
| wo-xps (19 files) | 3 days | quick | After common |
| wo-ofd (58 files) | 1 week | unspecified-high | After common |
| wo-djvu (117 files) | 1 week | unspecified-high | After common |
| wo-epub (86 files) | 1 week | unspecified-high | After common |
| wo-office-utils (90 files) | 1 week | unspecified-high | After common |
| Roundtrip tests for all 8 modules | 1 week | deep | After all modules |

**Exit Criteria:**
- All 8 Rust crates compile and pass tests
- 100% format roundtrip parity with C++ for all 8 formats
- Performance within 20% of C++

### Phase 3: Rust Core — Medium Formats (Weeks 15-28)

**Goal:** Rewrite medium-difficulty format modules (1,474 files total)

| Task | Effort | Category | Parallel |
|------|--------|----------|----------|
| wo-rtf (168 files) | 2 weeks | deep | No |
| wo-hwp (165 files) | 2 weeks | deep | Parallel with RTF |
| wo-pdf (301 files) | 4 weeks | deep | After RTF (reuses PDF knowledge) |
| wo-odf (841 files) | 5 weeks | deep | Parallel with PDF |
| wo-docx-renderer (46 files) | 1 week | quick | After ODF + PDF |
| Roundtrip tests for all 5 modules | 2 weeks | deep | After all modules |

**Exit Criteria:**
- 100% format roundtrip parity for RTF, HWP, PDF, ODF
- DOCX→PDF rendering via Rust produces identical output to C++
- Cumulative: 13 of 18 format modules in Rust

### Phase 4: Web UI Rewrite (Weeks 21-48)

**Goal:** Replace 28k files of Backbone/jQuery with React 19

**Starts mid-Phase 3, runs long. The web editor has two layers:**
- **Chrome** (toolbars, menus, panels, status bar) — this is the 28k file mess → REWRITE
- **Canvas** (document rendering) — framework-agnostic `<canvas>` → KEEP

| Task | Effort | Category | Parallel |
|------|--------|----------|----------|
| Editor shell (layout: toolbar + canvas + sidebar + statusbar) | 3 weeks | visual-engineering | No |
| Design system v2 (editor-specific components: FormulaBar, CellEditor, SlidePanel) | 3 weeks | visual-engineering | After shell |
| document-editor app (Word-like) | 5 weeks | visual-engineering | After shell |
| spreadsheet-editor app (Excel-like) | 6 weeks | visual-engineering | After shell |
| presentation-editor app (PowerPoint-like) | 5 weeks | visual-engineering | After shell |
| pdf-editor app | 3 weeks | visual-engineering | After shell |
| visio-editor app | 3 weeks | visual-engineering | After shell |
| forms-editor app | 3 weeks | visual-engineering | After shell |
| Plugin API (for aiautofill, etc.) | 3 weeks | deep | Parallel |
| Keyboard shortcut system (preserve ALL existing shortcuts) | 2 weeks | deep | Parallel |
| Accessibility (WCAG 2.1 AA) | 4 weeks | visual-engineering | Ongoing |
| Unit tests (Vitest + Testing Library) | Ongoing | quick | Continuous |

**Exit Criteria:**
- All 7 editor apps render documents identically to current version
- All keyboard shortcuts preserved
- Lighthouse accessibility > 90
- Bundle size < current (tree-shaking, code splitting)
- Feature flag can switch between old and new UI

### Phase 5: Rust Core — Large Formats (Weeks 29-52)

**Goal:** Rewrite the big format parsers (7,339 files)

| Task | Effort | Category | Parallel |
|------|--------|----------|----------|
| wo-unicode (998 files, ICU wrappers → icu4x) | 4 weeks | deep | No |
| wo-msbinary (2,954 files, legacy .doc/.xls/.ppt) | 12 weeks | deep | After unicode |
| wo-ooxml (3,387 files, DOCX/XLSX/PPTX) | 12 weeks | deep | After msbinary |
| wo-x2t (38 files, conversion orchestrator) | 2 weeks | deep | After all format crates |
| Roundtrip tests for all modules | 4 weeks | deep | After all modules |
| Performance optimization | 4 weeks | deep | After tests |

**Exit Criteria:**
- 100% format roundtrip parity for ALL formats (18/18 modules)
- DOCX, XLSX, PPTX parsing identical to C++ output
- Legacy .doc, .xls, .ppt parsing identical to C++ output
- Performance within 10% of C++ for all formats

### Phase 6: Rust Core — Rendering Engine (Weeks 53-72)

**Goal:** Rewrite DesktopEditor (3,152 files) — canvas rendering, fonts, raster

**This is the hardest phase. The rendering engine is the most complex subsystem.**

| Task | Effort | Category | Parallel |
|------|--------|----------|----------|
| wo-fonts (FreeType + HarfBuzz → Rust FFI + fontdb) | 4 weeks | deep | No |
| wo-raster (image processing, PNG/JPEG/TIFF/GIF) | 6 weeks | deep | After fonts |
| wo-renderer core (canvas API, drawing primitives) | 8 weeks | deep | After raster |
| wo-renderer text (text layout, shaping, bidirectional) | 4 weeks | deep | After renderer core |
| wo-renderer graphics (paths, transforms, blending, gradients) | 4 weeks | deep | Parallel with text |
| wo-common remaining (1,030 files, network, plugins, etc.) | 6 weeks | deep | Parallel |
| Rendering comparison tests (pixel-perfect vs C++) | 4 weeks | deep | After renderer |
| Performance benchmarking and optimization | 4 weeks | deep | After tests |

**Exit Criteria:**
- Documents render identically (pixel-perfect) to C++ version
- Font rendering matches C++ output across 100+ font files
- Performance within 15% of C++ for rendering benchmarks

### Phase 7: Desktop Tauri Rewrite (Weeks 57-72)

**Goal:** Replace CEF+Qt desktop with Tauri 2.0

**Starts mid-Phase 6, once Rust rendering engine is partially available.**

| Task | Effort | Category | Parallel |
|------|--------|----------|----------|
| Tauri shell (window management, menus, system tray) | 2 weeks | deep | No |
| Native file system bridge (Rust ↔ JS) | 2 weeks | deep | After shell |
| Print support (Rust print API) | 2 weeks | deep | After bridge |
| Auto-updater (Tauri updater) | 1 week | quick | After shell |
| Keychain integration | 1 week | quick | After bridge |
| Windows installer (NSIS) | 1 week | quick | Parallel |
| Linux packages (deb/rpm) | 1 week | quick | Parallel |
| macOS app bundle + signing | 1 week | quick | Parallel |
| Migrate Android app to Tauri mobile | 4 weeks | deep | After desktop stable |

**Exit Criteria:**
- Desktop opens, loads, edits, saves documents
- Native file save/open on Windows + Linux + macOS
- Print works
- Package sizes: Windows < 30MB, Linux < 25MB, macOS < 35MB

### Phase 8: WASM + Offline (Weeks 73-88)

**Goal:** Compile Rust core to WASM, enable browser-side rendering

**Now that core is Rust, WASM compilation is native (no Emscripten hacks).**

| Task | Effort | Category | Parallel |
|------|--------|----------|----------|
| Compile wo-x2t to WASM (format conversion in browser) | 4 weeks | deep | No |
| Client-side format conversion (DOCX→PDF, etc.) | 3 weeks | deep | After WASM |
| Compile wo-renderer to WASM (client-side rendering) | 6 weeks | deep | After x2t WASM |
| Offline editing (IndexedDB + Service Worker) | 4 weeks | deep | After rendering WASM |
| Performance optimization (WASM binary size, load time) | 3 weeks | deep | After all WASM |
| Gradual degradation strategy (server → WASM fallback) | 2 weeks | unspecified-high | After all |

**Exit Criteria:**
- DOCX→PDF conversion works in browser without server
- Document rendering works client-side
- Offline editing works 30+ minutes
- WASM binary < 10MB (gzipped, Rust is smaller than C++ WASM)

### Phase 9: Integrations + Hardening (Weeks 89-104)

**Goal:** Modernize integrations, harden for production

| Task | Effort | Category | Parallel |
|------|--------|----------|----------|
| WOPI server (Rust, replace document-server-integration) | 3 weeks | deep | No |
| Migrate Nextcloud app to new WOPI + API | 2 weeks | unspecified-high | After WOPI |
| Modernize word-office-opencloud (Node.js → Rust) | 3 weeks | deep | Parallel |
| WebDAV support | 2 weeks | deep | Parallel |
| Security audit (external) | 2 weeks | — | After all services |
| Load testing (conversion + co-authoring under load) | 2 weeks | deep | After security |
| Performance optimization pass | 3 weeks | deep | After load testing |
| Documentation (architecture, API, contributing) | 3 weeks | writing | Parallel |
| Update all integration examples (7 languages) | 2 weeks | quick | After docs |

**Exit Criteria:**
- WOPI works with Nextcloud 30+
- All integrations use new API
- Security audit passed
- Load test: 100 concurrent users editing, < 500ms latency

### Phase 10: Launch Preparation (Weeks 133-180)

**Goal:** Production readiness, migration path, release

**This is the buffer phase. 48 weeks allocated to absorb delays from Phases 0-9.**

| Task | Effort | Category | Parallel |
|------|--------|----------|----------|
| Migration guide (from Word Office to Word Office) | 2 weeks | writing | No |
| Deployment guide (Docker, Kubernetes, bare metal) | 2 weeks | writing | Parallel |
| Remove all C++ code from monorepo (except V8 + HarfBuzz) | 2 weeks | deep | After Phase 6-8 confirmed |
| Remove cxx bridges | 1 week | quick | After C++ removal |
| Beta release + feedback collection | 6 weeks | — | After removal |
| Bug fixes from beta | 6 weeks | deep | After beta |
| Performance optimization pass | 4 weeks | deep | After bug fixes |
| Release candidate + final QA | 4 weeks | deep | After optimization |
| v1.0 release | — | — | Final |

**Exit Criteria:**
- Zero C++ code in monorepo
- Beta testers confirm format parity
- Deployment works on Docker + bare metal
- v1.0 released

---

## 5. Resource Allocation

Phases overlap intentionally. Here's who works on what, when:

```
Week:    1-8   9-14  15-28  29-48  49-72  73-88  89-132  133-180
         ├──────┼──────┼──────┼──────┼──────┼──────┼───────┼───────┤
Lead     ████████████████████████████████████████████████████████████
Rust×2   ░░░░░░████████████████████████████████████████████░░░░░░░░
Rust Svc ░░░░░░████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
FE×2     ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
Desktop  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
QA       ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
DevOps   ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
Designer ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
Legal    ░░░░░████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
```

**Overlap rules:**
- Phase 1 (Server) + Phase 2 (Small Formats) share W9-14: 1 Rust Core + 1 Rust Services + 1 Frontend (design system)
- Phase 3 (Medium Formats) + Phase 4 (Web UI) overlap W25-30: 1 Rust Core + 2 Frontend
- Phase 6 (Rendering) + Phase 7 (Desktop) overlap W69-72: 2 Rust Core + 1 Desktop
- Phase 10 (Launch) is buffer — team redistributes to fix Phase 9 issues

**Peak team:** 8-10 people (W25-60 when 3+ phases overlap). **Minimum:** 5 FTE (when phases are sequential).

## 6. Timeline Overview

```
Phase 0:  Foundation + Discovery  ████████░░░░░░░░░░░░░░░░░░░░░░░░░  W1-8
Phase 1:  Server → Services       ░░░░░░░░████████████████░░░░░░░░░░  W9-28
Phase 2:  Core: Small Formats      ░░░░░░░░████░░░░░░░░░░░░░░░░░░░░░  W9-14
Phase 3:  Core: Medium Formats      ░░░░░░░░░░░░░░████████████░░░░░░░  W15-30
Phase 4:  Web UI Rewrite           ░░░░░░░░░░░░░░░░░░████████████████  W25-52
Phase 5:  Core: Large Formats      ░░░░░░░░░░░░░░░░░░░░░░░░░░░████░░  W31-60
Phase 6:  Core: Rendering           ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░████  W61-84
Phase 7:  Desktop Tauri             ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░████  W69-88
Phase 8:  WASM + Offline           ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  W89-108
Phase 9:  Integrations + Harden    ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  W109-132
Phase 10: Launch + Buffer          ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  W133-180
```

**Total: 180 weeks (3.5 years)** — includes 50 weeks buffer for discovery, complexity surprises, testing, and team ramp-up.

**Key milestones:**
- **Week 8:** Discovery complete, legal cleared, test corpus established
- **Week 14:** First Rust format modules shipped (8/18 formats)
- **Week 30:** Server decomposed into microservices, 13/18 format modules in Rust
- **Week 52:** Modern React web UI shipped
- **Week 60:** ALL format modules in Rust (18/18)
- **Week 84:** Rendering engine in Rust
- **Week 88:** Desktop app on Tauri
- **Week 108:** WASM in browser, offline editing
- **Week 132:** Production-hardened, all integrations migrated
- **Week 180:** v1.0 release, zero C++ code (except V8 + HarfBuzz FFI)

---

## 7. Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| **Format parity regression during Rust rewrite** | High | Critical | 10k+ document test corpus; golden master tests against original Office files; C++ stays maintained in parallel; behavioral vs. spec parity decision documented |
| **C++ UB dependency** — documents that rely on C++ undefined behavior | Medium | High | Discovery phase (ASan/UBSan profiling); document known UB-dependent behaviors; treat as C++ bugs if they don't match golden master |
| **Rendering engine rewrite takes much longer** | High | High | cxx bridge allows C++ rendering to be used while Rust rendering is developed; this is parallel development, not in-place replacement |
| **WASM binary size too large** | Medium | Medium | Rust WASM is smaller than C++; wasm-opt + wasm-strip; lazy-load format modules; realistic target: 15-20MB (not 10MB) |
| **Tauri WebView canvas performance** | Medium | High | POC in Phase 0 with real document rendering; benchmark early; fallback to Electron if >30% degradation |
| **Co-authoring breaks during service split** | Medium | Critical | Run old and new in parallel; proxy traffic gradually; both use same PostgreSQL state |
| **Team burnout (3.5 year project)** | High | High | Ship early (small formats in W14); modular architecture; celebrate milestones; buffer weeks built in |
| **Rust talent scarcity** (C++ + Rust + document formats) | High | High | This is the critical path, not code. Recruitment plan in Section 8. cxx bridge lowers barrier for incremental contribution. |
| **cxx bridge becomes permanent** | Medium | Medium | Lint rule: bridge removed within 6 months of Rust module shipping. Escalation review if not. |
| **Key person risk** — single engineer understands a critical module | High | Critical | Pair programming on all format modules; documentation-first; no siloed knowledge |
| **License compliance** (AGPL + Apache-2.0 + 29 third-party deps) | Medium | High | Legal review phase (Phase 0.5). SPDX headers. Audit all Rust crate licenses. |
| **Patent risk** in format parsers (OOXML, PDF algorithms) | Low | High | Legal review. C++ code may have implicit patent licenses from Word Office; Rust rewrite needs its own analysis. |
| **Data loss liability** — Rust version corrupts documents | Low | Critical | Roundtrip tests against golden masters. Feature flag: Rust conversion off by default until 100% parity proven. Rollback plan (Section 9). |
| **V8 dependency** — JavaScript macros require V8, no Rust equivalent | Certain | Medium | V8 stays as permanent C++ dependency via cxx bridge. Accept this. Boa is not production-ready. |
| **HarfBuzz FFI** — no pure Rust text shaping alternative | Certain | Low | HarfBuzz is a thin C library, well-maintained. FFI overhead is negligible for text shaping. |

### Escalation Triggers — Reconsider entire approach if:
- Discovery phase reveals >40% of C++ code has UB dependencies
- No Rust engineers with document format experience hired within 3 months
- Legal review identifies patent risks in format parsing
- Tauri POC shows >50% performance degradation vs CEF
- V8 major version update breaks cxx bridge API (must evaluate Boa maturity at that point)

### Rollback Strategy
See Section 9 for detailed contingency plan if Rust rewrite fails mid-way.

---

## 8. Team & Resource Requirements

| Role | Allocation | Phases |
|------|-----------|--------|
| Tech Lead / Architect | 1.0 FTE | All phases |
| Rust Engineer (Core) | 2.0 FTE | Phase 2-6, 8 |
| Rust Engineer (Services) | 1.0 FTE | Phase 1, 5, 9 |
| Frontend Engineer (React) | 2.0 FTE | Phase 0, 4, 7 |
| Desktop Engineer (Tauri/Rust) | 1.0 FTE | Phase 7 |
| QA Engineer | 1.0 FTE | All phases |
| DevOps / CI | 0.5 FTE | Phase 0, ongoing |
| Designer (Design System) | 0.5 FTE | Phase 0-4 |
| Technical Writer | 0.5 FTE | Phase 9-10 |
| Legal Counsel (part-time) | 0.25 FTE | Phase 0.5, 5, 10 |

**Minimum viable team:** 5 full-time (1 lead + 2 Rust + 1 frontend + 1 QA)

**Recruitment challenge (critical path):**
The hardest role to fill is "Rust engineer who also understands C++ document format parsing." This Venn diagram has near-zero overlap. Mitigation:
- Hire strong Rust engineers who can learn format specs (easier than teaching C++ devs Rust)
- Pair Rust engineers with C++ codebase experts during discovery phase
- Invest in internal training: Rust workshop + document format deep-dive
- Consider hiring from LibreOffice/Collabora ecosystem (they have both skills)

## 9. Rollback Strategy

**If Rust rewrite fails mid-way, the project is NOT lost.** The parallel development strategy ensures C++ codebase remains functional.

| Scenario | Rollback Action |
|----------|----------------|
| Single format module fails parity | Keep cxx bridge for that module; C++ handles that format; continue with other modules |
| Rendering engine rewrite fails | DesktopEditor stays as C++; use via cxx bridge permanently; focus on format parsers only |
| Team cannot be hired | Pause Rust rewrite; continue with server modernization (Fastify + TypeScript) and web UI rewrite (React) — both deliver value independently |
| Legal/patent issues discovered | Halt affected module; legal review; may need to license patents or redesign algorithm |
| Budget/time runs out | Ship what's done. Each phase is independently valuable. Rust format modules can be integrated into C++ server via cxx bridge. |

**Key principle:** Every phase delivers value independently. The project degrades gracefully, not catastrophically.

## 10. Legal Review Phase (Phase 0.5, Weeks 5-8)

**Runs in parallel with discovery phase (Weeks 5-8). Total effort: ~6 person-weeks, distributed across 4 calendar weeks with parallel execution.**

| Task | Effort | Parallel |
|------|--------|----------|
| Audit all 29 C++ third-party dependency licenses | 1 week | Runs in parallel |
| Verify Rust crate licenses are compatible (AGPL + Apache-2.0) | 1 week | Runs in parallel |
| Patent risk assessment for OOXML, PDF parsing algorithms | 2 weeks | Runs in parallel |
| Liability framework for document corruption scenarios | 1 week | Runs in parallel |
| Contributor agreement for new Rust code (CLAs) | 1 week | Runs in parallel |

**Exit criteria:** Written legal opinion confirming rewrite can proceed, or specific license/patent issues that must be resolved before continuing.

---

## 11. Technology Stack Summary

| Layer | Current | Target |
|-------|---------|--------|
| **Core Engine** | C++17 (10,385 files) | Rust (Cargo workspace, 18+ crates). Permanent exceptions: V8 (JS macros) + HarfBuzz (text shaping) via cxx bridge. |
| **Core Enterprise** | None | Rust (6 enterprise crates: digital signatures, redaction, DRM, watermarking, comparison, pro converter) |
| **Server** | Express 4, CommonJS, `co` | 5 Rust microservices (Axum) + PostgreSQL + Redis + RabbitMQ |
| **Frontend** | Backbone + jQuery + RequireJS + LESS + Grunt | React 19 + Zustand + Radix UI + Tailwind + Vite |
| **Frontend Enterprise** | None | React 19 (compliance dashboard, analytics, review workflows, enterprise admin) |
| **Desktop** | CEF + Qt 5.15 (9,505 files) | Tauri 2.0 (Rust shell + system WebView + React) |
| **Build** | Grunt + Makefile + CMake | Turborepo + Cargo + pnpm + Vite |
| **Testing** | Jest (minimal), no E2E | Vitest + Playwright + cargo test + cargo-fuzz + proptest + format roundtrip |
| **Linting** | ESLint (inconsistent) | Biome (Rust + JS/TS) |
| **CI/CD** | GitHub Actions | Forgejo Actions + Woodpecker CI |
| **Monitoring** | statsd | OpenTelemetry + Grafana |
| **Inter-service** | N/A (monolith) | HTTP+JSON (OpenAPI), Redis pub/sub, RabbitMQ jobs |
| **Data** | None (in-process) | PostgreSQL + Redis + RabbitMQ |
| **Monorepo** | 22 repos | pnpm + Cargo workspaces + Turborepo |
| **Design** | None | Custom on Radix UI + OKLCH tokens |
| **Licensing** | AGPL-3.0 only | Dual: AGPL-3.0 (Community) + Commercial (Enterprise) |

---

## 12. Dual License Architecture

### 12.1 License Model

World-Office uses a **dual license** model:

| Edition | License | Audience | Price |
|---------|---------|----------|-------|
| **Community** | AGPL-3.0 | Self-hosters, open source projects, evaluation | Free |
| **Enterprise** | Commercial | Companies that cannot use AGPL or need enterprise features | Paid |

**Why AGPL for Community, not Apache-2.0:**
- Protects against cloud SaaS competitors taking the code, hosting it, and selling it without contributing back
- Consistent with upstream Word Office licensing (AGPL-3.0)
- AGPL requires network use to trigger source disclosure — critical for a document server
- Commercial license provides an escape hatch for companies that find AGPL unacceptable

**Commercial license grants (that AGPL does not):**
- Right to use without AGPL network-use obligations (no need to open-source derivative SaaS products)
- Access to enterprise-only features (digital signatures, compliance, analytics, etc.)
- Priority support and SLA
- Indemnification and IP protection
- White-labeling rights

### 12.2 Code Organization Principle

**All code lives in one monorepo.** Enterprise features are gated by license, not by repository.

```
┌─────────────────────────────────────────────────────────┐
│                   world-office/ (monorepo)                │
│                                                          │
│  ┌──────────────────────────┐  ┌─────────────────────┐  │
│  │  AGPL-3.0 code           │  │  Commercial code     │  │
│  │  (open source)           │  │  (enterprise only)   │  │
│  │                          │  │                     │  │
│  │  core/                   │  │  core-enterprise/    │  │
│  │  services/              │  │  web-enterprise/     │  │
│  │  apps/web/              │  │  packages/           │  │
│  │  integrations/          │  │   enterprise-features/│  │
│  │  packages/ (base)       │  │   document-builder/  │  │
│  │                          │  │                     │  │
│  │  Build produces:        │  │  Build produces:     │  │
│  │  world-office           │  │  world-office-       │  │
│  │  (community edition)    │  │  enterprise         │  │
│  └──────────────────────────┘  └─────────────────────┘  │
│                                                          │
│  ┌──────────────────────────────────────────────────┐    │
│  │  Shared code (both editions)                      │    │
│  │  packages/ui-kit/, packages/editor-core/, etc.   │    │
│  └──────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

### 12.3 Feature Gating Mechanism

Enterprise features use **compile-time feature flags** (not runtime checks):

```toml
# core-enterprise/Cargo.toml
[features]
default = []  # No enterprise features without license

digital-signatures = ["dep:wo-pdf"]
redaction = ["dep:wo-pdf", "dep:wo-renderer"]
drm = []
watermark = ["dep:wo-renderer"]
comparison = ["dep:wo-ooxml"]
converter-pro = ["dep:wo-x2t"]
```

```rust
// In server code — enterprise features are optional deps
#[cfg(feature = "enterprise")]
use eo_enterprise::digital_signature::{sign_document, verify_signature};

pub fn convert_document(input: &Path) -> Result<()> {
    let doc = eo_x2t::convert(input)?;
    #[cfg(feature = "enterprise")]
    if needs_signing { sign_document(&doc)?; }
    Ok(())
}
```

**Frontend feature gating:**
```typescript
// packages/enterprise-features/src/license.ts
export function useEnterpriseFeature(feature: string): boolean {
  return isEnterpriseLicense && enabledFeatures.has(feature);
}
```

### 12.4 Build Variants

```
turbo run build:community    # → world-office (AGPL, no enterprise features)
turbo run build:enterprise   # → world-office-enterprise (AGPL + commercial features)
```

### 12.5 License Headers

**AGPL code:**
```
SPDX-FileCopyrightText: 2026 World-Office Contributors
SPDX-License-Identifier: AGPL-3.0-or-later

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published
by the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.
```

**Commercial code:**
```
SPDX-FileCopyrightText: 2026 World-Office Contributors
SPDX-License-Identifier: LicenseRef-World-Office-Commercial

This software is licensed under the World-Office Commercial License.
See LICENSE-COMMERCIAL for terms. Use without a valid license
agreement is strictly prohibited.
```

### 12.6 CLA (Contributor License Agreement)

All contributors must sign a CLA granting:
- World-Office the right to distribute under both AGPL-3.0 and Commercial licenses
- Copyright to their contributions
- A patent grant for any patents in their contributions

---

## 13. Enterprise Components

### 13.1 Category 1: Advanced Security

| Feature | Crate/Package | Description | Priority |
|---------|--------------|-------------|----------|
| **Digital Signatures** | `core-enterprise/wo-digital-signature` | PAdES (PDF), XAdES (OOXML), document timestamping. Integrates with hardware tokens (eIDAS, FIDO2). | P0 |
| **Document Encryption** | `core-enterprise/wo-drm` | AES-256 encryption at rest, password protection, certificate-based encryption. Integrates with Azure Key Vault, AWS KMS, HashiCorp Vault. | P0 |
| **Redaction** | `core-enterprise/wo-redaction` | Content redaction with black bars, content removal, metadata scrubbing. Pattern-based (SSN, credit cards, custom). | P1 |
| **Watermarking** | `core-enterprise/wo-watermark` | Dynamic watermarks (user, date, custom text, image). Visible and hidden (steganographic) modes. Applied on view, export, print. | P1 |
| **Document Comparison** | `core-enterprise/wo-comparison` | Legal redline comparison. Character-level diffs, style changes, image/table comparison. Accept/reject per change. | P2 |

### 13.2 Category 2: Admin & Compliance

| Feature | Package | Description | Priority |
|---------|---------|-------------|----------|
| **Compliance Dashboard** | `web-enterprise/compliance-dashboard` | Central audit log viewer. Who did what, when, from where. GDPR/HIPAA/SOX export. Data retention policies. | P0 |
| **Advanced Audit Logging** | `services/audit-service` (new) | Immutable append-only audit log (PostgreSQL + S3). Tamper-evident (hash chain). Per-user action log levels. | P0 |
| **SSO (SAML)** | `services/identity-service` extension | SAML 2.0 IdP federation. Shibboleth, Okta, Azure AD, OneLogin. SAML is enterprise-only. | P0 |
| **SCIM Provisioning** | `services/scim-service` (new) | Auto-provision/deprovision users from IdP. Group sync. Azure AD SCIM, Okta SCIM. | P1 |
| **DLP Dashboard** | `web-enterprise/compliance-dashboard` | Data loss prevention. Detect sensitive data (PII, financial, health). Policy-based alerts and blocks. | P2 |

### 13.3 Category 3: Collaboration

| Feature | Package | Description | Priority |
|---------|---------|-------------|----------|
| **Review Workflows** | `web-enterprise/review-workflows` | Approval chains: draft → review → approve → publish. Role-based reviewers. Multi-stage reviews. | P1 |
| **Advanced Change Tracking** | `core-enterprise/wo-comparison` extension | Granular per-author change tracking. Accept/reject per-change. Author attribution. | P2 |
| **Comment & Annotation** | `apps/web/` extension | Inline comments anchored to content. Comment threads, mentions, resolution tracking. | P2 |
| **Version History Pro** | `services/storage-service` extension | Unlimited version history with delta storage (AGPL: configurable). Point-in-time restore. | P1 |

### 13.4 Category 4: Integration & Automation

| Feature | Package | Description | Priority |
|---------|---------|-------------|----------|
| **Document Builder** | `packages/document-builder` | Template engine with mail merge. Connect to CRM/ERP data. Batch generation. Conditional logic. | P0 |
| **Webhook & API Automation** | `services/webhook-service` (new) | Outgoing webhooks on document events. Configurable per collection. Retry with backoff. | P1 |
| **CRM Connectors** | `integrations/enterprise/crm/` | Salesforce, HubSpot, Pipedrive connectors. Auto-create docs from CRM data. | P2 |
| **ERP Connectors** | `integrations/enterprise/erp/` | SAP, Microsoft Dynamics, Odoo connectors. Invoice generation from ERP. | P2 |
| **Bulk Conversion Pro** | `core-enterprise/wo-converter-pro` | Priority queue. Batch 1000+ documents. Advanced format support. Conversion templates. | P1 |
| **Headless API** | `services/api-gateway` extension | API-only mode. Rate limits per API key. Usage metering. | P1 |

### 13.5 Category 5: Analytics & Branding

| Feature | Package | Description | Priority |
|---------|---------|-------------|----------|
| **Usage Analytics** | `web-enterprise/analytics-dashboard` | Per-user, per-document, per-team metrics. Active time, edit frequency, collaboration patterns. | P1 |
| **Document Insights** | `web-enterprise/analytics-dashboard` | AI-powered: readability, sentiment, key topics, complexity. | P2 |
| **White-Labeling** | `packages/enterprise-features` | Custom logo, colors, product name, favicon, login page. Per-tenant branding. | P1 |
| **Custom Themes** | `packages/ui-kit` extension | Dark/light/custom theme support. Theme editor for admins. | P2 |
| **SSO Branding** | `apps/desktop/` extension | Custom login screen, branded desktop icon, custom About dialog. | P2 |

### 13.6 Enterprise Services (new)

| Service | Data Store | Why Separate |
|---------|-----------|-------------|
| **Audit Service** | PostgreSQL (immutable log) + S3 (archive) | Security-critical, needs separate access controls and retention |
| **SCIM Service** | PostgreSQL (sync state) | User provisioning runs on different schedule |
| **Webhook Service** | PostgreSQL (subscriptions) + Redis (delivery queue) | Webhook delivery needs retry logic, dead letter queues |

---

## 14. Enterprise Timeline Integration

| Phase | Community Work | Enterprise Work (parallel) |
|-------|----------------|---------------------------|
| **Phase 0** | Foundation, discovery | CLA system, license validation, commercial license key management |
| **Phase 1** | Server microservices | Audit Service, SAML in Identity Service |
| **Phase 2** | Small formats | Digital signatures, Document encryption |
| **Phase 3** | Medium formats | Redaction, Watermarking |
| **Phase 4** | Web UI rewrite | Enterprise admin panel, Feature flag runtime, License validation UI |
| **Phase 5** | Large formats | Document comparison, Bulk conversion pro |
| **Phase 6** | Rendering engine | Watermark rendering, redaction overlays |
| **Phase 7** | Desktop Tauri | Enterprise branding, SCIM provisioning |
| **Phase 8** | WASM + Offline | Offline encryption (secure enclave) |
| **Phase 9** | Integrations | CRM/ERP connectors, Webhooks, Headless API, Document Builder |
| **Phase 10** | Launch | Enterprise beta, pricing, license fulfillment |

### Revenue Model (for planning)

| Tier | Monthly Price | Features |
|------|-------------|----------|
| **Starter** | $5/user | AGPL without obligations, email support, SSO (OIDC) |
| **Professional** | $15/user | All Starter + Digital Signatures, Redaction, Review Workflows, Version History Pro |
| **Business** | $25/user | All Professional + DLP, CRM/ERP Connectors, Document Builder, Priority Support |
| **Enterprise** | Custom | All Business + White-Labeling, Custom Themes, SLA, Dedicated Support, On-Premise |
