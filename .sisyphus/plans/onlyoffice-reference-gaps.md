# Plan: WOPI-Gaps beheben (ONLYOFFICE-Referenz)

## Problem

Die React-Editor-GUI ist nicht funktionsfähig, weil 5 konkrete Abweichungen vom WOPI-Standard bestehen. ONLYOFFICE Document Server diente als Referenz, da es eine vollständig funktionierende WOPI-Implementierung ist.

---

## Gap 1 (🔴 KRITISCH): Token im Header statt Query-Parameter

**Betroffene Dateien:**
- `packages/wopi-client/src/wopi-client.ts` (Editor → Server)
- `core/crates/wo-docserver/src/lib.rs` (Server-Seite)

**Problem:**  
Der WOPI-Standard (MS-WOPI Spezifikation) verlangt den `access_token` als **Query-Parameter** in der URL:
```
GET /wopi/files/{file_id}?access_token={token}
```

Unser `wopi-client.ts` sendet den Token fälschlich als `Authorization: Bearer`-Header:
```typescript
const res = await fetch(url, { headers: authHeaders(conn.wopiAccessToken) })
```

Der Server erwartet korrekt `?access_token=` (via `Query<TokenQuery>`). Ergebnis: **Jeder WOPI-Request des Editors scheitert mit 401/400** → "Failed to load document."

**Lösung A (empfohlen) — Editor-Seite fixen:**
```typescript
// wopi-client.ts
const url = `${conn.docserverBase}/wopi/files/${conn.wopiFileId}?access_token=${encodeURIComponent(conn.wopiAccessToken)}`
const res = await fetch(url)
```

**Lösung B (optional zusätzlich) — Server flexibler machen:**
`wopi_check_file_info`, `wopi_get_file`, `wopi_put_file` zusätzlich `Authorization: Bearer` aus dem Header auslesen.

**Aufwand:** 1 Datei, ~5 Zeilen Änderung

---

## Gap 2 (🔴 KRITISCH): Discovery XML fehlt WOPISrc-Template

**Betroffene Datei:** `core/crates/wo-docserver/src/wopi.rs` (Zeile 160-194)

**Problem:**  
Die `urlsrc`-Attribute im Discovery XML müssen das `WOPISrc`-Template enthalten, damit der WOPI-Host (OCIS collaboration service) die WOPI-Quelle korrekt substituieren kann:

**ONLYOFFICE-Referenz:**
```xml
<action name="edit" ext="docx" 
  urlsrc="https://onlyoffice/hosting/wopi/word/edit?WOPISrc=<WOPISrc>"/>
```

**Unser aktuell:**
```xml
<action name="edit" ext="docx" 
  urlsrc="{base}/hosting/wopi/word/edit"/>
```

**Lösung:**  
`WOPISrc=<WOPISrc>` an jedes `urlsrc` anhängen.

**Aufwand:** 1 Datei, ~15 Zeilen

---

## Gap 3 (🔴 KRITISCH): CheckFileInfo zu minimal

**Betroffene Dateien:**
- `core/crates/wo-wopi/src/models.rs` (CheckFileInfoResponse)
- `core/crates/wo-docserver/src/lib.rs` (Handler)

**Problem:**  
Das CheckFileInfo-Response hat nur 9 Felder. OCIS collaboration service und der Editor brauchen weitere Felder für UI-Features:

**Unverzichtbar (brechen die UI ohne):**
| Feld | Typ | Zweck |
|------|-----|-------|
| `UserFriendlyName` | String | Anzeigename des Users |
| `LastModifiedTime` | String (ISO 8601) | Änderungsdatum |
| `BreadcrumbDocName` | String | Dateiname im Breadcrumb |
| `CloseUrl` | String | URL zum Schließen (zurück zu OCIS) |
| `HostAuthenticationId` | String | Auth-ID des Hosts |

**Wichtig für Vollständigkeit:**
| Feld | Typ | Zweck |
|------|-----|-------|
| `UserCanNotWriteRelative` | bool | Kein PutRelativeFile |
| `SupportsCoauth` | bool | Co-Authoring-Unterstützung |
| `SupportsRename` | bool | Umbenennen erlaubt |
| `SupportsDelete` | bool | Löschen erlaubt |
| `FileUrl` | String | Direkter Download-Link |

**Lösung:**  
`CheckFileInfoResponse` um diese Felder ergänzen. Den Proxy-Handler in `lib.rs` so anpassen, dass er Felder aus der OCIS-Antwort übernimmt oder sinnvolle Defaults setzt.

**Aufwand:** ~50 Zeilen in models.rs, ~20 Zeilen in lib.rs

---

## Gap 4 (🟡 MITTEL): WOPISrc im POST-Handler verworfen

**Betroffene Datei:** `core/crates/wo-docserver/src/lib.rs` (Zeile 242-284)

**Problem:**  
Die `build_editor_redirect_url`-Funktion extrahiert bei POST nur `access_token`, `file_id` und `embedded` aus dem Form-Body. Der `WOPISrc`-Parameter (den OCIS im Query-String übergibt) wird ignoriert.

Bei GET-Requests wird `WOPISrc` dagegen korrekt durchgereicht (Zeile 277-283).

**Lösung:**  
`WOPISrc` aus dem Query-String der ursprünglichen Request-URL extrahieren und an die Redirect-URL anhängen.

```rust
// request.uri().query() enthält z.B. "WOPISrc=https%3A%2F%2F..."
let query = request.uri().query().unwrap_or("");
let wopi_src = extract_query_param(query, "WOPISrc");
if !wopi_src.is_empty() {
    qs_parts.push(format!("WOPISrc={}", urlencoding(&wopi_src)));
}
```

**Aufwand:** ~10 Zeilen

---

## Gap 5 (🔴 KRITISCH): Editor UI exists not oder falscher Pfad

**Betroffene Datei:** `core/crates/wo-docserver/src/lib.rs` + Docker-Konfiguration

**Problem:**  
Die React-Editoren müssen gebuildet und ins richtige Verzeichnis kopiert werden. Der `editor_ui_dir` Pfad in der Konfiguration (Default: `./editor-ui`) muss auf ein existierendes Verzeichnis mit der korrekten Struktur zeigen:

```
editor_ui_dir/
  word/
    index.html          ← React App Entry
    static/js/main.js   ← Build artifacts
    static/css/...
  sheet/
    index.html
  slide/
    index.html
  pdf/
    index.html
  diagram/
    index.html
```

Wenn dieses Verzeichnis fehlt, serviert der wo-docserver die statische Landing Page, die keine Editier-Funktion hat.

**Lösung:**  
1. Build-Schritt in Dockerfile/Docker-Compose sicherstellen  
2. `EDITOR_UI_DIR`-Env-Variable korrekt setzen  
3. Oder: `ServeDir`-Fallback durch `editor_ui_service()` korrigieren

**Aufwand:** Config-Prüfung, ggf. Dockerfile-Anpassung

---

## Gap 6 (🟡 MITTEL): Editor-Config per PostMessage statt Redirect

**Betroffene Dateien:**
- `core/crates/wo-docserver/src/lib.rs` (hosting_wopi_handler)
- `packages/wopi-client/src/detect-wopi-params.ts`

**Problem:**  
Der `hosting_wopi_handler` setzt `window.__WORLD_OFFICE_CONFIG__` im Redirect-HTML, aber dieser Wert geht beim `window.location.replace` verloren (neue Seite, neuer JS-Kontext). Die eigentliche Config wird über die URL-Parameter transportiert — das funktioniert, ist aber fragil.

**ONLYOFFICE-Referenz:** ONLYOFFICE empfängt die Config per **WOPI PostMessage** vom Host-Fenster (dem OCIS-Iframe). Der WOPI-Standard definiert `postMessage` für die Kommunikation zwischen Host und Editor-Iframe.

**Lösung (optional):**  
Den Editor in einem IFrame hosten und die WOPI-PostMessage-Schnittstelle implementieren:
- `window.postMessage({ MessageName: "DocEditorInit", Values: { ... } }, "*")`
- Auf `window.postMessage` von OCIS hören (z.B. `Close`, `Save`)

**Aufwand:** Mittel — betrifft Editor-Architektur

---

## Priorisierte Umsetzungsreihenfolge

```
1. Gap 1 (Token im Header)     → 5 Minuten, behebt "Failed to load document"
2. Gap 2 (WOPISrc in Discovery) → 5 Minuten, korrigiert URL-Templates
3. Gap 3 (CheckFileInfo)        → 30 Minuten, ergänzt Pflichtfelder
4. Gap 5 (Editor UI Pfad)      → Prüfung, ggf. 15 Minuten
5. Gap 4 (WOPISrc in POST)     → 10 Minuten, sauberer Durchstich
6. Gap 6 (PostMessage)          → Optional, für Produktionsqualität
```

---

## Test-Plan

Nach jedem Fix:
1. `lsp_diagnostics` auf geänderten Dateien
2. `cargo test -p wo-docserver -p wo-wopi` (Rust-Tests)
3. Falls TypeScript: `pnpm typecheck packages/wopi-client`
4. Manueller Test mit E2E-Stack: `npm test` in `tests/`
5. WOPI-Chain-Test: `node test-wopi-chain.js`
