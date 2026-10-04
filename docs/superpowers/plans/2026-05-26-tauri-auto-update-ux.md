# Auto-Update UX Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement a minimal auto-update UX that checks releases.json on Codeberg Pages, notifies the user, and installs updates on demand.

**Architecture:** updater.rs fetches releases.json, compares semver via the `semver` crate, emits Tauri events. Tray menu exposes "Check for Updates" and "Update Available" items. Frontend bridge listens for events and shows a toast. A CI job on tag push publishes releases.json.

**Tech Stack:** Rust (reqwest, semver), Tauri 2.0 event system, TypeScript bridge, Forgejo Actions.

---

### Task 1: Add dependencies + rewrite updater.rs

**Files:**
- Modify: `desktop/tauri-poc/src-tauri/Cargo.toml` — add `reqwest` and `semver`
- Rewrite: `desktop/tauri-poc/src-tauri/src/updater.rs`

- [ ] **Step 1: Add deps to Cargo.toml**

```toml
# Add below keyring = "3"
reqwest = { version = "0.12", features = ["json"] }
semver = "1"
```

- [ ] **Step 2: Rewrite updater.rs**

```rust
use semver::Version;
use serde::Deserialize;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

const RELEASES_URL: &str =
    "https://world-office.codeberg.page/desktop-releases/releases.json";

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ReleaseInfo {
    latest_version: String,
    download_url: String,
    release_notes_url: String,
    checksum: String,
}

pub struct UpdateState {
    pub latest_version: Option<String>,
    pub download_url: Option<String>,
}

impl UpdateState {
    pub fn new() -> Self {
        Self {
            latest_version: None,
            download_url: None,
        }
    }
}

#[tauri::command]
pub fn get_current_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

#[tauri::command]
pub async fn check_for_updates(app: AppHandle) -> Result<Option<String>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let resp = client
        .get(RELEASES_URL)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch releases.json: {}", e))?;

    let release: ReleaseInfo = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse releases.json: {}", e))?;

    let current = Version::parse(env!("CARGO_PKG_VERSION"))
        .map_err(|e| format!("Invalid current version: {}", e))?;
    let latest = Version::parse(&release.latest_version)
        .map_err(|e| format!("Invalid latest version: {}", e))?;

    // Store the state for later install
    let state = app.state::<Mutex<UpdateState>>();
    let mut st = state.lock().unwrap();
    st.latest_version = Some(release.latest_version.clone());
    st.download_url = Some(release.download_url.clone());

    if latest > current {
        let _ = app.emit("update-available", &release);
        Ok(Some(release.latest_version))
    } else {
        let _ = app.emit("update-checked", "up-to-date");
        Ok(None)
    }
}

#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    let state = app.state::<Mutex<UpdateState>>();
    let st = state.lock().unwrap();
    let url = st
        .download_url
        .clone()
        .ok_or_else(|| "No pending update".to_string())?;
    drop(st);

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Download failed: {}", e))?;

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let tmp = std::env::temp_dir().join("world-office-update.deb");
    std::fs::write(&tmp, &bytes)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;

    // Emit download-complete
    let _ = app.emit("update-downloaded", ());

    // On Linux: install via dpkg
    let status = std::process::Command::new("pkexec")
        .args(["dpkg", "-i", &tmp.to_string_lossy()])
        .status()
        .map_err(|e| format!("Failed to run installer: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err("Installation failed (non-zero exit)".to_string())
    }
}
```

- [ ] **Step 3: Register updater state and commands in lib.rs**

```rust
// Add alongside other manages:
use std::sync::Mutex;
// ...
.manage(Mutex::new(updater::UpdateState::new()))

// Add alongside other commands:
updater::get_current_version,
updater::check_for_updates,
updater::install_update,
```

- [ ] **Step 4a: Add startup auto-check in lib.rs**

In the `.setup()` closure (or after `.manage()`), trigger a background update check on startup:

```rust
// After .manage(...) calls, before .on_window_event(...)
.setup(move |app| {
    let handle = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        // Silent check — errors are swallowed (user didn't request check)
        let _ = updater::check_for_updates(handle).await;
    });
    Ok(())
})
```

- [ ] **Step 4b: Verify Rust compiles**

Run: `cargo check`
Expected: No new errors (pre-existing warnings OK)

- [ ] **Step 5: Commit**

```bash
git add desktop/tauri-poc/src-tauri/Cargo.toml desktop/tauri-poc/src-tauri/src/updater.rs desktop/tauri-poc/src-tauri/src/lib.rs
git commit -m "feat(desktop): implement update check against releases.json with download and install"
```

---

### Task 2: Tray menu update items

**Files:**
- Modify: `desktop/tauri-poc/src-tauri/src/tray.rs`

- [ ] **Step 1: Read current tray.rs to understand menu building patterns**

Run: `cat desktop/tauri-poc/src-tauri/src/tray.rs`

- [ ] **Step 2: Add update items to tray menu**

The tray builder builds a menu. Add these items after the existing items:

```rust
// After the existing .item(...) calls in the tray menu builder:
let check_updates = MenuItemBuilder::with_id("check-updates", "Check for Updates").build(app)?;
let update_available = MenuItemBuilder::with_id("update-available", "Update Available — Install Now")
    .build(app)?;
```

Add them to the submenu, and in the event handler:

```rust
// In the OnMenuEvent handler:
"check-updates" => {
    // Spawn async check
    let app_handle = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = super::updater::check_for_updates(app_handle).await {
            eprintln!("Update check failed: {}", e);
        }
    });
}
"update-available" => {
    let app_handle = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = super::updater::install_update(app_handle).await {
            eprintln!("Update install failed: {}", e);
        }
    });
}
```

- [ ] **Step 3: Verify Rust compiles**

Run: `cargo check`

- [ ] **Step 4: Commit**

```bash
git add desktop/tauri-poc/src-tauri/src/tray.rs
git commit -m "feat(desktop): add update check and install tray menu items"
```

---

### Task 3: Frontend update notification

**Files:**
- Modify: `apps/web/apps/documenteditor-react/src/bridge/event-listener.ts`
- Modify: `apps/web/apps/documenteditor-react/src/App.tsx`

- [ ] **Step 1: Add update listener to event-listener.ts**

```typescript
// Add alongside the existing listenForMenuEvents export:
export async function listenForUpdateEvents(
  onUpdateAvailable: () => void,
): Promise<() => void> {
  const unlisten = (await import("@tauri-apps/api/event"))
    .listen("update-available", () => {
      onUpdateAvailable()
    })
  return unlisten
}
```

- [ ] **Step 2: Wire into App.tsx**

```typescript
// Add import
import { listenForUpdateEvents } from "./bridge/event-listener"

// Add state
const [updateAvailable, setUpdateAvailable] = useState(false)

// Add effect alongside existing useEffect
useEffect(() => {
  if (!isDesktop()) return
  let unlisten: (() => void) | undefined
  listenForUpdateEvents(() => {
    setUpdateAvailable(true)
  }).then((fn) => { unlisten = fn })
  return () => { unlisten?.() }
}, [])
```

- [ ] **Step 3: Show notification in the render**

Add above the theme provider wrapper:

```typescript
{updateAvailable && (
  <div style={{
    position: "fixed", top: 4, left: "50%", transform: "translateX(-50%)",
    zIndex: 10000, background: "#2ecc71", color: "#fff",
    padding: "6px 16px", borderRadius: 4, fontSize: 13,
    display: "flex", alignItems: "center", gap: 8,
  }}>
    <span>⬆</span> Update available <button onClick={() => setUpdateAvailable(false)} style={{marginLeft:8, cursor:"pointer"}}>Dismiss</button>
  </div>
)}
```

- [ ] **Step 4: Run typecheck**

Run: `pnpm typecheck --filter @world-office/documenteditor`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add apps/web/apps/documenteditor-react/src/bridge/event-listener.ts apps/web/apps/documenteditor-react/src/App.tsx
git commit -m "feat(desktop): add frontend update notification banner"
```

---

### Task 4: CI workflow for releases.json

**Files:**
- Create: `.forgejo/workflows/releases.yml`

- [ ] **Step 1: Verify existing CI files to match conventions**

Run: `ls .forgejo/workflows/`

- [ ] **Step 2: Create releases.yml**

```yaml
name: Release Metadata
on:
  push:
    tags:
      - "v*"

jobs:
  metadata:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          repository: World-Office/desktop-releases
          token: ${{ secrets.CODEBERG_PAT }}

      - name: Generate releases.json
        run: |
          VERSION="${GITHUB_REF_NAME#v}"
          cat > releases.json <<EOF
          {
            "latest_version": "${VERSION}",
            "download_url": "https://codeberg.org/World-Office/desktop-releases/releases/download/v${VERSION}/world-office_${VERSION}_amd64.deb",
            "release_notes_url": "https://codeberg.org/World-Office/server/releases/tag/v${VERSION}",
            "checksum": "sha256-placeholder"
          }
          EOF
          git config user.name "World Office Bot"
          git config user.email "bot@world-office.dev"
          git add releases.json
          git commit -m "release v${VERSION}: update releases.json"
          git push
```

- [ ] **Step 3: Validate YAML**

Run: `python3 -c "import yaml; yaml.safe_load(open('.forgejo/workflows/releases.yml'))"` or similar syntax check
Expected: No errors

- [ ] **Step 4: Commit**

```bash
git add .forgejo/workflows/releases.yml
git commit -m "ci: add workflow to publish releases.json on tag"
```

---

### Task 5: Verification

- [ ] **Step 1: Rust check**

Run: `cargo check`
Expected: 0 errors (pre-existing warnings only)

- [ ] **Step 2: TypeScript check**

Run: `pnpm typecheck --filter @world-office/documenteditor`
Expected: PASS

- [ ] **Step 3: Full workspace typecheck**

Run: `pnpm typecheck`
Expected: 20/20 success

- [ ] **Step 4: Review checklist**

- updater.rs: fetches releases.json ✓, parses semver ✓, emits events ✓, downloads ✓, installs ✓
- tray.rs: check-updates item ✓, update-available item ✓, both spawn async handlers ✓
- event-listener.ts: listens for update-available ✓
- App.tsx: shows notification banner ✓
- releases.yml: tag push triggers ✓
