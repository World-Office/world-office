# Test Suite E2E — ownCloud Integration Test

## TL;DR

> **Quick Summary**: Create a new testsuite repo at `codeberg.org/Word-Office/testsuite` with an end-to-end test that spins up OCIS + Document Server (built from our core/ fork) + word-office-opencloud companion via Docker Compose, then validates health checks, API endpoints, and WOPI discovery.
> 
> **Deliverables**:
> - New Codeberg repository `Word-Office/testsuite`
> - Jest-based E2E test suite with Docker Compose orchestration
> - Forgejo Actions CI workflow for automated test runs
> - Document Server Docker image built from Word-Office core/ fork
> - Test scenarios: service health, API endpoints, WOPI discovery XML, OCIS accessibility
> 
> **Estimated Effort**: Large
> **Parallel Execution**: YES - 4 waves
> **Critical Path**: Task 1 (repo) → Task 3 (DS Docker image) → Task 6 (Docker Compose stack) → Task 7 (E2E tests) → Task 9 (CI) → Task 10 (README+push)

---

## Context

### Original Request
"setup a testsuite (https://codeberg.org/Word-Office/testsuite) and test all components end2end (at least start planning it and then, as first task, create a opencoud setup that tests the Word-Office integration end2end!"

### Interview Summary
**Key Discussions**:
- **Test framework**: User chose Jest (consistent with server/ repo)
- **Document Server image**: User chose to build from our core/ fork despite the complexity
- **CI integration**: User wants Forgejo Actions from the start (not local-only)
- **WOPI test depth**: Health checks + WOPI Discovery first; document editing deferred
- **Building from fork**: User confirmed "Build from fork anyway" when warned about multi-hour C++ builds

**Research Findings**:
- server/ has Jest test suite + Forgejo CI workflows (lint, unit, postgres, mysql tests)
- word-office-opencloud has setup wizard, dashboard, health API, Docker Compose generation, WOPI config
- DocumentServer/ has deployment configs (Nginx, systemd)
- document-server-integration/ has example docker-compose.yml
- sdkjs repo is EMPTY (submodule not initialized) — build will require submodule init or alternative approach
- docker-ci/ has base Docker images: Ubuntu 24.04, Node 20, JDK 21, Grunt CLI
- core/ build system is CMake (C++)

### Metis Review
Self-review performed (Metis agent timed out). Key gaps identified and addressed:
- **Gap: Building DS from core/ is very complex** → Addressed with dedicated Task 3, explicit build steps, and fallback strategy
- **Gap: sdkjs is empty** → Addressed in Task 3 with submodule init step
- **Gap: Test timeout** → Addressed with per-test timeouts and Docker health wait strategies

---

## Work Objectives

### Core Objective
Create a testsuite repository with automated E2E tests that validate the full Word-Office ownCloud integration stack: OCIS + Document Server + word-office-opencloud companion, connected via WOPI protocol.

### Concrete Deliverables
- Codeberg repo `Word-Office/testsuite` with proper README, .gitignore, package.json
- Docker Compose file that spins up: OCIS, Document Server (from core/), word-office-opencloud companion
- Document Server Dockerfile that builds from our core/ fork
- Jest test suite covering: service health, API endpoints, WOPI discovery
- Forgejo Actions workflow for CI
- Setup scripts for local development

### Definition of Done
- [ ] `docker compose up` starts all services (OCIS, DS, companion)
- [ ] All services pass health checks within 10 minutes
- [ ] `npm test` passes all E2E test scenarios
- [ ] CI pipeline runs on push and passes
- [ ] Tests validate: health endpoints, WOPI discovery XML, API responses

### Must Have
- Docker Compose test stack with OCIS + Document Server + companion
- Health check tests for all services
- WOPI discovery endpoint test
- Companion API endpoint tests (/api/health, /api/config, /api/health/wopi)
- Forgejo Actions CI workflow
- Tests runnable both locally and in CI
- Document Server built from Word-Office core/ fork

### Must NOT Have (Guardrails)
- NO document editing tests (deferred to follow-up)
- NO Nextcloud integration tests (out of scope)
- NO performance/load tests (out of scope)
- NO modifications to existing repos (only create new testsuite repo)
- NO use of upstream `Word Office/documentserver` image — must build from our fork
- NO hardcoded secrets in test files — use env vars or generated values
- NO tests that require manual human intervention
- NO modification of docs/DESIGN.md or .sisyphus/ plan files

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed. No exceptions.

### Test Decision
- **Infrastructure exists**: NO (new repo)
- **Automated tests**: YES (TDD approach — tests define the contract, Docker stack satisfies it)
- **Framework**: Jest (matches server/ repo convention)
- **Test runner**: `npm test` via package.json scripts

### QA Policy
Every task MUST include agent-executed QA scenarios.
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

- **Docker services**: Use Bash (curl) — Hit health endpoints, assert status codes + response fields
- **Companion API**: Use Bash (curl) — Test setup wizard, config endpoint, health endpoint
- **WOPI protocol**: Use Bash (curl) — Fetch discovery XML, validate structure
- **CI pipeline**: Use Bash (git push) — Verify Forgejo Actions triggers and passes

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately — repo setup + Dockerfile):
├── Task 1: Create testsuite repo on Codeberg + local clone [quick]
├── Task 2: Project scaffolding (package.json, Jest config, .gitignore) [quick]
└── Task 3: Document Server Dockerfile (build from core/) [deep]

Wave 2 (After Wave 1 — Docker Compose + companion config):
├── Task 4: OCIS test configuration (env vars, JWT secrets) [quick]
├── Task 5: Companion app test configuration [quick]
└── Task 6: Docker Compose test stack (OCIS + DS + companion) [unspecified-high]

Wave 3 (After Wave 2 — E2E tests):
├── Task 7: Health check E2E tests (all services) [deep]
├── Task 8: WOPI discovery + API endpoint tests [deep]
└── Task 9: Forgejo Actions CI workflow [unspecified-high]

Wave 4 (After Wave 3 — documentation + finalization):
├── Task 10: README + push to Codeberg + create PR [writing]

Wave FINAL (After ALL tasks — verification):
├── F1: Plan compliance audit [oracle]
├── F2: Code quality review [unspecified-high]
├── F3: Real manual QA [unspecified-high]
└── F4: Scope fidelity check [deep]
→ Present results → Get explicit user okay

Critical Path: Task 1 → Task 3 → Task 6 → Task 7 → Task 9 → Task 10 → F1-F4
Parallel Speedup: ~50% faster than sequential
Max Concurrent: 3 (Wave 1)
```

### Dependency Matrix

| Task | Depends On | Blocks | Wave |
|------|-----------|--------|------|
| 1 | — | 2, 3, 4, 5, 6 | 1 |
| 2 | 1 | 4, 5, 6 | 1 |
| 3 | 1 | 6 | 1 |
| 4 | 2 | 6 | 2 |
| 5 | 2 | 6 | 2 |
| 6 | 3, 4, 5 | 7, 8 | 2 |
| 7 | 6 | 9 | 3 |
| 8 | 6, 7 | 9 | 3 |
| 9 | 7, 8 | 10 | 3 |
| 10 | 9 | F1-F4 | 4 |

### Agent Dispatch Summary

- **Wave 1**: 3 tasks — T1 → `quick`, T2 → `quick`, T3 → `deep`
- **Wave 2**: 3 tasks — T4 → `quick`, T5 → `quick`, T6 → `unspecified-high`
- **Wave 3**: 3 tasks — T7 → `deep`, T8 → `deep`, T9 → `unspecified-high`
- **Wave 4**: 1 task — T10 → `writing`
- **FINAL**: 4 tasks — F1 → `oracle`, F2 → `unspecified-high`, F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

- [x] 1. Create testsuite repository on Codeberg + local clone

  **What to do**:
  - Create repo `Word-Office/testsuite` via Codeberg API using token `2874a6ea3d58e33b1a86067290d623150af74857`
  - Set description to "End-to-end test suite for Word-Office integration components"
  - Set visibility to public
  - Clone to `C:\Users\Tobias\git\Word Office\testsuite` via SSH
  - Initialize with a `main` branch and initial commit

  **Must NOT do**:
  - Do NOT create the repo via web UI — use API
  - Do NOT use HTTPS clone — use SSH (`git@codeberg.org:Word-Office/testsuite.git`)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Single API call + git clone, straightforward operations
  - **Skills**: [`git-master`]
    - `git-master`: Git operations for clone, init, remote setup
  - **Skills Evaluated but Omitted**:
    - `playwright`: No browser interaction needed

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 2 if local dir pre-created)
  - **Parallel Group**: Wave 1
  - **Blocks**: Tasks 2, 3, 4, 5, 6
  - **Blocked By**: None (can start immediately)

  **References**:

  **Pattern References** (existing code to follow):
  - `server/.forgejo/workflows/` — Existing Forgejo CI workflows for reference on how CI is structured in this project

  **API/Type References** (contracts to implement against):
  - Codeberg API: `POST /api/v1/user/repos` with `{"name": "testsuite", "description": "...", "private": false, "auto_init": true, "default_branch": "main"}`
  - Base URL: `https://codeberg.org`
  - Auth header: `Authorization: token 2874a6ea3d58e33b1a86067290d623150af74857`

  **WHY Each Reference Matters**:
  - The existing CI workflows show the Forgejo Actions YAML structure and runner image conventions used in this project

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Repository exists on Codeberg
    Tool: Bash (curl)
    Preconditions: Codeberg API token is valid
    Steps:
      1. curl -s -H "Authorization: token 2874a6ea3d58e33b1a86067290d623150af74857" https://codeberg.org/api/v1/repos/Word-Office/testsuite
      2. Assert HTTP status 200
      3. Assert response JSON contains "name": "testsuite"
      4. Assert "private" is false
    Expected Result: HTTP 200 with correct repo metadata
    Failure Indicators: 404 (repo not created), 401 (token invalid), "private": true
    Evidence: .sisyphus/evidence/task-1-repo-exists.txt

  Scenario: Local clone is valid git repo
    Tool: Bash
    Preconditions: Repo exists on Codeberg
    Steps:
      1. git -C C:\Users\Tobias\git\Word Office\testsuite status
      2. Assert exit code 0
      3. git -C C:\Users\Tobias\git\Word Office\testsuite remote -v
      4. Assert output contains "codeberg.org:Word-Office/testsuite"
    Expected Result: Clean git repo with Codeberg remote
    Failure Indicators: Non-zero exit, missing remote
    Evidence: .sisyphus/evidence/task-1-local-clone.txt

  Scenario: Repo is empty / has initial commit
    Tool: Bash
    Preconditions: Clone exists
    Steps:
      1. git -C C:\Users\Tobias\git\Word Office\testsuite log --oneline
      2. Assert at least 1 commit exists
    Expected Result: At least the initial commit present
    Failure Indicators: "fatal: your current branch does not have any commits yet"
    Evidence: .sisyphus/evidence/task-1-initial-commit.txt
  ```

  **Commit**: YES
  - Message: `chore: initialize testsuite repository`
  - Files: (auto-generated by Codeberg init)
  - Pre-commit: none

- [x] 2. Project scaffolding (package.json, Jest config, .gitignore, directory structure)

  **What to do**:
  - Create `package.json` with:
    - name: `@Word Office/testsuite`
    - scripts: `test` (jest), `test:e2e` (jest --testPathPattern=tests/e2e), `test:health` (jest --testPathPattern=tests/e2e/health), `lint` (eslint)
    - dependencies: jest, @jest/globals, dotenv, axios, dockerode (for Docker API), wait-on (for service readiness)
    - devDependencies: eslint
    - engines: node >= 20.0.0
  - Create `jest.config.js` with:
    - testEnvironment: node
    - testTimeout: 300000 (5 minutes — Docker startup is slow)
    - setupFilesAfterSetup: ['./tests/setup.js'] (env loading, global config)
    - testMatch: ['**/tests/**/*.test.js']
    - verbose: true
    - forceExit: true (Docker connections may keep process alive)
  - Create `.gitignore` with node_modules/, .env, *.log, coverage/, .DS_Store
  - Create `tests/setup.js` — loads .env.test, exports shared config (URLs, timeouts, secrets)
  - Create directory structure:
    ```
    testsuite/
    ├── tests/
    │   ├── setup.js
    │   ├── e2e/
    │   │   ├── health/
    │   │   │   └── .gitkeep
    │   │   ├── wopi/
    │   │   │   └── .gitkeep
    │   │   └── api/
    │   │       └── .gitkeep
    │   └── helpers/
    │       └── docker.js
    ├── docker/
    │   └── documentserver/
    │       └── .gitkeep
    ├── jest.config.js
    ├── package.json
    ├── .env.test
    ├── .gitignore
    └── .forgejo/
        └── workflows/
            └── .gitkeep
    ```
  - Create `.env.test` with test configuration defaults:
    - OCIS_URL=http://localhost:9200
    - DOCUMENT_SERVER_URL=http://localhost:8080
    - COMPANION_URL=http://localhost:3000
    - OCIS_JWT_SECRET=<generated 32+ char string>
    - DOCUMENT_SERVER_JWT_SECRET=<generated 32+ char string>
    - DOCKER_COMPOSE_FILE=docker-compose.test.yml
    - HEALTH_CHECK_TIMEOUT=120000 (2 min wait per service)
    - GLOBAL_TIMEOUT=600000 (10 min total test suite)
  - Create `tests/helpers/docker.js` — utility functions:
    - `waitForService(url, timeout)` — poll URL until 200 or timeout
    - `getContainerHealth(containerName)` — check Docker container health
    - `runDockerCompose(command)` — execute docker compose commands
    - `isServiceReady(serviceName)` — check if all services are up

  **Must NOT do**:
  - Do NOT install dependencies via `npm install` (may fail without full node_modules context) — write package.json manually
  - Do NOT use `export` command (PowerShell)
  - Do NOT include real secrets — use generated placeholders

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: File creation only, no complex logic
  - **Skills**: [`git-master`]
    - `git-master`: For commit operations
  - **Skills Evaluated but Omitted**:
    - `playwright`: No browser needed

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 1 — but needs repo to exist first)
  - **Parallel Group**: Wave 1 (after Task 1 completes)
  - **Blocks**: Tasks 4, 5, 6
  - **Blocked By**: Task 1

  **References**:

  **Pattern References** (existing code to follow):
  - `server/package.json` — Package structure, Jest configuration, scripts naming
  - `server/jest.config.js` — If it exists, shows existing Jest patterns
  - `server/test/` or `server/tests/` — Test file organization patterns

  **WHY Each Reference Matters**:
  - server/ is the only repo with a working Jest setup — copy its conventions for consistency

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Package.json is valid JSON with correct structure
    Tool: Bash (node)
    Preconditions: testsuite/ directory exists
    Steps:
      1. node -e "const p = require('C:/Users/Tobias/git/Word Office/testsuite/package.json'); console.log(p.name, p.scripts.test)"
      2. Assert output contains "@Word Office/testsuite" and "jest"
    Expected Result: Package name and test script present
    Failure Indicators: Parse error, missing fields
    Evidence: .sisyphus/evidence/task-2-package.json

  Scenario: Jest config is valid
    Tool: Bash (node)
    Preconditions: jest.config.js exists
    Steps:
      1. node -e "const c = require('C:/Users/Tobias/git/Word Office/testsuite/jest.config.js'); console.log(c.testTimeout)"
      2. Assert output contains "300000"
    Expected Result: testTimeout = 300000
    Failure Indicators: Parse error, wrong timeout value
    Evidence: .sisyphus/evidence/task-2-jest-config.txt

  Scenario: Directory structure complete
    Tool: Bash
    Preconditions: All files created
    Steps:
      1. Test each directory exists: tests/e2e/health/, tests/e2e/wopi/, tests/e2e/api/, tests/helpers/, docker/documentserver/, .forgejo/workflows/
      2. Assert all return true
    Expected Result: All directories exist with .gitkeep files
    Failure Indicators: Missing directory
    Evidence: .sisyphus/evidence/task-2-directory-structure.txt

  Scenario: .env.test has no real secrets
    Tool: Bash (grep)
    Preconditions: .env.test created
    Steps:
      1. Search for any JWT_SECRET value that looks like a real key (longer than 60 chars with mixed case)
      2. Assert none found
    Expected Result: Only placeholder/generated values
    Failure Indicators: Real secret detected
    Evidence: .sisyphus/evidence/task-2-no-secrets.txt
  ```

  **Commit**: YES
  - Message: `feat(testsuite): add project scaffolding with Jest config`
  - Files: package.json, jest.config.js, .gitignore, .env.test, tests/setup.js, tests/helpers/docker.js, all .gitkeep files
  - Pre-commit: `node -e "JSON.parse(require('fs').readFileSync('package.json'))"` (validate JSON)

- [x] 3. Document Server Dockerfile (build from core/ fork)

  **What to do**:
  - Create `docker/documentserver/Dockerfile` that builds Document Server from `Word-Office/core/`
  - This is a multi-stage build:
    - **Stage 1 (Builder)**: Use `ubuntu:24.04` base, install build deps (build-essential, cmake, qtbase5-dev, libboost-all-dev, etc.), clone & build core/
    - **Stage 2 (Runtime)**: Use `ubuntu:24.04`, copy built artifacts, install runtime deps (nginx, nodejs, postgresql, rabbitmq), configure Document Server services
  - Key build steps for core/:
    - `git init && git remote add origin https://codeberg.org/Word-Office/core.git`
    - `git fetch origin && git checkout main`
    - Initialize submodules: `git submodule update --init --recursive` (critical: sdkjs is a submodule)
    - `cmake -B build -DCMAKE_BUILD_TYPE=Release`
    - `cmake --build build -j$(nproc)`
  - Expose port 8080 (HTTP) for Document Server
  - Add HEALTHCHECK: `curl -f http://localhost:8080/hosting/discovery || exit 1`
  - Set JWT_SECRET via environment variable (injected at runtime)
  - Also create `docker/documentserver/build.sh` — a helper script that:
    - Handles the submodule init for sdkjs
    - Provides build progress logging
    - Supports incremental builds via Docker cache

  **CRITICAL NOTE**: The sdkjs submodule is currently EMPTY in the workspace. The Dockerfile MUST handle this by:
  1. Cloning core/ fresh from Codeberg (not copying from workspace)
  2. Running `git submodule update --init --recursive` to fetch sdkjs
  3. If that fails, clone sdkjs separately from `https://codeberg.org/Word-Office/sdkjs.git`

  **Must NOT do**:
  - Do NOT copy core/ from the local workspace (it's a shallow clone with empty submodules)
  - Do NOT use `Word Office/documentserver:latest` — must build from our fork
  - Do NOT hardcode JWT secrets — use ARG/ENV for runtime injection
  - Do NOT skip the sdkjs submodule — it's required for JS editor components

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Complex multi-stage Dockerfile with C++ build, submodule handling, and multiple failure modes
  - **Skills**: [`git-master`]
    - `git-master`: Git submodule operations, cloning strategies
  - **Skills Evaluated but Omitted**:
    - `playwright`: No browser interaction

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 1, 2)
  - **Parallel Group**: Wave 1
  - **Blocks**: Task 6
  - **Blocked By**: Task 1 (needs repo to commit to)

  **References**:

  **Pattern References** (existing code to follow):
  - `docker-ci/ds-base/Dockerfile` — Base Docker image for Document Server (Ubuntu 24.04, Node 20, JDK 21)
  - `DocumentServer/` — Deployment configuration (Nginx, systemd services, what gets installed)
  - `document-server-integration/docker-compose.yml` — Example Docker Compose using Document Server

  **API/Type References** (contracts to implement against):
  - `core/CMakeLists.txt` — CMake build configuration, dependencies, build targets
  - `core/.gitmodules` — Submodule definitions (sdkjs path)
  - `word-office-opencloud/lib/compose.js` — Expected Document Server service config (port, env vars, volumes)

  **External References** (libraries and frameworks):
  - Word Office Document Server build docs: https://api.Word Office.com/docs/docs-api/additional-api-configuration/document-server-integration/
  - Docker multi-stage build: https://docs.docker.com/build/building/multi-stage/

  **WHY Each Reference Matters**:
  - `docker-ci/ds-base/Dockerfile` shows exactly what base packages are needed for the build
  - `core/CMakeLists.txt` defines the actual build targets and dependencies
  - `word-office-opencloud/lib/compose.js` defines what ports and env vars the companion expects

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Dockerfile syntax is valid
    Tool: Bash (docker)
    Preconditions: Docker is running
    Steps:
      1. docker build --check docker/documentserver/ 2>&1 || echo "Syntax check not supported, falling back to dry-run parse"
      2. Verify Dockerfile has FROM, RUN, COPY, EXPOSE, HEALTHCHECK directives
    Expected Result: No syntax errors in Dockerfile
    Failure Indicators: "unknown instruction", "parse error"
    Evidence: .sisyphus/evidence/task-3-dockerfile-syntax.txt

  Scenario: Dockerfile uses our core/ fork
    Tool: Bash (grep)
    Preconditions: Dockerfile exists
    Steps:
      1. grep -i "codeberg.org/Word-Office/core" docker/documentserver/Dockerfile
      2. Assert match found
      3. grep -i "Word Office/documentserver" docker/documentserver/Dockerfile
      4. Assert NO match (not using upstream image)
    Expected Result: References Word-Office/core, NOT upstream image
    Failure Indicators: Upstream image reference found
    Evidence: .sisyphus/evidence/task-3-fork-reference.txt

  Scenario: Dockerfile handles sdkjs submodule
    Tool: Bash (grep)
    Preconditions: Dockerfile exists
    Steps:
      1. grep -i "submodule" docker/documentserver/Dockerfile
      2. Assert match found
      3. grep -i "sdkjs" docker/documentserver/Dockerfile
      4. Assert match found
    Expected Result: Submodule init and sdkjs references present
    Failure Indicators: Missing submodule handling
    Evidence: .sisyphus/evidence/task-3-submodule-handling.txt

  Scenario: Dockerfile uses runtime JWT secret
    Tool: Bash (grep)
    Preconditions: Dockerfile exists
    Steps:
      1. grep "JWT_SECRET" docker/documentserver/Dockerfile
      2. Assert ENV or ARG for JWT_SECRET exists
      3. Assert NO hardcoded 32+ char secret strings
    Expected Result: JWT_SECRET is parameterized via ENV/ARG
    Failure Indicators: Hardcoded secret value
    Evidence: .sisyphus/evidence/task-3-jwt-parameterized.txt
  ```

  **Commit**: YES
  - Message: `feat(testsuite): add Document Server Dockerfile built from core/ fork`
  - Files: docker/documentserver/Dockerfile, docker/documentserver/build.sh
  - Pre-commit: `grep -q "codeberg.org/Word-Office/core" docker/documentserver/Dockerfile` (verify fork reference)

- [x] 4. OCIS test configuration (env vars, JWT secrets, Docker image)

  **What to do**:
  - Create `docker/ocis/.env` with OCIS test configuration:
    - `OCIS_DOMAIN=localhost`
    - `OCIS_URL=http://localhost:9200`
    - `OCIS_JWT_SECRET=<generated 44-char base64 string>`
    - `PROXY_TLS=false` (no TLS in test environment)
    - `STORAGE_HOME_DRIVER=owncloud`
    - `STORAGE_USERS_DRIVER=owncloud`
    - `AUTHENTICATION_HANDLERS=basic`
    - `IDM_CREATE_DEMO_USERS=true` (create test users: admin, testuser)
    - `OCIS_LOG_LEVEL=warn` (reduce log noise in tests)
    - `WEB_UI_CONFIG_FILE=/etc/ocis/web-ui.json`
  - Create `docker/ocis/web-ui.json` with WOPI handler configuration:
    - mimeTypeHandlers for office file extensions pointing to `/wopi`
    - Editor app name set to "Word-Office Document Server"
  - Pin OCIS Docker image to `owncloud/ocis:5.0` or latest stable (document the version choice)
  - Create a helper script `docker/ocis/generate-secrets.sh` that generates random JWT secrets for test env

  **Must NOT do**:
  - Do NOT use TLS in test config (HTTP only for simplicity)
  - Do NOT use production domains
  - Do NOT hardcode secrets in committed files — use placeholder or generated-in-script

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Config file creation, straightforward
  - **Skills**: []
  - **Skills Evaluated but Omitted**:
    - `git-master`: No git complexity

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 3, 5)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 6
  - **Blocked By**: Task 2

  **References**:

  **Pattern References** (existing code to follow):
  - `word-office-opencloud/lib/ocis-config.js` — OCIS configuration structure (web-ui.json format, env vars)
  - `word-office-opencloud/lib/config.js` — Required env vars and their defaults
  - `word-office-opencloud/.env.example` — Example .env with all config options

  **WHY Each Reference Matters**:
  - `ocis-config.js` defines the exact web-ui.json structure the companion generates — our test must match

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: OCIS env file has required variables
    Tool: Bash (grep)
    Preconditions: docker/ocis/.env exists
    Steps:
      1. grep "OCIS_DOMAIN" docker/ocis/.env && grep "OCIS_JWT_SECRET" docker/ocis/.env && grep "PROXY_TLS" docker/ocis/.env
      2. Assert all three found
    Expected Result: All required env vars present
    Failure Indicators: Missing required variable
    Evidence: .sisyphus/evidence/task-4-ocis-env.txt

  Scenario: web-ui.json is valid JSON with WOPI handler
    Tool: Bash (node)
    Preconditions: docker/ocis/web-ui.json exists
    Steps:
      1. node -e "const c=JSON.parse(require('fs').readFileSync('docker/ocis/web-ui.json','utf8')); console.log(c.editor.mimeTypeHandlers.length)"
      2. Assert output >= 1
    Expected Result: Valid JSON with at least one mimeTypeHandler
    Failure Indicators: Parse error, no handlers
    Evidence: .sisyphus/evidence/task-4-webui-json.txt

  Scenario: No TLS in test config
    Tool: Bash (grep)
    Preconditions: docker/ocis/.env exists
    Steps:
      1. grep "PROXY_TLS=false" docker/ocis/.env
      2. Assert match found
    Expected Result: PROXY_TLS explicitly set to false
    Failure Indicators: Missing or set to true
    Evidence: .sisyphus/evidence/task-4-no-tls.txt
  ```

  **Commit**: YES
  - Message: `feat(testsuite): add OCIS test configuration`
  - Files: docker/ocis/.env, docker/ocis/web-ui.json, docker/ocis/generate-secrets.sh
  - Pre-commit: `node -e "JSON.parse(require('fs').readFileSync('docker/ocis/web-ui.json'))"`

- [x] 5. Companion app test configuration (word-office-opencloud setup for testing)

  **What to do**:
  - Create `docker/companion/.env` with test configuration for the word-office-opencloud companion:
    - `PORT=3000`
    - `NODE_ENV=test`
    - `OCIS_DOMAIN=localhost`
    - `DOCUMENT_SERVER_DOMAIN=localhost`
    - `OCIS_JWT_SECRET=<same as OCIS config>`
    - `DOCUMENT_SERVER_JWT_SECRET=<generated>`
    - `OCIS_INTERNAL_PORT=9200`
    - `DOCUMENT_SERVER_INTERNAL_PORT=8080`
    - `OCIS_DATA_DIR=./data/ocis`
    - `DOCUMENT_SERVER_DATA_DIR=./data/documentserver`
    - `ENABLE_SSL=false` (no TLS in test)
    - `ENABLE_METRICS=false` (reduce noise)
    - `ENABLE_LOGS=true`
  - Create a test-specific Dockerfile `docker/companion/Dockerfile` for the companion:
    - Use `node:20-alpine` base
    - Copy the companion source from the local workspace (not from git clone)
    - Set up the .env from docker/companion/.env
    - Healthcheck on /api/health
    - Expose port 3000

  **Must NOT do**:
  - Do NOT modify the actual word-office-opencloud repo — all test config stays in testsuite/
  - Do NOT use production domains or real secrets
  - Do NOT enable SSL in test mode

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Config file creation, simple Dockerfile
  - **Skills**: []
  - **Skills Evaluated but Omitted**:
    - `playwright`: No browser needed

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 3, 4)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 6
  - **Blocked By**: Task 2

  **References**:

  **Pattern References** (existing code to follow):
  - `word-office-opencloud/.env.example` — All available config variables
  - `word-office-opencloud/Dockerfile` — Existing companion Dockerfile pattern
  - `word-office-opencloud/lib/config.js` — Required vs optional config, validation rules

  **WHY Each Reference Matters**:
  - The companion's config.js validates env vars at startup — our test .env must satisfy all validation rules

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Companion env satisfies config.js validation
    Tool: Bash (node)
    Preconditions: docker/companion/.env exists, word-office-opencloud/lib/config.js exists
    Steps:
      1. Load docker/companion/.env vars into environment
      2. Try to require word-office-opencloud/lib/config.js with those vars
      3. Assert no error thrown
    Expected Result: Config loads without validation errors
    Failure Indicators: "Missing required configuration", "must be at least 32 characters"
    Evidence: .sisyphus/evidence/task-5-companion-env.txt

  Scenario: Companion Dockerfile exists and references correct base
    Tool: Bash (grep)
    Preconditions: docker/companion/Dockerfile exists
    Steps:
      1. grep "node:20" docker/companion/Dockerfile
      2. Assert match found
      3. grep "HEALTHCHECK" docker/companion/Dockerfile
      4. Assert match found
    Expected Result: Node 20 base image and healthcheck present
    Failure Indicators: Missing base image or healthcheck
    Evidence: .sisyphus/evidence/task-5-companion-dockerfile.txt
  ```

  **Commit**: YES
  - Message: `feat(testsuite): add companion test configuration`
  - Files: docker/companion/.env, docker/companion/Dockerfile
  - Pre-commit: `node -e "JSON.parse(require('fs').readFileSync('docker/companion/.env').toString().split('\\n').filter(l=>!l.startsWith('#')&&l.includes('=')).reduce((o,l)=>{const[k,...v]=l.split('=');o[k.trim()]=v.join('=').trim();return o},{}))"` (validate env format)

- [x] 6. Docker Compose test stack (OCIS + Document Server + companion)

  **What to do**:
  - Create `docker-compose.test.yml` with three services:

    **Service: documentserver**
    - build: context = `./docker/documentserver/`
    - container_name: `test-documentserver`
    - ports: `8080:80`
    - environment: JWT_SECRET from .env.test, JWT_HEADER=Authorization, JWT_IN_BODY=true
    - healthcheck: `curl -f http://localhost:80/hosting/discovery || exit 1` (interval 10s, timeout 5s, retries 30)
    - volumes: documentserver-data (named volume for persistence within test run)
    - networks: test-network

    **Service: ocis**
    - image: `owncloud/ocis:latest`
    - container_name: `test-ocis`
    - ports: `9200:9200`
    - environment: all vars from docker/ocis/.env plus:
      - COLLABORATION_WOPI_SRC=http://localhost:9200
      - COLLABORATION_APP_ADDR=http://documentserver:80
      - COLLABORATION_APP_NAME=Word-Office Document Server
    - healthcheck: `curl -f http://localhost:9200/health || exit 1` (interval 10s, timeout 5s, retries 30)
    - volumes: ocis-data (named volume), mount web-ui.json
    - depends_on: documentserver (with condition: service_healthy)
    - networks: test-network

    **Service: companion**
    - build: context = `./docker/companion/`
    - container_name: `test-companion`
    - ports: `3000:3000`
    - environment: all vars from docker/companion/.env plus OCIS_INTERNAL_URL=http://ocis:9200, DOCUMENT_SERVER_INTERNAL_URL=http://documentserver:80
    - healthcheck: `curl -f http://localhost:3000/api/health || exit 1` (interval 10s, timeout 5s, retries 30)
    - depends_on: ocis (with condition: service_healthy)
    - networks: test-network

  - Named volumes: documentserver-data, ocis-data
  - Network: test-network (bridge driver)
  - Create `scripts/start-test-stack.sh` — starts Docker Compose, waits for all healthy
  - Create `scripts/stop-test-stack.sh` — stops and removes containers + volumes

  **Must NOT do**:
  - Do NOT use host networking (use bridge network for isolation)
  - Do NOT expose unnecessary ports
  - Do NOT use privileged mode
  - Do NOT set `restart: always` (test containers should stop when tests finish)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Complex Docker Compose with health checks, dependencies, volume mounts, and network config
  - **Skills**: [`git-master`]
    - `git-master`: For committing multi-file changes
  - **Skills Evaluated but Omitted**:
    - `playwright`: No browser needed

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 2 (sequential — needs all Wave 2 inputs)
  - **Blocks**: Tasks 7, 8
  - **Blocked By**: Tasks 3, 4, 5

  **References**:

  **Pattern References** (existing code to follow):
  - `document-server-integration/docker-compose.yml` — Example Document Server Docker Compose
  - `word-office-opencloud/lib/compose.js` — Docker Compose generation logic (service structure, labels, networks)
  - `DocumentServer/` — Deployment configuration reference

  **API/Type References** (contracts to implement against):
  - `docker/documentserver/Dockerfile` (from Task 3) — Build context
  - `docker/companion/Dockerfile` (from Task 5) — Build context
  - `docker/ocis/.env` (from Task 4) — OCIS env vars

  **External References**:
  - Docker Compose healthcheck syntax: https://docs.docker.com/compose/compose-file/05-services/#healthcheck
  - OCIS Docker image docs: https://owncloud.dev/ocis/deployment/container/

  **WHY Each Reference Matters**:
  - `compose.js` defines the expected Docker Compose structure — our test stack should mirror what the companion generates
  - The integration example shows proven Docker Compose patterns for Document Server

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Docker Compose file is valid YAML
    Tool: Bash (docker compose)
    Preconditions: docker-compose.test.yml exists
    Steps:
      1. docker compose -f docker-compose.test.yml config
      2. Assert exit code 0
    Expected Result: Valid Docker Compose configuration
    Failure Indicators: Parse error, validation error, missing service
    Evidence: .sisyphus/evidence/task-6-compose-valid.txt

  Scenario: All three services defined
    Tool: Bash (grep)
    Preconditions: docker-compose.test.yml exists
    Steps:
      1. grep "documentserver:" docker-compose.test.yml
      2. grep "ocis:" docker-compose.test.yml
      3. grep "companion:" docker-compose.test.yml
      4. Assert all three found
    Expected Result: All three services present
    Failure Indicators: Missing service definition
    Evidence: .sisyphus/evidence/task-6-services-defined.txt

  Scenario: Health checks defined for all services
    Tool: Bash (grep)
    Preconditions: docker-compose.test.yml exists
    Steps:
      1. grep -c "healthcheck:" docker-compose.test.yml
      2. Assert count is 3 (one per service)
    Expected Result: Three healthcheck blocks
    Failure Indicators: Fewer than 3 healthchecks
    Evidence: .sisyphus/evidence/task-6-healthchecks.txt

  Scenario: No restart: always policy
    Tool: Bash (grep)
    Preconditions: docker-compose.test.yml exists
    Steps:
      1. grep "restart: always" docker-compose.test.yml
      2. Assert NO match found
    Expected Result: No "restart: always" anywhere
    Failure Indicators: Match found
    Evidence: .sisyphus/evidence/task-6-no-restart.txt
  ```

  **Commit**: YES
  - Message: `feat(testsuite): add Docker Compose test stack with OCIS + DS + companion`
  - Files: docker-compose.test.yml, scripts/start-test-stack.sh, scripts/stop-test-stack.sh
  - Pre-commit: `docker compose -f docker-compose.test.yml config > /dev/null 2>&1`

- [x] 7. Health check E2E tests (all services)

  **What to do**:
  - Create `tests/e2e/health/documentserver.test.js`:
    - Test: Document Server container is running
    - Test: `/hosting/discovery` returns HTTP 200
    - Test: `/hosting/discovery` returns valid XML with `<wopi-discovery>` root element
    - Test: `/healthcheck` returns HTTP 200 (Document Server internal health)
    - Test: Document Server responds within 5 minutes of stack start
  - Create `tests/e2e/health/ocis.test.js`:
    - Test: OCIS container is running
    - Test: `http://localhost:9200/health` returns HTTP 200
    - Test: OCIS responds with JSON containing `"status"` field
    - Test: OCIS `/.well-known/openid-configuration` returns valid OIDC discovery
    - Test: OCIS responds within 5 minutes of stack start
  - Create `tests/e2e/health/companion.test.js`:
    - Test: Companion container is running
    - Test: `http://localhost:3000/api/health` returns HTTP 200
    - Test: Health response has `"overall": "healthy"` or `"degraded"` (not "down")
    - Test: Health response lists all expected services
    - Test: Companion responds within 2 minutes of OCIS being healthy
  - Create `tests/e2e/health/stack.test.js` — Full stack integration:
    - Test: All three services are simultaneously healthy
    - Test: Services can reach each other (companion → OCIS, companion → DS)
    - Test: No containers in restart loop

  Use `tests/helpers/docker.js` utilities for:
  - `waitForService(url, timeout)` — retry with exponential backoff
  - `getContainerHealth(name)` — Docker API health status

  **Must NOT do**:
  - Do NOT test document editing (out of scope)
  - Do NOT use Playwright (API-only tests, no browser)
  - Do NOT set test timeouts below 60s (Docker startup is slow)

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Test design requires understanding of all service health endpoints, Docker wait strategies, and failure modes
  - **Skills**: []
  - **Skills Evaluated but Omitted**:
    - `playwright`: API-only tests, no browser
    - `git-master`: No git complexity

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 8)
  - **Parallel Group**: Wave 3
  - **Blocks**: Task 9
  - **Blocked By**: Task 6

  **References**:

  **Pattern References** (existing code to follow):
  - `server/test/` or `server/tests/` — Jest test patterns used in this project
  - `server/.forgejo/workflows/unit-tests.yml` — Test runner configuration
  - `word-office-opencloud/lib/health.js` — Health check logic (what endpoints are checked, how)

  **API/Type References** (contracts to implement against):
  - Document Server health: `GET /hosting/discovery` (returns XML), `GET /healthcheck` (returns 200)
  - OCIS health: `GET /health` (returns JSON with status), `GET /.well-known/openid-configuration` (OIDC)
  - Companion health: `GET /api/health` (returns JSON with overall, services, timestamp)

  **WHY Each Reference Matters**:
  - `health.js` defines the exact health check logic — tests should verify the same endpoints
  - `server/test/` shows the Jest test patterns used in this project (assertions, mocking, structure)

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: All health test files are valid Jest tests
    Tool: Bash (node)
    Preconditions: Test files created
    Steps:
      1. node -e "const fs=require('fs'); ['documentserver.test.js','ocis.test.js','companion.test.js','stack.test.js'].forEach(f => { const c=fs.readFileSync('tests/e2e/health/'+f,'utf8'); if(!c.includes('describe(')||!c.includes('test(')) throw new Error(f+' missing Jest syntax'); }); console.log('PASS')"
      2. Assert exit code 0
    Expected Result: All files contain describe() and test() blocks
    Failure Indicators: Missing Jest syntax
    Evidence: .sisyphus/evidence/task-7-jest-syntax.txt

  Scenario: Tests use helper utilities
    Tool: Bash (grep)
    Preconditions: Test files created
    Steps:
      1. grep -r "waitForService" tests/e2e/health/
      2. Assert at least 1 match
      3. grep -r "helpers/docker" tests/e2e/health/
      4. Assert at least 1 match
    Expected Result: Helper utilities are imported and used
    Failure Indicators: No imports found
    Evidence: .sisyphus/evidence/task-7-helpers-used.txt

  Scenario: Test timeouts are reasonable for Docker
    Tool: Bash (grep)
    Preconditions: Test files created
    Steps:
      1. grep -r "timeout" tests/e2e/health/ (or check jest.config.js testTimeout)
      2. Assert timeout >= 60000ms somewhere in test config or individual tests
    Expected Result: At least 60s timeout for Docker operations
    Failure Indicators: No timeout config, or timeout < 60s
    Evidence: .sisyphus/evidence/task-7-timeouts.txt
  ```

  **Commit**: YES
  - Message: `feat(testsuite): add health check E2E tests for all services`
  - Files: tests/e2e/health/documentserver.test.js, tests/e2e/health/ocis.test.js, tests/e2e/health/companion.test.js, tests/e2e/health/stack.test.js
  - Pre-commit: `node -e "require('./jest.config.js')"` (validate Jest config loads)

- [x] 8. WOPI discovery + Companion API endpoint tests

  **What to do**:
  - Create `tests/e2e/wopi/discovery.test.js`:
    - Test: `GET /hosting/discovery` returns valid XML
    - Test: XML contains `<wopi-discovery>` root element
    - Test: XML contains at least one `<app>` element with `name` attribute
    - Test: XML contains `<action>` elements with `urlsrc` attributes
    - Test: Action URLs point to Document Server (localhost:8080)
    - Test: Discovery includes handlers for .docx, .xlsx, .pptx extensions
    - Test: Discovery response has correct Content-Type header (text/xml or application/xml)
  - Create `tests/e2e/api/companion.test.js`:
    - Test: `GET /api/health` returns 200 with valid JSON structure
    - Test: `GET /api/health` response has `overall`, `services`, `timestamp` fields
    - Test: `GET /api/config` returns sanitized config (no secrets)
    - Test: `GET /api/config` response has OCIS_DOMAIN, DOCUMENT_SERVER_DOMAIN but no JWT_SECRET
    - Test: `GET /api/health/wopi` returns WOPI connectivity status
    - Test: `POST /setup` with valid data returns redirect or 200
    - Test: `POST /setup` with missing OCIS_DOMAIN returns validation error
    - Test: `POST /setup` with invalid domain format returns validation error
  - Create `tests/e2e/api/ocis-wopi.test.js`:
    - Test: OCIS `/wopi` endpoint is accessible (WOPI host)
    - Test: WOPI discovery XML from DS is parseable by OCIS collaboration service
    - Test: OCIS collaboration service registers with Document Server

  **Must NOT do**:
  - Do NOT test actual document opening/editing (out of scope)
  - Do NOT use Playwright
  - Do NOT test with invalid JWT tokens (deferred to security testing)

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: WOPI protocol validation requires understanding of XML structure, API contracts, and service integration
  - **Skills**: []
  - **Skills Evaluated but Omitted**:
    - `playwright`: No browser needed

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 7)
  - **Parallel Group**: Wave 3
  - **Blocks**: Task 9
  - **Blocked By**: Task 6

  **References**:

  **Pattern References** (existing code to follow):
  - `server/test/` — Jest test patterns
  - `tests/e2e/health/stack.test.js` (from Task 7) — Similar test structure

  **API/Type References** (contracts to implement against):
  - WOPI Discovery spec: `GET /hosting/discovery` returns `<wopi-discovery>` XML
  - Companion API: `GET /api/health`, `GET /api/config`, `GET /api/health/wopi`, `POST /setup`
  - `word-office-opencloud/routes/api.js` — API endpoint handlers and response shapes
  - `word-office-opencloud/routes/setup.js` — Setup wizard validation logic

  **External References**:
  - WOPI protocol spec: https://learn.microsoft.com/en-us/microsoft-365/cloud-storage-partner-program/rest/
  - Word Office Document Server API: https://api.Word Office.com/docs/docs-api/additional-api-configuration/document-server-integration/

  **WHY Each Reference Matters**:
  - The WOPI discovery XML structure is standardized — tests must validate exact XML elements
  - The companion routes define the exact API contract — test assertions must match

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: WOPI test validates XML structure
    Tool: Bash (grep)
    Preconditions: tests/e2e/wopi/discovery.test.js exists
    Steps:
      1. grep "wopi-discovery" tests/e2e/wopi/discovery.test.js
      2. Assert match found
      3. grep "xml" tests/e2e/wopi/discovery.test.js
      4. Assert match found (XML parsing is present)
    Expected Result: Test contains WOPI XML validation
    Failure Indicators: Missing XML/WOPI references
    Evidence: .sisyphus/evidence/task-8-wopi-xml-test.txt

  Scenario: API tests cover all companion endpoints
    Tool: Bash (grep)
    Preconditions: tests/e2e/api/companion.test.js exists
    Steps:
      1. grep "/api/health" tests/e2e/api/companion.test.js
      2. grep "/api/config" tests/e2e/api/companion.test.js
      3. grep "/api/health/wopi" tests/e2e/api/companion.test.js
      4. grep "/setup" tests/e2e/api/companion.test.js
      5. Assert all four found
    Expected Result: All four API endpoints tested
    Failure Indicators: Missing endpoint test
    Evidence: .sisyphus/evidence/task-8-api-coverage.txt

  Scenario: Config test verifies no secrets leaked
    Tool: Bash (grep)
    Preconditions: tests/e2e/api/companion.test.js exists
    Steps:
      1. grep -i "secret" tests/e2e/api/companion.test.js
      2. Assert match found (test checks for secret absence)
    Expected Result: Test explicitly verifies secrets are not exposed
    Failure Indicators: No secret-checking assertion found
    Evidence: .sisyphus/evidence/task-8-no-secrets-test.txt
  ```

  **Commit**: YES
  - Message: `feat(testsuite): add WOPI discovery and companion API endpoint tests`
  - Files: tests/e2e/wopi/discovery.test.js, tests/e2e/api/companion.test.js, tests/e2e/api/ocis-wopi.test.js
  - Pre-commit: `node -c tests/e2e/wopi/discovery.test.js && node -c tests/e2e/api/companion.test.js`

- [x] 9. Forgejo Actions CI workflow

  **What to do**:
  - Create `.forgejo/workflows/e2e.yml` with:
    - **Trigger**: push to `main`, pull_request to `main`
    - **Runner**: `ubuntu-latest` (or Docker-in-Docker capable runner)
    - **Steps**:
      1. Checkout testsuite repo
      2. Set up Node.js 20
      3. Install dependencies: `npm ci`
      4. Checkout core/ fork (needed for DS Docker build): `git clone --depth=1 https://codeberg.org/Word-Office/core.git docker/documentserver/core`
      5. Initialize sdkjs submodule: `git submodule update --init --recursive` in core dir
      6. Build Docker images: `docker compose -f docker-compose.test.yml build`
      7. Start stack: `docker compose -f docker-compose.test.yml up -d`
      8. Wait for services: `./scripts/wait-for-stack.sh` (polls health endpoints, max 10 min)
      9. Run E2E tests: `npm test`
      10. Capture Docker logs on failure: `docker compose -f docker-compose.test.yml logs`
      11. Tear down: `docker compose -f docker-compose.test.yml down -v`
    - **Timeout**: 45 minutes total (DS build is slow)
    - **Artifacts**: Save test results XML, Docker logs as artifacts on failure
  - Create `scripts/wait-for-stack.sh`:
    - Polls all three service health endpoints
    - Reports progress every 30 seconds
    - Exits 0 when all healthy, exits 1 if timeout (10 min)
  - Create `.forgejo/workflows/lint.yml`:
    - **Trigger**: push, pull_request
    - **Steps**: Checkout, Node 20, npm ci, npm run lint
    - **Timeout**: 5 minutes

  **Must NOT do**:
  - Do NOT use Docker Hub images — build from fork
  - Do NOT cache Docker layers between runs (unreliable in CI)
  - Do NOT use `docker:stable-dind` as runner — use a proper CI runner

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: CI workflow design with Docker-in-Docker, multi-step build, and complex failure handling
  - **Skills**: [`git-master`]
    - `git-master`: Git submodule operations in CI
  - **Skills Evaluated but Omitted**:
    - `playwright`: No browser in CI

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on Tasks 7 + 8 for test file locations)
  - **Parallel Group**: Wave 3 (sequential after 7 and 8)
  - **Blocks**: Task 10
  - **Blocked By**: Tasks 7, 8

  **References**:

  **Pattern References** (existing code to follow):
  - `server/.forgejo/workflows/unit-tests.yml` — Existing Forgejo CI workflow structure
  - `server/.forgejo/workflows/lint-format.yml` — Lint workflow pattern
  - `word-office-nextcloud/.forgejo/workflows/` — Another Forgejo CI reference

  **API/Type References** (contracts to implement against):
  - Forgejo Actions YAML syntax (similar to GitHub Actions): https://forgejo.org/docs/latest/user/actions/
  - Docker Compose in CI: needs Docker socket or DinD setup

  **WHY Each Reference Matters**:
  - Existing workflows show the exact YAML structure Forgejo expects — triggers, runner labels, step syntax

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: CI workflow files are valid YAML
    Tool: Bash (python or node)
    Preconditions: .forgejo/workflows/*.yml created
    Steps:
      1. node -e "require('fs').readFileSync('.forgejo/workflows/e2e.yml','utf8'); console.log('e2e.yml: valid')"
      2. Parse with js-yaml or similar
      3. Assert no parse errors
    Expected Result: Both YAML files parse without errors
    Failure Indicators: YAML syntax error
    Evidence: .sisyphus/evidence/task-9-ci-yaml.txt

  Scenario: E2E workflow includes Docker build step
    Tool: Bash (grep)
    Preconditions: .forgejo/workflows/e2e.yml exists
    Steps:
      1. grep "docker compose" .forgejo/workflows/e2e.yml
      2. Assert at least 3 matches (build, up, down)
    Expected Result: Build, start, and teardown commands present
    Failure Indicators: Missing Docker Compose commands
    Evidence: .sisyphus/evidence/task-9-docker-steps.txt

  Scenario: Wait-for-stack script is executable
    Tool: Bash
    Preconditions: scripts/wait-for-stack.sh exists
    Steps:
      1. cat scripts/wait-for-stack.sh | head -1
      2. Assert shebang line (#!/bin/bash or #!/usr/bin/env bash)
    Expected Result: Script has proper shebang
    Failure Indicators: No shebang or wrong interpreter
    Evidence: .sisyphus/evidence/task-9-wait-script.txt

  Scenario: Workflow triggers on push and PR
    Tool: Bash (grep)
    Preconditions: .forgejo/workflows/e2e.yml exists
    Steps:
      1. grep "push:" .forgejo/workflows/e2e.yml
      2. grep "pull_request:" .forgejo/workflows/e2e.yml
      3. Assert both found
    Expected Result: Both triggers defined
    Failure Indicators: Missing trigger
    Evidence: .sisyphus/evidence/task-9-triggers.txt
  ```

  **Commit**: YES
  - Message: `ci(testsuite): add Forgejo Actions E2E workflow`
  - Files: .forgejo/workflows/e2e.yml, .forgejo/workflows/lint.yml, scripts/wait-for-stack.sh
  - Pre-commit: `node -e "require('js-yaml').load(require('fs').readFileSync('.forgejo/workflows/e2e.yml','utf8'))"` (validate YAML)

- [x] 10. README + push to Codeberg + create PR

  **What to do**:
  - Create `README.md` with:
    - Project title: `Word-Office Test Suite`
    - Description: End-to-end test suite for Word-Office integration components
    - Prerequisites: Docker 20.10+, Node.js 20+, Docker Compose 2.0+
    - Quick Start section:
      - `git clone git@codeberg.org:Word-Office/testsuite.git`
      - `npm install`
      - `docker compose -f docker-compose.test.yml build` (first time only, ~2-4 hours for DS)
      - `docker compose -f docker-compose.test.yml up -d`
      - `npm test`
    - Architecture diagram showing test flow: tests → companion → OCIS + DS
    - Test coverage section listing what's tested (health, WOPI discovery, API endpoints)
    - CI section explaining Forgejo Actions integration
    - Configuration section explaining .env.test
    - Troubleshooting section for common issues (Docker not running, ports in use, build failures)
    - Word-Office disclaimer (exact wording from established pattern)
    - License: AGPL-3.0
  - Push all commits to Codeberg via SSH: `git remote add ssh git@codeberg.org:Word-Office/testsuite.git && git push ssh main`
  - Verify all files are on remote: `curl -s https://codeberg.org/api/v1/repos/Word-Office/testsuite/contents | grep -c '"name"'`
  - No PR needed — this IS the main branch of a new repo

  **Must NOT do**:
  - Do NOT include the Codeberg API token in the README
  - Do NOT use HTTPS push (use SSH)
  - Do NOT modify docs/DESIGN.md or .sisyphus/ files

  **Recommended Agent Profile**:
  - **Category**: `writing`
    - Reason: README documentation writing task
  - **Skills**: [`git-master`]
    - `git-master`: Git push, remote management
  - **Skills Evaluated but Omitted**:
    - `playwright`: No browser needed

  **Parallelization**:
  - **Can Run In Parallel**: NO (final task)
  - **Parallel Group**: Wave 4
  - **Blocks**: F1-F4 (Final Verification Wave)
  - **Blocked By**: Tasks 7, 8, 9

  **References**:

  **Pattern References** (existing code to follow):
  - `server/Readme.md` — README format for this project's repos
  - `word-office-opencloud/README.md` — Companion README (architecture, setup, API sections)
  - Any existing repo README for disclaimer wording

  **API/Type References** (contracts to implement against):
  - Codeberg SSH: `git@codeberg.org:Word-Office/testsuite.git`
  - Codeberg API: `GET /api/v1/repos/Word-Office/testsuite/contents`

  **WHY Each Reference Matters**:
  - Existing READMEs establish the documentation style — new README should match

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: README contains all required sections
    Tool: Bash (grep)
    Preconditions: README.md exists
    Steps:
      1. grep -i "quick start" README.md
      2. grep -i "architecture" README.md
      3. grep -i "prerequisites" README.md
      4. grep -i "troubleshooting" README.md
      5. grep -i "disclaimer" README.md
      6. Assert all five found
    Expected Result: All required sections present
    Failure Indicators: Missing section
    Evidence: .sisyphus/evidence/task-10-readme-sections.txt

  Scenario: README has correct Codeberg URLs
    Tool: Bash (grep)
    Preconditions: README.md exists
    Steps:
      1. grep "codeberg.org/Word-Office/testsuite" README.md
      2. Assert at least 2 matches (clone URL + somewhere else)
    Expected Result: Codeberg URLs present
    Failure Indicators: No Codeberg URL or github.com URL found
    Evidence: .sisyphus/evidence/task-10-codeberg-urls.txt

  Scenario: Code is pushed to Codeberg
    Tool: Bash (curl)
    Preconditions: git push completed
    Steps:
      1. curl -s https://codeberg.org/api/v1/repos/Word-Office/testsuite/contents | grep -o '"name"'
      2. Assert at least 5 files found (README.md, package.json, jest.config.js, etc.)
    Expected Result: Files visible on Codeberg
    Failure Indicators: 404 or empty response
    Evidence: .sisyphus/evidence/task-10-codeberg-push.txt

  Scenario: Disclaimer wording is correct
    Tool: Bash (grep)
    Preconditions: README.md exists
    Steps:
      1. grep "entirely separate from" README.md
      2. Assert match found
      3. grep "Ascensio System SIA" README.md
      4. Assert match found
    Expected Result: Full disclaimer present with correct wording
    Failure Indicators: Missing disclaimer or incorrect wording
    Evidence: .sisyphus/evidence/task-10-disclaimer.txt
  ```

  **Commit**: YES
  - Message: `docs(testsuite): add README with setup and usage instructions`
  - Files: README.md
  - Pre-commit: `grep -q "Quick Start" README.md`

---

## Final Verification Wave

> 4 review agents run in PARALLEL. ALL must APPROVE. Present consolidated results to user and get explicit "okay" before completing.

- [x] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists (read file, curl endpoint, run command). For each "Must NOT Have": search codebase for forbidden patterns — reject with file:line if found. Check evidence files exist in .sisyphus/evidence/. Compare deliverables against plan.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [x] F2. **Code Quality Review** — `unspecified-high`
  Run `npm test` in testsuite/. Review all files for: hardcoded secrets, missing error handling, proper Jest patterns, Docker best practices. Check for AI slop: excessive comments, over-abstraction, unused imports.
  Output: `Tests [N pass/N fail] | Files [N clean/N issues] | VERDICT`

- [x] F3. **Real Manual QA** — `unspecified-high`
  Start from clean state. Run `docker compose up` then `npm test`. Execute EVERY QA scenario from EVERY task. Verify cross-service integration (WOPI discovery reaches DS, companion health checks reach all services). Save evidence to `.sisyphus/evidence/final-qa/`.
  Output: `Scenarios [N/N pass] | Integration [N/N] | VERDICT`

- [x] F4. **Scope Fidelity Check** — `deep`
  For each task: read "What to do", read actual diff. Verify 1:1 — everything in spec was built, nothing beyond spec was built. Check "Must NOT do" compliance. Flag unaccounted changes.
  Output: `Tasks [N/N compliant] | Unaccounted [CLEAN/N files] | VERDICT`

---

## Commit Strategy

| Wave | Commit Message | Files |
|------|---------------|-------|
| 1 | `feat(testsuite): initialize repo with project scaffolding` | package.json, .gitignore, jest.config.js, .env.test |
| 2 | `feat(testsuite): add Document Server Dockerfile built from core/` | docker/documentserver/Dockerfile |
| 3 | `feat(testsuite): add Docker Compose test stack` | docker-compose.test.yml, .env.test |
| 4 | `feat(testsuite): add health check E2E tests` | tests/health/*.test.js |
| 5 | `feat(testsuite): add WOPI discovery and API tests` | tests/wopi/*.test.js, tests/api/*.test.js |
| 6 | `ci(testsuite): add Forgejo Actions workflow` | .forgejo/workflows/e2e.yml |
| 7 | `docs(testsuite): add README with usage instructions` | README.md |

---

## Success Criteria

### Verification Commands
```bash
# In testsuite/ directory:
docker compose -f docker-compose.test.yml up -d    # Expected: all containers start
npm test                                            # Expected: all tests pass
curl http://localhost:9200/health                    # Expected: OCIS responds
curl http://localhost:8080/hosting/discovery         # Expected: WOPI XML from DS
curl http://localhost:3000/api/health                # Expected: companion health JSON
```

### Final Checklist
- [ ] All "Must Have" present
- [ ] All "Must NOT Have" absent
- [ ] All tests pass (`npm test` exits 0)
- [ ] CI pipeline passes on Codeberg
- [ ] Docker Compose stack starts and all services healthy
- [ ] No hardcoded secrets in any file
