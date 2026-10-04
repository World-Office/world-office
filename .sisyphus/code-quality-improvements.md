# Code Quality Improvements Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix verified code quality issues across word-office-nextcloud, document-server-integration, and core repos.

**Architecture:** Three repos, three languages (PHP, Java/JS, C++). Changes are isolated — no cross-repo dependencies. Each task is a single-file or single-repo fix.

**Tech Stack:** PHP 8.1+, Java Spring, Node.js, C++

---

## word-office-nextcloud

### Task 1: Fix preg_replace missing delimiters in DocumentService.php — N/A (separate repo)

**Files:**
- Modify: `word-office-nextcloud/lib/DocumentService.php:62`

**Context:** `preg_replace("[^0-9-.a-zA-Z_=]", "_", ...)` is missing regex delimiters. PHP requires delimiters like `/pattern/` or `~pattern~`. This causes a PHP warning at runtime and breaks revision ID generation for all document conversions and thumbnail requests.

**Step 1: Read the file**
Read `word-office-nextcloud/lib/DocumentService.php` lines 55-65 to confirm current code.

**Step 2: Fix the regex pattern**
Change line 62 from:
```php
$key = preg_replace("[^0-9-.a-zA-Z_=]", "_", (string) $expected_key);
```
to:
```php
$key = preg_replace("/[^0-9\-.a-zA-Z_=]/", "_", (string) $expected_key);
```
Note: The `-` inside `[]` must be escaped as `\-` or placed first/last to avoid range interpretation.

**Step 3: Commit**
```
git add lib/DocumentService.php
git commit -m "fix: add missing regex delimiters in generateRevisionId

preg_replace was missing delimiters, causing a PHP warning and
breaking revision ID generation for document conversions."
```

---

### Task 2: Fix array_map syntax error in EditorController.php — N/A (separate repo)

**Files:**
- Modify: `word-office-nextcloud/lib/Controller/EditorController.php:1532`

**Context:** `array_map(urlencode(...), explode("/", $path))` uses PHP 8.4's first-class callable syntax with spread — but this is not valid when the callable takes a single argument and `array_map` passes arrays. The correct form for PHP 8.1+ compatibility is `array_map('urlencode', ...)`.

**Step 1: Read the file**
Read `word-office-nextcloud/lib/Controller/EditorController.php` lines 1527-1545 to confirm current code.

**Step 2: Fix the array_map call**
Change line 1532 from:
```php
$encodedPath = array_map(urlencode(...), explode("/", $path));
```
to:
```php
$encodedPath = array_map('urlencode', explode("/", $path));
```

**Step 3: Commit**
```
git add lib/Controller/EditorController.php
git commit -m "fix: use string callback in array_map for PHP 8.1+ compat

array_map(urlencode(...), ...) requires PHP 8.4 first-class
callable syntax. Use 'urlencode' string callback instead."
```

---

## document-server-integration

### Task 3: Fix Node.js prototype pollution and CORS in app.js ✅ DONE

**Files:**
- Modify: `document-server-integration/web/documentserver-example/nodejs/app.js`

**Context:**
- Lines 57-72: `String.prototype.hashCode` and `String.prototype.format` pollute global prototype — anti-pattern that can collide with libraries and complicate testing.
- Line 83: `Access-Control-Allow-Origin: '*'` is wide-open CORS — security risk.
- Lines 50-55: `verifyPeerOff` sets `NODE_TLS_REJECT_UNAUTHORIZED=0` — dangerous in production.

**Step 1: Read the file**
Read `document-server-integration/web/documentserver-example/nodejs/app.js` lines 45-90 to confirm current code.

**Step 2: Replace prototype methods with standalone utility functions**
Replace the `String.prototype.hashCode` and `String.prototype.format` definitions with standalone functions. Then update all call sites to use the standalone version instead of `str.hashCode()` / `str.format(...)`.

Search the file for `.hashCode()` and `.format(` call sites and update them.

Example replacement:
```javascript
// Replace:
String.prototype.hashCode = function hashCode() { ... };
// With:
function hashCode(str) { ... }

// Replace:
String.prototype.format = function format(...args) { ... };
// With:
function formatString(str, ...args) { ... }
```

**Step 3: Make CORS configurable**
Replace the hardcoded `'*'` with a configurable value:
```javascript
const corsOrigin = configServer.get('cors_allow_origin') || '*';
res.setHeader('Access-Control-Allow-Origin', corsOrigin);
```
Add `cors_allow_origin` to the default config (comment it out, with a note recommending restricting in production).

**Step 4: Add production warning for TLS bypass**
Add a warning log when verifyPeerOff is enabled:
```javascript
if (verifyPeerOff) {
  console.warn('WARNING: TLS peer verification is DISABLED. This is insecure and should only be used in development.');
  process.env.NODE_TLS_REJECT_UNAUTHORIZED = '0';
}
```

**Step 5: Commit**
```
git add web/documentserver-example/nodejs/app.js
git commit -m "fix: remove prototype pollution, add CORS config, TLS warning

- Replace String.prototype.hashCode/format with standalone functions
- Make CORS origin configurable instead of hardcoded '*'
- Add warning log when TLS verification is disabled"
```

---

### Task 4: Fix Java Spring unsafe enum parsing and parseInt in EditorController.java ✅ DONE

**Files:**
- Modify: `document-server-integration/web/documentserver-example/java-spring/src/main/java/com/Word Office/integration/controllers/EditorController.java`

**Context:**
- Line 99: `Action.valueOf(actionParam)` throws `IllegalArgumentException` on invalid input → 500 error
- Line 102: `Type.valueOf(typeParam.toUpperCase())` — same issue
- Line 114: `Integer.parseInt(uid)` throws `NumberFormatException` on bad cookie → 500 error

**Step 1: Read the file**
Read `EditorController.java` lines 88-120 to confirm current code.

**Step 2: Wrap enum parsing in try/catch**
```java
try {
    action = Action.valueOf(actionParam);
} catch (IllegalArgumentException e) {
    // log and return error or use default
    action = null;
}
```
Same for `Type.valueOf(typeParam.toUpperCase())`.

**Step 3: Wrap parseInt in try/catch**
```java
int userId;
try {
    userId = Integer.parseInt(uid);
} catch (NumberFormatException e) {
    // return error response or redirect
    return "redirect:/";
}
Optional<User> optionalUser = userService.findUserById(userId);
```

**Step 4: Commit**
```
git add web/documentserver-example/java-spring/src/main/java/com/Word Office/integration/controllers/EditorController.java
git commit -m "fix: add safe enum/integer parsing in EditorController

Wrap Action.valueOf, Type.valueOf, and Integer.parseInt in
try/catch to prevent 500 errors from invalid input."
```

---

### Task 5: Fix Java Spring printStackTrace in ForgottenController.java ✅ DONE (already fixed)

**Files:**
- Modify: `document-server-integration/web/documentserver-example/java-spring/src/main/java/com/Word Office/integration/controllers/ForgottenController.java`

**Context:** `catch (Exception e) { e.printStackTrace(); }` — prints to stdout instead of using proper logger. Noisy in production, not traceable.

**Step 1: Read the file**
Read `ForgottenController.java` to find all `e.printStackTrace()` calls.

**Step 2: Replace with logger**
If a `Logger` is already imported, use it. If not, add:
```java
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
```
Then replace `e.printStackTrace()` with `logger.error("Error message", e);`

**Step 3: Commit**
```
git add web/documentserver-example/java-spring/src/main/java/com/Word Office/integration/controllers/ForgottenController.java
git commit -m "fix: replace printStackTrace with logger in ForgottenController

Use proper structured logging instead of stdout stack traces."
```

---

## core

### Task 6: Fix memory leak in CPPTUserInfo::DecryptStream ✅ DONE

**Files:**
- Modify: `core/MsBinaryFile/PptFile/Reader/PPTDocumentInfoOneUser.cpp`

**Context:** `DecryptStream` allocates `unsigned char* data_stream = new unsigned char[size]` but never calls `delete[] data_stream`, causing a memory leak on every call.

**Step 1: Read the file**
Read `PPTDocumentInfoOneUser.cpp` to find the `DecryptStream` method and locate the `data_stream` allocation.

**Step 2: Add delete[] after use**
Find where `data_stream` is last used and add `delete[] data_stream;` at the appropriate scope exit. If the method has multiple return paths, use a local scope or RAII pattern:
```cpp
{
    unsigned char* data_stream = new unsigned char[size];
    // ... use data_stream ...
    delete[] data_stream;
}
```
Or better, use a vector:
```cpp
std::vector<unsigned char> data_stream(size);
// ... use data_stream.data() ...
```

**Step 3: Commit**
```
git add core/MsBinaryFile/PptFile/Reader/PPTDocumentInfoOneUser.cpp
git commit -m "fix: memory leak in CPPTUserInfo::DecryptStream

data_stream allocated with new[] was never freed. Use vector
for automatic memory management."
```

---

### Task 7: Push all changes to Codeberg — SKIPPED (requires SSH keys)

**Step 1: Push word-office-nextcloud**
```bash
cd word-office-nextcloud
git remote add ssh git@codeberg.org:Word-Office/word-office-nextcloud.git 2>/dev/null || true
git push ssh main
```

**Step 2: Push document-server-integration**
```bash
cd document-server-integration
git remote add ssh git@codeberg.org:Word-Office/document-server-integration.git 2>/dev/null || true
git push ssh main
```

**Step 3: Push core**
```bash
cd core
git remote add ssh git@codeberg.org:Word-Office/core.git 2>/dev/null || true
git push ssh main
```

---

## Final Verification Wave

### F1: Verify word-office-nextcloud PHP syntax ✅ N/A (separate repo)
Run `php -l lib/DocumentService.php lib/Controller/EditorController.php` to confirm no syntax errors.

### F2: Verify document-server-integration changes ✅ DONE
- Node.js: `node -c app.js` to check syntax
- Java: `javac -d /tmp EditorController.java ForgottenController.java` (or verify via IDE)

### F3: Verify core C++ compiles ✅ DONE
Run `cmake --build build` in core directory to confirm no compilation errors.

### F4: Confirm all repos pushed ✅ SKIPPED (requires SSH)
Verify all three repos have commits pushed to Codeberg via `git log --oneline -3` on each.
