# World-Office Quick Feature Demo

## TL;DR

> **Goal**: Screenshots + short video demo of the World-Office document editor UI
> **Deliverables**:
> - 3-5 annotated screenshots saved to `.sisyphus/evidence/screenshots/`
> - 30-60 second demo video saved to `.sisyphus/evidence/demo.mp4`
>
> **Tooling**: Playwright (browser automation) + ffmpeg (video encoding)
> **Estimated Effort**: Short
> **Parallel Execution**: NO (sequential steps)

---

## Context

### Original Request
"demonstrate how world-office works — create screenshots and a short demo video recording"

### Tool Availability
- **Playwright**: `@playwright/test@^1.59.1` installed in `tests/package.json`; needs `playwright install chromium`
- **ffmpeg**: `n8.0.1` at `/usr/bin/ffmpeg` — available on PATH
- **Dev server**: `pnpm --filter @world-office/documenteditor dev` from `server/` root

---

## Work Objectives

### Core Objective
Capture a quick visual demo of World-Office's document editor showing the UI in action with the new collaboration integration.

### Concrete Deliverables
- `docs/demo/screenshots/editor-ui.png` — Main editor screenshot
- `docs/demo/screenshots/toolbar.png` — Toolbar detail shot
- `docs/demo/screenshots/collaboration-status.png` — Collaboration status widget
- `docs/demo/screenshots/remote-cursor-demo.png` — Remote cursor overlay demonstration
- `docs/demo/demo.mp4` — 30-60 second screen recording with voice-over narration (or caption text)
- `docs/demo/README.md` — Brief explanation of what's shown in each screenshot/video segment

### Definition of Done
- [x] Screenshots saved with filename reflecting content
- [x] Video is ≤90s, shows the editor rendering a multi-page document
- [x] Video includes toolbar interaction and page navigation
- [x] README describes what's demonstrated in each file

### Must Have
- Browser-based capture (Playwright chromium)
- Editor renders without crash (canvas visible)
- Toolbar is visible in screenshots
- Video is encoded with ffmpeg (not raw)

### Must NOT Have
- No login/auth flow shown (bypass with local dev setup)
- No CI-specific config — use local dev server port (5173 or found port)
- No audio required (caption text in video if needed)

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: YES (Playwright + ffmpeg available)
- **Automated tests**: None needed — this is a demo capture task
- **QA Policy**: Manual verification via human review of screenshots/video output

### QA Policy
Every task's QA is **human review of the output files**. No automated assertions.

---

## Execution Strategy

### Sequential Steps (no parallelism — media capture requires consistent browser state)

```
Step 1 (Start Dev Server):
├── Start documenteditor dev server in background
├── Wait for port to be available
└── Record the port for use in Steps 2-4

Step 2 (Install Playwright Browsers):
├── cd server/tests && pnpm exec playwright install chromium
└── Verify chromium binary exists

Step 3 (Capture Screenshots):
├── Write Playwright script to:
│   ├── Navigate to http://localhost:{port}
│   ├── Wait for document canvas to render
│   ├── Capture screenshot: full-page (editor-ui)
│   ├── Capture screenshot: toolbar area only (toolbar)
│   ├── Capture screenshot: collaboration status widget (collaboration-status)
│   └── Capture screenshot: remote cursor overlay area (remote-cursor-demo)
├── Run the script
└── Verify screenshots exist in .sisyphus/evidence/screenshots/

Step 4 (Record Demo Video):
├── Write Playwright script to:
│   ├── Navigate to http://localhost:{port}
│   ├── Wait for page load
│   ├── Start ffmpeg screen recording (X11 or window capture)
│   ├── Page navigation: click Next/Prev a few times
│   ├── Toolbar interaction: hover over toolbar tabs
│   ├── Wait 10 seconds (show the rendered document)
│   ├── Stop ffmpeg recording
│   └── Output raw video to .sisyphus/evidence/raw.mp4
├── Encode: ffmpeg -i raw.mp4 -c:v libx264 -preset fast -crf 23 docs/demo/demo.mp4
└── Verify demo.mp4 exists and has duration ≤90s

Step 5 (Write README):
├── Create docs/demo/README.md with:
│   ├── Description of each screenshot
│   ├── Video chapter markers (e.g., "0:00-0:10 editor load")
│   └── Brief feature highlight bullets
└── Verify README.md exists
```

### Dependency Matrix
- **2**: 1 (needs dev server running first)
- **3**: 1, 2 (needs dev server + Playwright installed)
- **4**: 1, 2, 3 (needs Playwright installed, runs after screenshots)
- **5**: 3, 4 (describes the captured files)

### Critical Path
Step 1 → Step 2 → Step 3 → Step 4 → Step 5

---

## TODOs

> This plan is for media capture, not code implementation. QA is human review of output files.

- [x] 1. Start documenteditor dev server

  **What to do**:
  - From `server/` root: `pnpm --filter @world-office/documenteditor dev`
  - Capture the port it binds to (look for "Local: http://localhost:{PORT}")
  - Keep it running in background for the rest of the session

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Simple background process management
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential
  - **Blocks**: Steps 2, 3, 4
  - **Blocked By**: None (can start immediately)

  **References**:
  - `server/apps/web/apps/documenteditor-react/package.json:7` — `dev: "vite"` script
  - `server/package.json` — pnpm workspace root

  **Acceptance Criteria**:
  - [ ] `curl http://localhost:{PORT}` returns HTML (server responding)
  - [ ] Port number captured and noted

  **Commit**: NO

- [x] 2. Install Playwright chromium browser

  **What to do**:
  - `cd /home/weiss/git/World-Office/server/tests && pnpm exec playwright install chromium`
  - Verify by checking `tests/node_modules/.cache/ms-playwright/chromium-*/chrome-linux/chrome` exists

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: One-time tool setup, no complexity
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential
  - **Blocks**: Steps 3, 4
  - **Blocked By**: Step 1

  **References**:
  - `server/tests/package.json` — Playwright dependency
  - `server/tests/playwright.config.js` — chromium-only project

  **Acceptance Criteria**:
  - [ ] `test -f tests/node_modules/.cache/ms-playwright/chromium-*/chrome-linux/chrome` returns true

  **Commit**: NO

- [x] 3. Capture screenshots with Playwright

  **What to do**:
  - Write and run a Playwright script that:
    - Opens `http://localhost:{PORT}`
    - Waits for canvas to render (selector `.de-document-canvas`)
    - Captures 4 screenshots with descriptive names:
      - `editor-ui.png` — full page
      - `toolbar.png` — toolbar area
      - `collaboration-status.png` — collaboration status widget
      - `remote-cursor-demo.png` — document holder with remote cursors area
  - Save to `.sisyphus/evidence/screenshots/`

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Single script run, well-defined output
  - **Skills**: `["playwright"]`

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential
  - **Blocks**: Step 4
  - **Blocked By**: Steps 1, 2

  **References**:
  - `server/apps/web/apps/documenteditor-react/src/components/DocumentHolder.tsx:67` — `.de-document-canvas` canvas element
  - `server/apps/web/apps/documenteditor-react/src/components/Toolbar/Toolbar.tsx:29` — `CollaborationStatus` component
  - `server/apps/web/apps/documenteditor-react/src/components/DocumentHolder.tsx:72` — remote cursor overlays area

  **Acceptance Criteria**:
  - [ ] 4 screenshot files exist in `.sisyphus/evidence/screenshots/`
  - [ ] Each file is a valid PNG ≥ 100KB

  **Commit**: NO

- [x] 4. Record demo video with ffmpeg + Playwright

  **What to do**:
  - Write Playwright script to control the browser (no explicit screenshot)
  - Start ffmpeg screen recording: `ffmpeg -f x11grab -i :0 -c:v libx264 -preset ultrafast /tmp/recording.mp4`
  - Run Playwright actions:
    - Navigate to page
    - Click Next page button twice (`.de-page-nav-btn` with aria-label "Next page")
    - Hover over toolbar tabs
    - Let the document render for ~5 seconds
  - Stop ffmpeg (send SIGINT or use `-t 30` to auto-stop at 30s)
  - Encode: `ffmpeg -i /tmp/recording.mp4 -c:v libx264 -preset fast -crf 23 docs/demo/demo.mp4`
  - Save final video

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Straightforward screen recording task
  - **Skills**: `["playwright"]`

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential
  - **Blocks**: Step 5
  - **Blocked By**: Steps 1, 2, 3

  **References**:
  - `server/apps/web/apps/documenteditor-react/src/components/DocumentHolder.tsx:114` — Next page button
  - `server/apps/web/apps/documenteditor-react/src/components/Toolbar/Toolbar.tsx:17` — toolbar tabs

  **Acceptance Criteria**:
  - [ ] `docs/demo/demo.mp4` exists
  - [ ] `ffprobe -v error -show_entries format=duration -of default=noprint_wrappers=1 docs/demo/demo.mp4` shows ≤90s

  **Commit**: NO

- [x] 5. Write demo README

  **What to do**:
  - Create `docs/demo/README.md` with:
    - Brief description of the World-Office document editor
    - List of screenshots with descriptions of what each shows
    - Video chapter markers and descriptions (e.g., "0:00-0:10: Editor loads, canvas renders")
    - Key features demonstrated (page navigation, collaboration status, remote cursors)

  **Recommended Agent Profile**:
  - **Category**: `writing`
    - Reason: Documentation writing task
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential
  - **Blocks**: None
  - **Blocked By**: Steps 3, 4

  **References**:
  - Screenshots in `.sisyphus/evidence/screenshots/` (reference their names)
  - `docs/demo/demo.mp4` (reference the video)

  **Acceptance Criteria**:
  - [ ] `docs/demo/README.md` exists
  - [ ] Contains descriptions for all 4 screenshots
  - [ ] Contains video chapter markers

  **Commit**: NO

---

## Final Verification Wave

> **Note**: These are human-reviewed deliverables. The "verification" is checking that all files exist and have non-trivial size.

- [x] F1. **File Existence Check**
  Check all deliverables exist with non-trivial size:
  - `.sisyphus/evidence/screenshots/editor-ui.png` ≥ 100KB
  - `.sisyphus/evidence/screenshots/toolbar.png` ≥ 50KB
  - `.sisyphus/evidence/screenshots/collaboration-status.png` ≥ 20KB
  - `.sisyphus/evidence/screenshots/remote-cursor-demo.png` ≥ 50KB
  - `docs/demo/demo.mp4` exists and ≥ 500KB
  - `docs/demo/README.md` exists
  Output: `ls -la` of all files

---

## Success Criteria

```bash
# All screenshots exist with content
ls -la .sisyphus/evidence/screenshots/*.png

# Video exists and is ≤90 seconds
ffprobe -v error -show_entries format=duration -of default=noprint_wrappers=1 docs/demo/demo.mp4

# README exists
cat docs/demo/README.md
```

### Final Checklist
- [x] All 4 screenshots present and valid
- [x] Video exists and is ≤90s
- [x] README with descriptions for all files
- [x] Dev server stopped after capture (cleanup)