# WOPI-First Bridge: Nextcloud Integration Plan

**Goal:** Replace ONLYOFFICE DocsAPI iframe with World-Office React editors in the Nextcloud ODT integration, using WOPI protocol as the bridge.

**Created:** 2026-07-13
**Status:** **Completed**

---

## Background

The Nextcloud integration (`server/integrations/nextcloud/`) currently loads ONLYOFFICE's proprietary `DocsAPI.DocEditor` SDK in an iframe. World-Office has custom React editors (document, spreadsheet, presentation) that already implement WOPI client protocol but are not wired into Nextcloud.

This plan bridges these two systems by implementing WOPI host endpoints in the Nextcloud PHP app and loading the React editors in embedded mode.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│  Nextcloud (PHP)                                        │
│  ┌──────────────┐  ┌───────────────────────────────┐   │
│  │ editor.php   │  │ EditorController.php           │   │
│  │ (iframe) ───┼──┤ + WOPI Host endpoints (NEW)    │   │
│  └──────┬───────┘  └───────────────────────────────┘   │
└─────────┼───────────────────────────────────────────────┘
          │ iframe src = wo-docserver + WOPI params
          ▼
┌─────────────────────────────────────────────────────────┐
│  wo-docserver (Rust)                                    │
│  Serves React editor bundles as static assets            │
└─────────┬───────────────────────────────────────────────┘
          ▼
┌─────────────────────────────────────────────────────────┐
│  React Editor (embedded mode)                            │
│  ┌───────────────┐  ┌─────────────────────────────┐    │
│  │ Editor UI      │  │ @world-office/wopi-client    │    │
│  │ (WASM render)  │  │ (CheckFileInfo/Get/PutFile)  │    │
│  └───────────────┘  └─────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

---

## Phase 1: WOPI Host Endpoints in Nextcloud PHP (Backend Foundation)

**Effort:** L | **Risk:** High (protocol correctness)
**Dependency:** None

### 1.1 WOPI Base Controller [L]
Create `WOPIController.php` extending Nextcloud `AppFramework\Controller`.
- Implement `files` endpoint: `GET /apps/worldoffice/wopi/files/{fileId}`
  - Map Nextcloud file → WOPI CheckFileInfo response (JSON)
  - Return: `BaseFileName`, `Size`, `Version`, `UserId`, `UserName`, `UserCanWrite`, `SupportsUpdate`, `UserFriendlyName`
- Implement `contents` endpoint: `GET /apps/worldoffice/wopi/files/{fileId}/contents`
  - Read file from Nextcloud storage → binary response
- Implement `file save` endpoint: `PUT /apps/worldoffice/wopi/files/{fileId}/contents`
  - Accept binary body → write to Nextcloud storage
  - Return: X-WOPI-ItemVersion header
- Token validation middleware (reuse existing JWT logic from `DocumentService.php`)

**Success:** `curl` against endpoints returns valid WOPI JSON and file content.

### 1.2 Lock/Unlock Endpoints [M]
Add to `WOPIController.php`:
- `POST /apps/worldoffice/wopi/files/{fileId}/lock` — acquire lock
- `POST /apps/worldoffice/wopi/files/{fileId}/unlock` — release lock
- `POST /apps/worldoffice/wopi/files/{fileId}/refreshLock` — refresh lock
- Use Nextcloud's lock provider or in-memory (single-instance OK for MVP)

**Success:** Lock acquisition prevents concurrent writes; unlock allows next editor.

### 1.3 Routing Registration [S]
Register WOPI routes in `lib/AppInfo/Application.php`:
- Wire all WOPI endpoints to `WOPIController`
- Ensure CORS headers for iframe cross-origin requests
- Add `X-WOPI-ServerVersion` response header

**Success:** All WOPI routes return 405 for unsupported methods, 404 for missing files.

---

## Phase 2: React Editor Embedded Mode

**Effort:** L | **Risk:** Medium
**Dependency:** Phase 1 (needs WOPI host to test)

### 2.1 Embedded Mode Bootstrap [M]
In `server/apps/web/apps/documenteditor-react/src/` (and other editors):
- Detect `embedded=true` URL param or `window.__WORLD_OFFICE_CONFIG__.embedded`
- When embedded:
  - Hide toolbar chrome (file menu, settings, etc.)
  - Hide slide navigator / sheet tabs (presentation/spreadsheet)
  - Full-bleed canvas area
  - Disable Ctrl+S (auto-save only)
  - Emit `postMessage` lifecycle events to parent iframe

**Success:** Editor loads with minimal chrome, fills iframe.

### 2.2 PostMessage Protocol [L]
Define bidirectional `postMessage` protocol between React editor ↔ Nextcloud parent:

**Editor → Parent (upstream):**
- `app_ready` — editor initialized
- `document_ready` — file loaded and rendered
- `document_modified` — unsaved changes exist
- `document_saved` — save completed (with version)
- `error` — fatal error with code + message
- `request_close` — user wants to close (unsaved prompt)

**Parent → Editor (downstream):**
- `save` — trigger save (Ctrl+S equivalent)
- `close` — force close
- `set_user` — update user info
- `theme` — dark/light mode change

**Success:** Events flow bidirectionally; Nextcloud parent can receive save notifications.

### 2.3 Auto-Save Integration [M]
In embedded mode:
- Wire save to WOPI `PutFile` on change (debounced 3s)
- Show save indicator (dot/saving/saved)
- Handle lock conflicts (retry with backoff)
- On `postMessage('close')` from parent, save-then-close

**Success:** Changes auto-persist; closing editor saves file.

### 2.4 wo-docserver Static Serving [M]
Configure `wo-docserver` to serve built React editor bundles:
- Add route: `GET /editors/{type}/` → serve `index.html`
  - Types: `document`, `spreadsheet`, `presentation`
- Bundle React apps with proper public path
- Cache headers: immutable for hashed assets, short for HTML

**Success:** React editor loads from `wo-docserver/editors/document/?access_token=...&file_id=...`

---

## Phase 3: Nextcloud Integration Wiring

**Effort:** L | **Risk:** High (migration from DocsAPI)
**Dependency:** Phase 1, Phase 2

### 3.1 Rewrite editor.php Template [M]
Replace DocsAPI iframe with React editor iframe:
- Change iframe `src` from DocsAPI to `wo-docserver/editors/{type}/`
- Pass WOPI params: `access_token` (JWT), `file_id`, `embedded=true`
- Pass user info: `user_id`, `user_name`, `lang`
- Keep existing container div structure (CSS compatibility)

**Success:** Nextcloud "Edit" button opens React editor in iframe.

### 3.2 Rewrite editor.js as PostMessage Bridge [L]
Replace `DocsAPI.DocEditor` initialization with postMessage listener:
- Listen for upstream events from React editor iframe
- Map events to existing Nextcloud UI actions:
  - `document_ready` → hide loading spinner
  - `document_modified` → show unsaved indicator in Nextcloud toolbar
  - `document_saved` → update version display
  - `error` → show Nextcloud error dialog
  - `request_close` → show unsaved-changes dialog
- Forward downstream commands from Nextcloud to editor:
  - Theme change, user update, save, close
- Remove all DocsAPI references

**Success:** All existing Nextcloud UI interactions work with React editor.

### 3.3 EditorController Config Migration [M]
Modify `EditorController::indexAction()` (current entry point):
- Instead of building ONLYOFFICE config object, generate:
  - WOPI access token (JWT with file_id, user_id, expiry)
  - WOPI host base URL (self-referencing)
  - Editor type (from file MIME type)
- Pass to template as data attributes (same pattern as current)
- Keep backward compatibility flag: if `use_docsapi=true` param, fall back to ONLYOFFICE

**Success:** `indexAction` returns WOPI params; backward compat flag works.

### 3.4 Feature Port: Save-As, Insert-Image, History [L]
Port remaining DocsAPI-specific features:
- **Save-As:** Add WOPI `PutRelativeFile` endpoint + postMessage handler
- **Insert-Image:** Add postMessage `insert_image` command + file picker bridge
- **History:** Add `GET /wopi/files/{id}/versions` endpoint + postMessage protocol
- **Favorites/Action Links:** Map to Nextcloud API directly (no WOPI needed)

**Success:** All features from current integration work with React editor.

---

## Phase 4: Multi-Editor Support

**Effort:** M | **Risk:** Medium
**Dependency:** Phase 3

### 4.1 Editor Type Routing [S]
In `EditorController`, route by MIME type:
- `application/vnd.oasis.opendocument.text` → document editor
- `application/vnd.oasis.opendocument.spreadsheet` → spreadsheet editor
- `application/vnd.oasis.opendocument.presentation` → presentation editor
- Fallback: document editor with warning

### 4.2 Spreadsheet Embedded Mode [M]
Adapt `spreadsheeteditor-react` for embedded mode (same pattern as 2.1–2.3):
- Hide sheet tabs in embedded mode (or keep if requested)
- Wire save/load via WOPI
- PostMessage protocol reuse

### 4.3 Presentation Embedded Mode [M]
Adapt `presentationeditor-react` for embedded mode:
- Hide slide sorter sidebar in embedded mode
- Wire save/load via WOPI
- PostMessage protocol reuse

**Success:** All three editor types open from Nextcloud file manager.

---

## Phase 5: Testing & Cleanup

**Effort:** M | **Risk:** Low
**Dependency:** All prior phases

### 5.1 E2E Test Update [M]
Update `server/tests/` Playwright tests:
- Add test: open ODT file from Nextcloud → verify React editor loads
- Add test: edit → save → reopen → verify content persisted
- Add test: concurrent open → verify lock behavior
- Add test: save-as → verify new file created

### 5.2 Remove DocsAPI Dependency [S]
Once fully migrated:
- Remove `api.js` CDN reference from templates
- Remove `DocsAPI` type declarations
- Delete `viewer.js` (DocsAPI-specific)
- Clean up unused ONLYOFFICE config code in EditorController
- Keep `DirectEditor.php` (Nextcloud API, still needed)

### 5.3 Documentation [S]
Update Nextcloud integration README:
- Architecture diagram (WOPI flow)
- Configuration: `wo-docserver` URL, JWT secret
- Migration guide from ONLYOFFICE

**Success:** Clean removal of DocsAPI; tests pass; docs updated.

---

## Parallel Execution Opportunities

```
Phase 1 (PHP WOPI)    ──┐
                        ├── Can start together
Phase 2.1-2.2 (React)  ──┘   (Phase 2 can stub WOPI responses)

Phase 3.3 (Config) ─── independent, can start with Phase 1

Phase 2.4 (docserver) ── independent
Phase 4.1 (routing)   ── independent

Phase 3.1-3.4 (Nextcloud wiring) ── depends on Phase 1 + 2

Phase 4.2-4.3 (other editors) ── parallel, each M effort

Phase 5 (testing/cleanup) ── sequential, last
```

**Maximum parallelism:** Phase 1 + Phase 2 + Phase 3.3 + Phase 2.4 + Phase 4.1 can all start simultaneously.

---

## Risk Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| WOPI protocol mismatch | Editor can't save/load | Write Phase 1 first; test with curl before wiring UI |
| DocsAPI feature gap | Missing features in React editor | Keep backward compat flag; incremental migration |
| JWT token format incompatibility | Auth failures | Reuse existing `DocumentService.php` JWT logic |
| Cross-origin iframe restrictions | postMessage blocked | Verify CORS headers; test early in Phase 2.2 |
| File locking race conditions | Data loss | Use Nextcloud lock provider; retry with backoff |

---

## Summary

| Phase | Tasks | Effort | Key Deliverable |
|-------|-------|--------|-----------------|
| 1. WOPI Host (PHP) | 3 | L | Nextcloud serves WOPI endpoints |
| 2. React Embedded | 4 | L | Editors run in iframe with postMessage |
| 3. Integration Wiring | 4 | L | Nextcloud opens React editors, full feature parity |
| 4. Multi-Editor | 3 | M | All three editor types work |
| 5. Testing & Cleanup | 3 | M | Tests pass, DocsAPI removed |

**Total effort:** ~5L + 2M ≈ 2.5 weeks with 2 developers, or 1 week with full parallelism.
