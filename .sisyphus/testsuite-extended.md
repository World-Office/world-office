# Test Suite — Extended Test Scenarios

## TL;DR

> **Quick Summary**: Extend the testsuite with comprehensive WOPI protocol tests, security validation, and Playwright-based document editing tests.
>
> **Deliverables**:
> - Security test suite (JWT validation, input validation, XSS/CSRF)
> - WOPI protocol operation tests (CheckFileInfo, Lock/Unlock, GetFile/PutFile)
> - Playwright document editing tests (open, edit, save, close)
> - Edge case tests (empty files, large files, concurrent access)
>
> **Estimated Effort**: Large
> **Parallel Execution**: YES - 4 waves
> **Critical Path**: T1-T3 (Security) → T4-T6 (WOPI) → T7-T9 (Playwright) → T10-T11 (Edge Cases)

---

## Context

### Original Request
"Plan more test scenarios for the Word-Office testsuite" with priority on security validation.

### Interview Summary
**Key Discussions**:
- **Test Coverage**: Full WOPI protocol + Document editing + Security + edge cases
- **Testing Tool**: Playwright for browser-based document operations
- **Priority**: Security validation first
- **Test Approach**: Real Document Server (not mocked)

**Current Test Coverage**:
- Health checks: DS, OCIS, Companion, Stack (4 test files)
- WOPI discovery: XML validation
- API endpoints: /api/health, /api/config, /api/health/wopi

**Research Findings**:
- WOPI protocol has defined operations: CheckFileInfo, Lock/Unlock, GetFile/PutFile
- JWT validation is critical for OCIS ↔ DS communication
- Playwright is recommended for document editing tests (browser-based)
- Edge cases include: empty files, large files, concurrent access, network failures

---

## Work Objectives

### Core Objective
Extend the testsuite with comprehensive test coverage for security, WOPI protocol operations, document editing, and edge cases.

### Concrete Deliverables
- `tests/e2e/security/*.test.js` - Security validation tests
- `tests/e2e/wopi/operations.test.js` - WOPI protocol operations
- `tests/e2e/documents/*.test.js` - Playwright document editing tests
- `tests/e2e/edge/*.test.js` - Edge case handling tests
- Playwright configuration for browser-based testing

### Definition of Done
- [ ] All security tests pass (JWT, input validation, XSS/CSRF)
- [ ] WOPI protocol operations validated
- [ ] Document editing works end-to-end via Playwright
- [ ] Edge cases handled gracefully

### Must Have
- JWT token validation tests (invalid, expired, malformed)
- Input validation on all endpoints
- WOPI CheckFileInfo, Lock/Unlock, GetFile/PutFile tests
- Playwright document open/edit/save/close tests

### Must NOT Have (Guardrails)
- NO performance/load tests
- NO accessibility tests
- NO tests that require manual intervention
- NO mocked WOPI responses (use real DS)

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** - ALL verification is agent-executed.

### Test Decision
- **Infrastructure exists**: YES (Jest + Playwright to be added)
- **Automated tests**: YES (TDD approach)
- **Framework**: Jest + Playwright

### QA Policy
Every task MUST include agent-executed QA scenarios.
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately — Security foundation):
├── Task 1: JWT validation tests [deep]
├── Task 2: Input validation tests [deep]
└── Task 3: XSS/CSRF protection tests [deep]

Wave 2 (After Wave 1 — WOPI Protocol):
├── Task 4: CheckFileInfo operation tests [deep]
├── Task 5: Lock/Unlock operation tests [deep]
└── Task 6: GetFile/PutFile operation tests [deep]

Wave 3 (After Wave 2 — Playwright Setup + Document Editing):
├── Task 7: Playwright configuration + setup [quick]
├── Task 8: Document open/edit/save tests [visual-engineering]
└── Task 9: Co-editing tests [deep]

Wave 4 (After Wave 3 — Edge Cases):
├── Task 10: Empty/large file tests [quick]
└── Task 11: Concurrent access + network failure tests [deep]

Wave FINAL (After ALL tasks — verification):
├── F1: Plan compliance audit [oracle]
├── F2: Code quality review [unspecified-high]
├── F3: Real manual QA [unspecified-high]
└── F4: Scope fidelity check [deep]
→ Present results → Get explicit user okay
```

### Dependency Matrix

| Task | Depends On | Blocks | Wave |
|------|-----------|--------|------|
| 1 | — | F1-F4 | 1 |
| 2 | — | F1-F4 | 1 |
| 3 | — | F1-F4 | 1 |
| 4 | — | F1-F4 | 2 |
| 5 | — | F1-F4 | 2 |
| 6 | — | F1-F4 | 2 |
| 7 | — | 8, 9 | 3 |
| 8 | 7 | F1-F4 | 3 |
| 9 | 7 | F1-F4 | 3 |
| 10 | — | F1-F4 | 4 |
| 11 | — | F1-F4 | 4 |

---

## TODOs

- [x] 1. JWT Validation Tests

  **What to do**:
  - Create `tests/e2e/security/jwt.test.js`
  - Test invalid JWT token → 401 Unauthorized
  - Test expired JWT token → 401 Unauthorized
  - Test malformed JWT (invalid signature) → 401 Unauthorized
  - Test missing JWT header → 401 Unauthorized
  - Test valid JWT → 200 OK

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with T2, T3)

  **QA Scenarios**:
  ```
  Scenario: Invalid JWT returns 401
    Tool: Bash (curl)
    Steps:
      1. curl -H "Authorization: Bearer invalid_token" http://localhost:8080/hosting/discovery
      2. Assert HTTP 401
    Evidence: .sisyphus/evidence/task-1-invalid-jwt.txt
  ```

  **Commit**: YES
  - Message: `test(security): add JWT validation tests`

- [x] 2. Input Validation Tests

  **What to do**:
  - Create `tests/e2e/security/input-validation.test.js`
  - Test malformed JSON body → 400 Bad Request
  - Test missing required fields → 400 Bad Request
  - Test invalid field types → 400 Bad Request
  - Test SQL injection attempts → 400/sanitized
  - Test path traversal attempts → 403 Forbidden

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with T1, T3)

  **QA Scenarios**:
  ```
  Scenario: Malformed JSON returns 400
    Tool: Bash (curl)
    Steps:
      1. curl -X POST -H "Content-Type: application/json" -d '{"invalid": }' http://localhost:3000/api/setup
      2. Assert HTTP 400
    Evidence: .sisyphus/evidence/task-2-malformed-json.txt
  ```

  **Commit**: YES
  - Message: `test(security): add input validation tests`

- [x] 3. XSS/CSRF Protection Tests

  **What to do**:
  - Create `tests/e2e/security/xss-csrf.test.js`
  - Test XSS payload in document name → sanitized
  - Test XSS payload in file content → sanitized
  - Test CSRF without token → 403 Forbidden
  - Test CSRF with invalid token → 403 Forbidden

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with T1, T2)

  **QA Scenarios**:
  ```
  Scenario: XSS payload is sanitized
    Tool: Bash (curl)
    Steps:
      1. POST document with name "<script>alert('xss')</script>.docx"
      2. Assert response does not contain unescaped script tag
    Evidence: .sisyphus/evidence/task-3-xss-sanitized.txt
  ```

  **Commit**: YES
  - Message: `test(security): add XSS/CSRF protection tests`

- [x] 4. WOPI CheckFileInfo Tests

  **What to do**:
  - Create `tests/e2e/wopi/check-file-info.test.js`
  - Test CheckFileInfo returns required fields (BaseFileName, OwnerId, Size, Version, UserId)
  - Test UserCanWrite, UserCanNotWriteRelative flags
  - Test SupportsUpdate, SupportsLocks flags
  - Test with different file types (docx, xlsx, pptx)
  - Validate response Content-Type: application/json

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with T5, T6)

  **QA Scenarios**:
  ```
  Scenario: CheckFileInfo returns valid structure
    Tool: Bash (curl)
    Steps:
      1. GET /wopi/files/{file_id}?access_token={token}
      2. Assert JSON contains BaseFileName, OwnerId, Size, Version, UserId
    Evidence: .sisyphus/evidence/task-4-check-file-info.txt
  ```

  **Commit**: YES
  - Message: `test(wopi): add CheckFileInfo operation tests`

- [x] 5. WOPI Lock/Unlock Tests

  **What to do**:
  - Create `tests/e2e/wopi/lock-unlock.test.js`
  - Test Lock operation → X-WOPI-Lock header returned
  - Test Unlock operation → 200 OK
  - Test Unlock with wrong lock → 409 Conflict
  - Test RefreshLock operation
  - Test GetLock operation returns current lock

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with T4, T6)

  **QA Scenarios**:
  ```
  Scenario: Lock prevents concurrent edits
    Tool: Bash (curl)
    Steps:
      1. POST /wopi/files/{file_id}/lock with X-WOPI-Lock header
      2. Attempt second lock with different X-WOPI-Lock
      3. Assert 409 Conflict
    Evidence: .sisyphus/evidence/task-5-lock-conflict.txt
  ```

  **Commit**: YES
  - Message: `test(wopi): add Lock/Unlock operation tests`

- [x] 6. WOPI GetFile/PutFile Tests

  **What to do**:
  - Create `tests/e2e/wopi/file-operations.test.js`
  - Test GetFile returns file content with correct Content-Type
  - Test PutFile updates file content
  - Test PutFile with invalid X-WOPI-Lock → 409 Conflict
  - Test file size after PutFile matches sent size
  - Test binary file content preservation

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with T4, T5)

  **QA Scenarios**:
  ```
  Scenario: PutFile updates content
    Tool: Bash (curl)
    Steps:
      1. POST /wopi/files/{file_id}/contents with new content
      2. GET /wopi/files/{file_id}/contents
      3. Assert returned content matches sent content
    Evidence: .sisyphus/evidence/task-6-put-get-file.txt
  ```

  **Commit**: YES
  - Message: `test(wopi): add GetFile/PutFile operation tests`

- [x] 7. Playwright Configuration Setup

  **What to do**:
  - Install Playwright: `npm install -D @playwright/test`
  - Create `playwright.config.js` with:
    - baseURL: http://localhost:9200 (OCIS)
    - testDir: tests/e2e/documents
    - browser: chromium
    - screenshot: only-on-failure
    - video: retain-on-failure
  - Create `tests/e2e/documents/.gitkeep`
  - Update package.json scripts: "test:e2e:browser": "playwright test"

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with T8, T9 after this completes)
  - **Blocks**: Tasks 8, 9

  **QA Scenarios**:
  ```
  Scenario: Playwright config is valid
    Tool: Bash
    Steps:
      1. npx playwright test --list
      2. Assert exit code 0 (config parses successfully)
    Evidence: .sisyphus/evidence/task-7-playwright-config.txt
  ```

  **Commit**: YES
  - Message: `test(setup): add Playwright configuration`

- [x] 8. Document Open/Edit/Save Tests (Playwright)

  **What to do**:
  - Create `tests/e2e/documents/editing.spec.js`
  - Test: Open document via WOPI editor URL
  - Test: Type text in document editor
  - Test: Save document (trigger auto-save or manual save)
  - Test: Close document tab
  - Test: Reopen document → changes persisted
  - Use Playwright page interactions: page.goto(), page.type(), page.click()

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
  - **Skills**: [`playwright`]
    - `playwright`: Browser automation for document editing

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with T9, after T7)

  **QA Scenarios**:
  ```
  Scenario: Edit and save document
    Tool: Playwright
    Steps:
      1. page.goto('http://localhost:9200/f/{file_id}')
      2. Wait for editor to load (selector: .editor-canvas)
      3. page.type('.editor-canvas', 'Test content')
      4. Wait for auto-save (2 seconds)
      5. page.close()
      6. Reopen document
      7. Assert 'Test content' visible in document
    Evidence: .sisyphus/evidence/task-8-edit-save.png
  ```

  **Commit**: YES
  - Message: `test(documents): add Playwright document editing tests`

- [x] 9. Co-editing Tests (Playwright)

  **What to do**:
  - Create `tests/e2e/documents/coediting.spec.js`
  - Test: Open same document in two browser contexts
  - Test: User A types → User B sees changes (real-time)
  - Test: Both users edit different sections → no conflict
  - Test: Both users edit same section → last-write-wins or conflict resolution
  - Use Promise.all() for parallel browser contexts

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: [`playwright`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with T8, after T7)

  **QA Scenarios**:
  ```
  Scenario: Real-time co-editing works
    Tool: Playwright
    Steps:
      1. Create two browser contexts: contextA, contextB
      2. Both open same document URL
      3. contextA.page.type('.editor', 'User A content')
      4. Wait 2 seconds
      5. Assert contextB.page.content() contains 'User A content'
    Evidence: .sisyphus/evidence/task-9-coediting.png
  ```

  **Commit**: YES
  - Message: `test(documents): add co-editing tests`

- [x] 10. Empty/Large File Edge Case Tests

  **What to do**:
  - Create `tests/e2e/edge/file-sizes.test.js`
  - Test: Empty file (0 bytes) → opens successfully
  - Test: Large file (50MB) → opens within timeout
  - Test: File at size limit (100MB) → appropriate error if limit exceeded
  - Test: Very long filename → handled gracefully
  - Test: Special characters in filename → sanitized

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 4 (with T11)

  **QA Scenarios**:
  ```
  Scenario: Empty file opens successfully
    Tool: Bash (curl)
    Steps:
      1. Create empty file in OCIS: touch empty.docx
      2. GET /wopi/files/{empty_file_id}?access_token={token}
      3. Assert 200 OK with Size: 0
    Evidence: .sisyphus/evidence/task-10-empty-file.txt
  ```

  **Commit**: YES
  - Message: `test(edge): add file size edge case tests`

- [x] 11. Concurrent Access + Network Failure Tests

  **What to do**:
  - Create `tests/e2e/edge/concurrency.test.js`
  - Test: 10 concurrent file opens → all succeed
  - Test: Concurrent writes to same file → lock prevents corruption
  - Test: Network timeout during save → retry mechanism works
  - Test: DS restart during editing → graceful reconnection
  - Test: Invalid file type (.exe, .zip) → appropriate error

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 4 (with T10)

  **QA Scenarios**:
  ```
  Scenario: Concurrent file opens succeed
    Tool: Bash (curl + xargs)
    Steps:
      1. Generate 10 parallel curl requests to same file
      2. Assert all return 200 OK
    Evidence: .sisyphus/evidence/task-11-concurrent.txt
  ```

  **Commit**: YES
  - Message: `test(edge): add concurrency and network failure tests`

---

## Final Verification Wave

- [x] F1. Plan Compliance Audit — `oracle` ✅ APPROVE
  Read the plan end-to-end. For each "Must Have": verify test exists. For each "Must NOT Have": search for forbidden patterns. Check evidence files exist. Compare deliverables against plan.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | VERDICT: APPROVE/REJECT`

- [x] F2. Code Quality Review — `unspecified-high` ✅ APPROVE
  Run `npx playwright test --list` + `npm test`. Review all new test files for: proper assertions, evidence capture, timeout handling. Check for hardcoded secrets.
  Output: `Tests [N pass/N fail] | Files [N clean/N issues] | VERDICT`

- [x] F3. Real Manual QA — `unspecified-high` (+ `playwright` skill if UI) ✅ APPROVE
  Start from clean state. Execute tests: `npm test && npx playwright test`. Verify Playwright screenshots saved. Test cross-feature integration.
  Output: `Tests [N/N pass] | Screenshots [N captured] | VERDICT`

- [x] F4. Scope Fidelity Check — `deep` ✅ APPROVE
  For each task: read "What to do", read actual test file. Verify 1:1 — all tests in spec were built, nothing beyond spec. Check "Must NOT do" compliance.
  Output: `Tasks [N/N compliant] | VERDICT`

---

## Commit Strategy

Group commits by wave for atomic changes.

---

## Success Criteria

### Verification Commands
```bash
npm test                                    # All Jest tests pass
npx playwright test                         # All Playwright tests pass
docker compose -f docker-compose.test.yml ps  # All services healthy
```

### Final Checklist
- [ ] All "Must Have" present
- [ ] All "Must NOT Have" absent
- [ ] All tests pass
