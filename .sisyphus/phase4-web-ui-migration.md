# Phase 4: Web UI Migration — Backbone Desktop → React 19

**Created:** 2026-04-11
**Scope:** Migrate 509 Backbone/jQuery files (497 JS + 12 HTML) across 7 apps to React 19
**Strategy:** Incremental — build editor shell, then migrate app-by-app (simplest first)
**Principle:** Chrome (toolbars/menus/panels/statusbar) → REWRITE. Canvas (document rendering) → KEEP via SDK bridge.

## Architecture

### Target Stack
- React 19 + TypeScript + Vite
- MobX 6 (state management — matches existing mobile pattern)
- @world-office/design-system (existing 12 components + tokens)
- Zustand (for editor state — lightweight alternative to MobX for complex state)
- i18next (localization)
- Biome (linting/formatting — matches project config)

### Migration Order (simplest → most complex)
1. **common** (155 files) — Shared components, utilities, base views
2. **visioeditor** (21 JS + 2 HTML) — Smallest app
3. **pdfeditor** (54 JS + 2 HTML) — Simple viewer
4. **presentationeditor** (62 JS + 3 HTML) — Medium complexity
5. **documenteditor** (90 JS + 2 HTML) — Full-featured editor
6. **spreadsheeteditor** (115 JS + 3 HTML) — Most complex app

## TODOs

### Phase 4A: Foundation — Editor Shell & Design System v2
- [x] 4A-1: Create `apps/web/apps/editor-shell/` Vite + React 19 + TypeScript project scaffold
- [x] 4A-2: Design system v2 — add editor-specific primitives to `@world-office/design-system` (Toolbar, Menu, Dropdown, Panel, ResizableSplit, TabBar, ContextMenu, Tooltip, IconButton, ToggleButton, ToolbarGroup, Separator, StatusBar)
- [x] 4A-3: Create SDK bridge module (`@world-office/sdk-bridge`) — TypeScript wrapper around existing `sdkjs` canvas API
- [x] 4A-4: Create MobX editor store foundation (EditorStore, DocumentStore, UIStore, CollaborationStore)
- [x] 4A-5: Create i18n setup with existing translation files from `apps/web/translation/`

### Phase 4B: Common Components Migration
- [x] 4B-1: Migrate common/main/lib/core/ — Application, NotificationCenter, keymaster
- [x] 4B-2: Migrate common/main/lib/component/ — BaseView → React components, Viewport, DocumentHolder
- [x] 4B-3: Migrate common/main/lib/controller/ — Scaling, Themes, Fonts, History, TabStyler, Desktop
- [x] 4B-4: Migrate common/main/lib/util/ — Tip, LocalStorage, utils
- [x] 4B-5: Migrate common/main/lib/mods/ — ALL VENDORED, replace with npm packages

### Phase 4C: Visio Editor (smallest app — proof of concept)
- [x] 4C-1: Migrate visioeditor/main/app/controller/Viewport.tsx
- [x] 4C-2: Migrate visioeditor toolbar and statusbar
- [x] 4C-3: Migrate visioeditor main controller and views
- [x] 4C-4: Integrate SDK bridge for canvas rendering
- [x] 4C-5: Verify visioeditor renders and is interactive

### Phase 4D: PDF Editor
- [x] 4D-1: Migrate pdfeditor toolbar, statusbar, and views
- [x] 4D-2: Migrate pdfeditor controllers (Main, Search, Print, etc.)
- [x] 4D-3: Verify pdfeditor renders and is interactive

### Phase 4E: Presentation Editor
- [x] 4E-1: Migrate presentationeditor toolbar, statusbar, and views
- [x] 4E-2: Migrate presentationeditor controllers
- [x] 4E-3: Verify presentationeditor renders and is interactive

### Phase 4F: Document Editor
- [x] 4F-1: Migrate documenteditor toolbar, statusbar, and views
- [x] 4F-2: Migrate documenteditor controllers (LeftMenu, RightMenu, Search, etc.)
- [x] 4F-3: Verify documenteditor renders and is interactive

### Phase 4G: Spreadsheet Editor (most complex)
- [x] 4G-1: Migrate spreadsheeteditor toolbar, statusbar, and views
- [x] 4G-2: Migrate spreadsheeteditor controllers (all sheets, formulas, etc.)
- [x] 4G-3: Verify spreadsheeteditor renders and is interactive

## Final Verification Wave
- [x] F1: ALL editor apps build with `pnpm build` — zero errors
- [x] F2: TypeScript strict mode passes across all migrated packages
- [x] F3: Visual regression — each editor renders identical to Backbone version
- [x] F4: Biome lint passes across all new/modified files
