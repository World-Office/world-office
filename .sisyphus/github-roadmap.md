# Word Office .github Repo Documentation Update

## TL;DR

> **Quick Summary**: Create and update 4 documentation files in the `Word-Office/.github` repo on Codeberg with a project roadmap, architecture map, contributing guide, and updated org profile — all adapted from the GitHub PR #2 blueprint with Codeberg-specific links.
> 
> **Deliverables**:
> - `ROADMAP.md` — 5-phase roadmap adapted from PR #2 for Codeberg
> - `ARCHITECTURE.md` — Full 22-repo dependency graph, build order, and inventory
> - `CONTRIBUTING.md` — Contributor guide with Codeberg links and dev setup
> - `profile/README.md` — Updated org profile with Codeberg links
> 
> **Estimated Effort**: Quick
> **Parallel Execution**: YES — 3 waves
> **Critical Path**: Task 1 (ARCHITECTURE.md) → Task 2 (ROADMAP.md) → Task 3 (CONTRIBUTING.md) → Task 4 (profile/README.md)

---

## Context

### Original Request
User asked to "plan the roadmap in very detail" using GitHub PR #2 (`https://github.com/Word Office/.github/pull/2`) as blueprint, then "go through all repos to understand what they are good for and draw an architecture map", and finally push everything to Codeberg.

### Interview Summary
**Key Discussions**:
- Target platform is **Codeberg** (`codeberg.org/Word-Office`), NOT GitHub
- The `.github` repo exists on Codeberg as a fork of Word Office
- PR #2 contains 7 files: ROADMAP.md, CONTRIBUTING.md, ISSUE_TEMPLATE/*, PULL_REQUEST_TEMPLATE.md, SECURITY.md
- User wants roadmap + architecture map, NOT issue/PR templates (separate work item)

**Research Findings**:
- Architecture map covers all 22 repos across 6 tiers (Engine, Assembly, Desktop, Integration, Assets, Org)
- Build order is: `core/` → `sdkjs/` → `web-apps/` → `server/` → `DocumentServer/` (and parallel desktop stack)
- 3 repos on Codeberg not in original AGENTS.md: `artwork/`, `word-office-opencloud/`, `documents-app-android/`
- `sdkjs/` is empty locally (git submodule not initialized)
- All 22 repos now cloned locally

### Metis Review
**Identified Gaps** (addressed):
- Codeberg `.github` repo may not render `profile/README.md` like GitHub → **Auto-resolved**: Codeberg uses `profile/` directory for org landing page (verified by existing README.md)
- GitHub-specific content in PR #2 needs URL adaptation → **Addressed in tasks**: Systematic URL replacement
- Build commands in CONTRIBUTING.md may be outdated → **Addressed**: Add "Last tested" date and version prerequisites
- File size limits needed → **Addressed**: Each file has line-count acceptance criteria

---

## Work Objectives

### Core Objective
Update the `Word-Office/.github` repo on Codeberg with 4 documentation files that provide project direction, technical architecture, and contributor onboarding.

### Concrete Deliverables
- `.github/ROADMAP.md` — 5-phase roadmap with Codeberg links
- `.github/ARCHITECTURE.md` — Dependency graph, build order, repo inventory
- `.github/CONTRIBUTING.md` — Contributor guide adapted for Codeberg
- `.github/profile/README.md` — Updated org profile with Codeberg links

### Definition of Done
- [x] All 4 files exist in `.github/` directory
- [x] Zero `github.com` URLs in any file (only `codeberg.org`)
- [x] `ROADMAP.md` ≤ 300 lines, `ARCHITECTURE.md` ≤ 500 lines, `CONTRIBUTING.md` ≤ 250 lines, `profile/README.md` ≤ 150 lines
- [x] All repo links point to `codeberg.org/Word-Office`

### Must Have
- 5-phase structure matching PR #2 blueprint
- ASCII dependency graph showing all 22 repos
- Build order documentation
- 22-repo inventory table with purpose, language, license
- All URLs pointing to Codeberg

### Must NOT Have (Guardrails)
- **DO NOT** modify files in any other repo — only `.github/`
- **DO NOT** create issue templates, PR templates, or security policy (separate work item)
- **DO NOT** add new directories, images, or assets
- **DO NOT** include any `github.com` URLs
- **DO NOT** modify `.github/screenshots/` or `.github/.git/`
- **AI SLOP**: No excessive markdown formatting, no emoji spam, no "comprehensive" headers

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed.

### Test Decision
- **Infrastructure exists**: NO (documentation-only, no tests needed)
- **Automated tests**: None
- **Framework**: None

### QA Policy
Every task MUST include agent-executed QA scenarios.
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

- **Documentation QA**: Use `grep` for URL verification, `wc` for line counts

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately — no dependencies):
├── Task 1: Create ARCHITECTURE.md [writing]
└── Task 2: Create CONTRIBUTING.md [writing]

Wave 2 (After Wave 1 — depends on ARCHITECTURE.md):
├── Task 3: Create ROADMAP.md [writing]
└── Task 4: Update profile/README.md [writing]

Wave FINAL (After ALL tasks — verification):
└── Task F1: Full verification — URL check, line counts, link validation [quick]
```

### Agent Dispatch Summary

- **Wave 1**: 2 tasks — T1 `writing`, T2 `writing`
- **Wave 2**: 2 tasks — T3 `writing`, T4 `writing`
- **FINAL**: 1 task — F1 `quick`

---

## TODOs

- [x] 1. Create ARCHITECTURE.md

  **What to do**:
  - Create `.github/ARCHITECTURE.md` with the full Word Office architecture documentation
  - Include the ASCII dependency graph from the architecture map draft (`.sisyphus/drafts/roadmap-architecture-map.md`)
  - Include the complete 22-repo inventory table organized by tier
  - Include the build order (topological sort)
  - Include a license compliance summary table
  - Include a "Where to Look" quick-reference table (adapted from AGENTS.md files)
  - Add SPDX header: `SPDX-FileCopyrightText: 2026 Word Office contributors` + `SPDX-License-Identifier: CC0-1.0`
  - Keep total file ≤ 500 lines

  **Must NOT do**:
  - Do not include any `github.com` URLs
  - Do not duplicate build instructions (reference CONTRIBUTING.md instead)
  - Do not create subdirectories

  **Recommended Agent Profile**:
  - **Category**: `writing`
    - Reason: Pure documentation task, no code changes
  - **Skills**: [`writing-plans`]
    - `writing-plans`: Markdown documentation conventions

  **Parallelization**:
  - **Can Run In Parallel**: YES (Wave 1)
  - **Parallel Group**: Wave 1 (with Task 2)
  - **Blocks**: Task 3 (ROADMAP.md references ARCHITECTURE.md)
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `.sisyphus/drafts/roadmap-architecture-map.md` — Full architecture map draft with dependency graph, build order, repo inventory
  - `.github/AGENTS.md` — 22-repo overview table with "Where to Look" sections (use for quick-reference table)

  **Content Source** (ROADMAP.md blueprint):
  - `https://raw.githubusercontent.com/tobias-weiss-ai-xr/.github/docs/add-roadmap/ROADMAP.md` — Phase structure, audit findings, license issues tables

  **External References**:
  - Codeberg org page: `https://codeberg.org/Word-Office` — Verify all repo names match

  **WHY Each Reference Matters**:
  - `roadmap-architecture-map.md`: Contains the complete dependency graph and repo inventory we created — source of truth for ARCHITECTURE.md content
  - `.github/AGENTS.md`: Has the authoritative per-repo "Where to Look" tables — useful for quick-reference section
  - PR #2 ROADMAP.md: License audit findings table needed for architecture doc's compliance section

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: File exists and has correct structure
    Tool: Bash (powershell)
    Preconditions: File created at .github/ARCHITECTURE.md
    Steps:
      1. Test-Path .github/ARCHITECTURE.md → $true
      2. Select-String "SPDX-FileCopyrightText" .github/ARCHITECTURE.md → match found
      3. Select-String "codeberg.org/Word-Office" .github/ARCHITECTURE.md → 3+ matches
    Expected Result: File exists, has SPDX header, has Codeberg links
    Evidence: .sisyphus/evidence/task-1-architecture-exists.txt

  Scenario: Zero GitHub URLs
    Tool: Bash (powershell)
    Preconditions: File created
    Steps:
      1. Select-String "github.com" .github/ARCHITECTURE.md
    Expected Result: Zero matches (no GitHub URLs)
    Failure Indicators: Any github.com URL found
    Evidence: .sisyphus/evidence/task-1-no-github-urls.txt

  Scenario: Line count within limit
    Tool: Bash (powershell)
    Preconditions: File created
    Steps:
      1. (Get-Content .github/ARCHITECTURE.md | Measure-Object -Line).Lines
    Expected Result: ≤ 500 lines
    Evidence: .sisyphus/evidence/task-1-line-count.txt

  Scenario: Contains dependency graph
    Tool: Bash (powershell)
    Preconditions: File created
    Steps:
      1. Select-String "core/" .github/ARCHITECTURE.md → 3+ matches (core referenced in multiple sections)
      2. Select-String "DocumentServer" .github/ARCHITECTURE.md → 2+ matches
      3. Select-String "desktop-sdk" .github/ARCHITECTURE.md → 1+ match
    Expected Result: Key repos referenced in dependency context
    Evidence: .sisyphus/evidence/task-1-deps-present.txt
  ```

  **Commit**: YES
  - Message: `Add architecture documentation with dependency graph and repo inventory`
  - Files: `.github/ARCHITECTURE.md`
  - Pre-commit: URL check, line count

- [x] 2. Create CONTRIBUTING.md

  **What to do**:
  - Create `.github/CONTRIBUTING.md` adapted from PR #2 CONTRIBUTING.md
  - Replace ALL `github.com/Word Office` URLs with `codeberg.org/Word-Office`
  - Update the repository overview table to include ALL 22 repos (PR #2 has 19, add artwork/, word-office-opencloud/, documents-app-android/)
  - Update development setup to reference Codeberg fork workflow
  - Replace "docker compose" with Codeberg-appropriate commands
  - Add SPDX header: `SPDX-FileCopyrightText: 2026 Word Office contributors` + `SPDX-License-Identifier: CC0-1.0`
  - Keep total file ≤ 250 lines

  **Must NOT do**:
  - Do not include any `github.com` URLs
  - Do not add issue/PR templates (separate work item)
  - Do not add Code of Conduct (separate work item)
  - Do not include build commands that haven't been tested

  **Recommended Agent Profile**:
  - **Category**: `writing`
    - Reason: Documentation adaptation task
  - **Skills**: [`writing-plans`]
    - `writing-plans`: Markdown conventions, structured documentation

  **Parallelization**:
  - **Can Run In Parallel**: YES (Wave 1)
  - **Parallel Group**: Wave 1 (with Task 1)
  - **Blocks**: None (independent of other docs)
  - **Blocked By**: None

  **References**:

  **Content Source** (CONTRIBUTING.md blueprint):
  - `https://raw.githubusercontent.com/tobias-weiss-ai-xr/.github/docs/add-roadmap/CONTRIBUTING.md` — Full CONTRIBUTING.md from PR #2 (195 lines)

  **Pattern References**:
  - `.sisyphus/drafts/roadmap-architecture-map.md` — 22-repo inventory table (use for repo overview table)

  **External References**:
  - Codeberg fork docs: `https://docs.codeberg.org/getting-started/first-steps/#forking-a-repository`

  **WHY Each Reference Matters**:
  - PR #2 CONTRIBUTING.md: Source blueprint — adapt content, don't rewrite from scratch
  - `roadmap-architecture-map.md`: Need complete 22-repo list for the overview table (PR #2 only has 19)

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: File exists with SPDX header
    Tool: Bash (powershell)
    Preconditions: File created at .github/CONTRIBUTING.md
    Steps:
      1. Test-Path .github/CONTRIBUTING.md → $true
      2. Select-String "SPDX-FileCopyrightText" .github/CONTRIBUTING.md → match found
    Expected Result: File exists with proper header
    Evidence: .sisyphus/evidence/task-2-contributing-exists.txt

  Scenario: All GitHub URLs replaced with Codeberg
    Tool: Bash (powershell)
    Preconditions: File created
    Steps:
      1. Select-String "github.com/Word Office" .github/CONTRIBUTING.md
      2. Select-String "codeberg.org/Word-Office" .github/CONTRIBUTING.md → 5+ matches
    Expected Result: Zero github.com/Word Office URLs, multiple codeberg.org URLs
    Evidence: .sisyphus/evidence/task-2-url-check.txt

  Scenario: Repo overview table has 22 entries
    Tool: Bash (powershell)
    Preconditions: File created
    Steps:
      1. Select-String "\|.*\|.*\|.*\|" .github/CONTRIBUTING.md | Measure-Object
    Expected Result: ≥ 23 table rows (22 repos + header)
    Evidence: .sisyphus/evidence/task-2-repo-count.txt

  Scenario: Line count within limit
    Tool: Bash (powershell)
    Preconditions: File created
    Steps:
      1. (Get-Content .github/CONTRIBUTING.md | Measure-Object -Line).Lines
    Expected Result: ≤ 250 lines
    Evidence: .sisyphus/evidence/task-2-line-count.txt
  ```

  **Commit**: YES
  - Message: `Add contributing guide with 22-repo overview and Codeberg workflow`
  - Files: `.github/CONTRIBUTING.md`

- [x] 3. Create ROADMAP.md

  **What to do**:
  - Create `.github/ROADMAP.md` adapted from PR #2 ROADMAP.md
  - Preserve 5-phase structure: Foundation → Feature Parity → Desktop Apps → Integration Ecosystem → Innovation
  - Replace ALL `github.com` URLs with `codeberg.org/Word-Office`
  - Adapt the audit log: update license findings, add desktop-sdk Phase 1 completion status
  - Adapt partner list for Codeberg community context
  - Update "How to Get Involved" links to Codeberg
  - Add reference to ARCHITECTURE.md for technical details
  - Add SPDX header: `SPDX-FileCopyrightText: 2026 Word Office contributors` + `SPDX-License-Identifier: CC0-1.0`
  - Keep total file ≤ 300 lines

  **Must NOT do**:
  - Do not remove phases or restructure the roadmap (preserve PR #2 blueprint structure)
  - Do not include any `github.com` URLs
  - Do not add new phases not in PR #2
  - Do not modify audit findings table structure

  **Recommended Agent Profile**:
  - **Category**: `writing`
    - Reason: Documentation adaptation with technical content
  - **Skills**: [`writing-plans`]
    - `writing-plans`: Structured documentation, roadmap conventions

  **Parallelization**:
  - **Can Run In Parallel**: YES (Wave 2, with Task 4)
  - **Parallel Group**: Wave 2 (with Task 4)
  - **Blocks**: None (parallel with Task 4)
  - **Blocked By**: Task 1 (references ARCHITECTURE.md)

  **References**:

  **Content Source** (ROADMAP.md blueprint):
  - `https://raw.githubusercontent.com/tobias-weiss-ai-xr/.github/docs/add-roadmap/ROADMAP.md` — Full ROADMAP.md from PR #2 (250+ lines)

  **Pattern References**:
  - `.sisyphus/drafts/roadmap-architecture-map.md` — Architecture details to cross-reference
  - `.sisyphus/notepads/desktop-sdk-rewrite/analysis.md` — desktop-sdk Phase 1 completion status for audit log

  **External References**:
  - Codeberg issues: `https://codeberg.org/Word-Office/DocumentServer/issues`

  **WHY Each Reference Matters**:
  - PR #2 ROADMAP.md: Source blueprint — adapt content, preserve structure
  - `analysis.md`: desktop-sdk Phase 1 license cleanup is DONE — update audit log to reflect this

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: File exists with all 5 phases
    Tool: Bash (powershell)
    Preconditions: File created at .github/ROADMAP.md
    Steps:
      1. Test-Path .github/ROADMAP.md → $true
      2. Select-String "Phase 1" .github/ROADMAP.md → match found
      3. Select-String "Phase 2" .github/ROADMAP.md → match found
      4. Select-String "Phase 3" .github/ROADMAP.md → match found
      5. Select-String "Phase 4" .github/ROADMAP.md → match found
      6. Select-String "Phase 5" .github/ROADMAP.md → match found
    Expected Result: All 5 phases present
    Evidence: .sisyphus/evidence/task-3-phases-present.txt

  Scenario: Zero GitHub URLs
    Tool: Bash (powershell)
    Preconditions: File created
    Steps:
      1. Select-String "github.com" .github/ROADMAP.md
    Expected Result: Zero matches
    Evidence: .sisyphus/evidence/task-3-no-github-urls.txt

  Scenario: References ARCHITECTURE.md
    Tool: Bash (powershell)
    Preconditions: File created
    Steps:
      1. Select-String "ARCHITECTURE" .github/ROADMAP.md → match found
    Expected Result: At least one reference to ARCHITECTURE.md
    Evidence: .sisyphus/evidence/task-3-arch-reference.txt

  Scenario: Line count within limit
    Tool: Bash (powershell)
    Preconditions: File created
    Steps:
      1. (Get-Content .github/ROADMAP.md | Measure-Object -Line).Lines
    Expected Result: ≤ 300 lines
    Evidence: .sisyphus/evidence/task-3-line-count.txt
  ```

  **Commit**: YES
  - Message: `Add project roadmap with 5 phases, audit findings, and partner list`
  - Files: `.github/ROADMAP.md`

- [x] 4. Update profile/README.md

  **What to do**:
  - Update `.github/profile/README.md` with Codeberg links
  - Replace `github.com/Word Office` with `codeberg.org/Word-Office`
  - Replace `github.com/Word Office/DocumentServer` with `codeberg.org/Word-Office/DocumentServer`
  - Fix the screenshot URL to point to Codeberg raw content
  - Add links to new docs: ROADMAP.md, ARCHITECTURE.md, CONTRIBUTING.md
  - Do NOT change the org description, partner list, or FAQ content
  - Keep total file ≤ 150 lines

  **Must NOT do**:
  - Do not rewrite the org description or FAQ
  - Do not add new sections beyond linking to new docs
  - Do not change the partner/supporter list
  - Do not modify screenshots/

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Simple URL replacement, minimal content changes
  - **Skills**: []
    - No special skills needed for URL replacement

  **Parallelization**:
  - **Can Run In Parallel**: YES (Wave 2, with Task 3)
  - **Parallel Group**: Wave 2 (with Task 3)
  - **Blocks**: None
  - **Blocked By**: None (independent)

  **References**:

  **Pattern References**:
  - `.github/profile/README.md` — Current org profile (75 lines, needs URL fixes)

  **External References**:
  - Codeberg raw file URL pattern: `https://codeberg.org/Word-Office/.github/raw/branch/main/profile/README.md`

  **WHY Each Reference Matters**:
  - `profile/README.md`: The file to modify — contains 3 GitHub URLs that need Codeberg replacement
  - Raw URL pattern: Screenshots reference GitHub raw content — need correct Codeberg equivalent

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Zero GitHub URLs in profile
    Tool: Bash (powershell)
    Preconditions: File updated
    Steps:
      1. Select-String "github.com" .github/profile/README.md
    Expected Result: Zero matches (all GitHub URLs replaced)
    Evidence: .sisyphus/evidence/task-4-no-github-urls.txt

  Scenario: Links to new docs present
    Tool: Bash (powershell)
    Preconditions: File updated
    Steps:
      1. Select-String "ROADMAP" .github/profile/README.md → match found
      2. Select-String "CONTRIBUTING" .github/profile/README.md → match found
      3. Select-String "ARCHITECTURE" .github/profile/README.md → match found
    Expected Result: Links to all 3 new documentation files present
    Evidence: .sisyphus/evidence/task-4-doc-links.txt

  Scenario: Screenshot URL is valid
    Tool: Bash (powershell)
    Preconditions: File updated
    Steps:
      1. Select-String "screenshot" .github/profile/README.md → match with codeberg.org URL
    Expected Result: Screenshot URL points to codeberg.org, not github.com
    Evidence: .sisyphus/evidence/task-4-screenshot-url.txt
  ```

  **Commit**: YES
  - Message: `Update org profile with Codeberg links and documentation references`
  - Files: `.github/profile/README.md`

---

## Final Verification Wave (MANDATORY — after ALL implementation tasks)

- [x] F1. **Full Documentation Verification** — `quick`
  Run all QA scenarios from all tasks against the actual files. Check:
  1. Zero `github.com` URLs across all 4 files (grep recursive)
  2. All Codeberg repo links are valid (22 repo names match)
  3. Line counts within limits (ROADMAP ≤300, ARCHITECTURE ≤500, CONTRIBUTING ≤250, profile ≤150)
  4. All inter-document references work (ROADMAP→ARCHITECTURE, profile→ROADMAP/CONTRIBUTING)
  5. SPDX headers present on all new files
  Output: `URLs [PASS/FAIL] | Line counts [N/N] | Cross-refs [N/N] | VERDICT`

---

## Commit Strategy

- **1**: `docs: Add architecture documentation with dependency graph and repo inventory` — `.github/ARCHITECTURE.md`
- **2**: `docs: Add contributing guide with 22-repo overview and Codeberg workflow` — `.github/CONTRIBUTING.md`
- **3**: `docs: Add project roadmap with 5 phases, audit findings, and partner list` — `.github/ROADMAP.md`
- **4**: `docs: Update org profile with Codeberg links and documentation references` — `.github/profile/README.md`

---

## Success Criteria

### Verification Commands
```powershell
# Zero GitHub URLs across all docs
Get-ChildItem -Recurse -Include *.md -Path .github/ | Select-String "github.com"
# Expected: 0 matches

# All files exist
Test-Path .github/ROADMAP.md; Test-Path .github/ARCHITECTURE.md; Test-Path .github/CONTRIBUTING.md; Test-Path .github/profile/README.md
# Expected: all $true

# Line count checks
(Get-Content .github/ROADMAP.md | Measure-Object -Line).Lines -le 300
(Get-Content .github/ARCHITECTURE.md | Measure-Object -Line).Lines -le 500
(Get-Content .github/CONTRIBUTING.md | Measure-Object -Line).Lines -le 250
(Get-Content .github/profile/README.md | Measure-Object -Line).Lines -le 150
# Expected: all $true

# Codeberg links present
Get-ChildItem -Recurse -Include *.md -Path .github/ | Select-String "codeberg.org/Word-Office" | Measure-Object
# Expected: 10+ matches
```

### Final Checklist
- [x] All 4 files created/updated in `.github/`
- [x] Zero `github.com` URLs
- [x] All `codeberg.org/Word-Office` repo names match actual Codeberg org
- [x] ROADMAP.md has all 5 phases
- [x] ARCHITECTURE.md has dependency graph + 22-repo table
- [x] CONTRIBUTING.md has 22-repo overview table
- [x] profile/README.md has links to new docs
- [x] All files have SPDX headers
- [x] All files within line-count limits
