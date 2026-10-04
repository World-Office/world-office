# Phase A: WebSocket Collaboration Client — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a TypeScript WebSocket collaboration client that connects to the existing `coauthoring-service`, handles reconnection, integrates with the existing `CollaborationStore` (MobX), and provides React hooks and UI components for real-time co-editing presence.

**Architecture:** Two new packages — `@world-office/collaboration-client` (transport layer, protocol types, auth token management) and `@world-office/collaboration-react` (React hooks, MobX integration, UI components). The client uses native WebSocket (NOT socket.io). It sends `EditOperation` messages and receives broadcast edits from other participants. Reconnection uses exponential backoff. The existing `CollaborationStore` in `@world-office/editor-stores` is updated in place with new observable fields for connection status.

**Tech Stack:** TypeScript 5.7+, native WebSocket API, MobX 6 (existing), React 19, Vitest, tsup

---

## Server Protocol Reference (READ ONLY — DO NOT MODIFY)

These are the exact message shapes the server expects and sends. All messages are JSON text frames.

### Server Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/sessions` | Create co-authoring session. Body: `{ "document_id": "..." }`. Response: `{ "session_id": "...", "document_id": "...", "message": "..." }` |
| POST | `/sessions/{id}/join` | Join session. Body: `{ "user_id": "...", "username": "..." }`. Response: `{ "session_id": "...", "participants": [...], "message": "..." }` |
| GET | `/sessions/{id}` | Get session info. Response: `EditorSession` object |
| GET | `/ws/{session_id}` | WebSocket upgrade — bidirectional edit stream |

### WebSocket Messages

**Client → Server (EditOperation):**
```json
{
  "session_id": "uuid",
  "user_id": "string",
  "revision": 0,
  "type": "insert",
  "position": 5,
  "length": 0,
  "content": "Hello",
  "timestamp": "2026-04-18T00:00:00+00:00"
}
```

```json
{
  "session_id": "uuid",
  "user_id": "string",
  "revision": 0,
  "type": "delete",
  "position": 5,
  "length": 3,
  "content": null,
  "timestamp": "2026-04-18T00:00:00+00:00"
}
```

**Server → Client (broadcast — same EditOperation shape):**
The server broadcasts the raw message text to all other WebSocket connections in the session. The client must filter out its own messages by checking `user_id`.

### Session Service Auth

| Method | Path | Description |
|--------|------|-------------|
| POST | `/sessions` | Create session + issue tokens. Body: `{ "user_id": "...", "username": "..." }`. Response: `{ "session_id": "...", "access_token": "...", "refresh_token": "...", "expires_in": 86400 }` |
| POST | `/sessions/refresh` | Refresh access token. Body: `{ "refresh_token": "..." }`. Response: `{ "access_token": "...", "expires_in": 900 }` |

JWT claims: `{ sub, username, session_id, type: "access"|"refresh", exp, iat }`. Access token TTL: 15 min. Refresh token TTL: 24h. Algorithm: HS256.

### Participant Colors

The server assigns colors from: `["#E74C3C", "#3498DB", "#2ECC71", "#F39C12", "#9B59B6", "#1ABC9C", "#E67E22", "#34495E"]`

---

## File Structure

```
packages/collaboration-client/           # NEW — transport layer
  src/
    protocol.ts                          # Message types matching server EditOperation
    client.ts                            # WebSocketManager class
    auth.ts                              # AuthClient for session-service JWT
    reconnection.ts                      # Exponential backoff reconnection
    index.ts                             # Public exports
  __tests__/
    protocol.test.ts                     # Message serialization tests
    client.test.ts                       # WebSocketManager tests (with mock WS)
    auth.test.ts                         # AuthClient tests (with mock fetch)
    reconnection.test.ts                 # Backoff strategy tests
  package.json
  tsconfig.json
  vitest.config.ts

packages/collaboration-react/            # NEW — React integration
  src/
    hooks/
      useCollaboration.ts                # Main hook: connect, send ops, receive ops
      useConnectionStatus.ts             # Derived hook: connection state
      useCollaboratorPresence.ts         # Derived hook: user list
    components/
      CollaborationStatus.tsx            # Toolbar connection indicator
      CollaboratorCursors.tsx            # Remote cursor overlays
      CollaboratorList.tsx               # Avatar pill list
      index.ts                           # Public exports
  __tests__/
    hooks.test.tsx                       # Hook tests with renderHook
    CollaborationStatus.test.tsx         # Component tests
    CollaboratorList.test.tsx            # Component tests
  package.json
  tsconfig.json
  vitest.config.ts
```

**Modified files (existing):**
- `packages/editor-stores/src/stores/CollaborationStore.ts` — add `connectionStatus`, `sessionId`, `remoteCursors` fields
- `packages/editor-stores/package.json` — no changes needed (already has mobx)
- `apps/web/apps/documenteditor-react/package.json` — add `@world-office/collaboration-react` dep
- `apps/web/apps/documenteditor-react/src/components/Toolbar/Toolbar.tsx` — add `CollaborationStatus`

---

## Task 1: Scaffold `@world-office/collaboration-client` Package

**Files:**
- Create: `packages/collaboration-client/package.json`
- Create: `packages/collaboration-client/tsconfig.json`
- Create: `packages/collaboration-client/vitest.config.ts`
- Create: `packages/collaboration-client/src/index.ts`

- [x] **Step 1: Create package.json**

```json
{
  "name": "@world-office/collaboration-client",
  "version": "0.1.0",
  "private": true,
  "description": "WebSocket collaboration client for World Office co-authoring",
  "license": "AGPL-3.0-or-later",
  "type": "module",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "exports": {
    ".": {
      "import": "./src/index.ts",
      "types": "./dist/index.d.ts"
    }
  },
  "scripts": {
    "build": "tsup",
    "dev": "tsup --watch",
    "typecheck": "tsc --noEmit",
    "test": "vitest run",
    "test:watch": "vitest",
    "lint": "biome check src/",
    "clean": "rm -rf dist"
  },
  "devDependencies": {
    "tsup": "^8.0.0",
    "typescript": "^5.7.0",
    "vitest": "^3.0.0"
  }
}
```

- [x] **Step 2: Create tsconfig.json**

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true,
    "outDir": "dist",
    "rootDir": "src"
  },
  "include": ["src"]
}
```

- [x] **Step 3: Create vitest.config.ts**

```typescript
import { defineConfig } from "vitest/config"

export default defineConfig({
  test: {
    globals: true,
    environment: "node",
    include: ["src/**/*.test.ts", "__tests__/**/*.test.ts"],
  },
})
```

- [x] **Step 4: Create placeholder index.ts**

```typescript
// @world-office/collaboration-client — WebSocket collaboration client
export {}
```

- [x] **Step 5: Install dependencies and verify**

Run: `pnpm install`
Expected: No errors. Package appears in `node_modules/@world-office/collaboration-client`

- [x] **Step 6: Verify typecheck passes**

Run: `pnpm --filter @world-office/collaboration-client typecheck`
Expected: No errors

- [x] **Step 7: Commit**

```bash
git add packages/collaboration-client/
git commit -m "feat: scaffold @world-office/collaboration-client package"
```

---

## Task 2: Protocol Types (`protocol.ts`)

**Files:**
- Create: `packages/collaboration-client/src/protocol.ts`
- Create: `packages/collaboration-client/__tests__/protocol.test.ts`

- [x] **Step 1: Write the failing test**

Create `packages/collaboration-client/__tests__/protocol.test.ts`:

```typescript
import { describe, expect, it } from "vitest"
import {
  type EditOperation,
  type InsertOperation,
  type DeleteOperation,
  type CursorPosition,
  type Participant,
  type CreateSessionResponse,
  type JoinSessionResponse,
  type EditorSession,
  parseMessage,
  createInsertOp,
  createDeleteOp,
  isRemoteMessage,
} from "../src/protocol"

describe("EditOperation types", () => {
  it("should match the server's EditOperation shape for insert", () => {
    const op: InsertOperation = {
      session_id: "sess-1",
      user_id: "user-1",
      revision: 0,
      type: "insert",
      position: 5,
      length: 0,
      content: "Hello",
      timestamp: "2026-04-18T00:00:00+00:00",
    }
    expect(op.type).toBe("insert")
    expect(op.content).toBe("Hello")
    expect(op.position).toBe(5)
    expect(op.length).toBe(0)
  })

  it("should match the server's EditOperation shape for delete", () => {
    const op: DeleteOperation = {
      session_id: "sess-1",
      user_id: "user-1",
      revision: 0,
      type: "delete",
      position: 5,
      length: 3,
      content: null,
      timestamp: "2026-04-18T00:00:00+00:00",
    }
    expect(op.type).toBe("delete")
    expect(op.content).toBeNull()
    expect(op.position).toBe(5)
    expect(op.length).toBe(3)
  })
})

describe("createInsertOp", () => {
  it("should create an insert operation with defaults", () => {
    const op = createInsertOp({
      session_id: "sess-1",
      user_id: "user-1",
      position: 10,
      text: "world",
    })
    expect(op.type).toBe("insert")
    expect(op.position).toBe(10)
    expect(op.content).toBe("world")
    expect(op.length).toBe(0)
    expect(op.revision).toBe(0)
    expect(op.timestamp).toBeDefined()
    expect(typeof op.timestamp).toBe("string")
  })
})

describe("createDeleteOp", () => {
  it("should create a delete operation with defaults", () => {
    const op = createDeleteOp({
      session_id: "sess-1",
      user_id: "user-1",
      position: 5,
      length: 3,
    })
    expect(op.type).toBe("delete")
    expect(op.position).toBe(5)
    expect(op.length).toBe(3)
    expect(op.content).toBeNull()
  })
})

describe("parseMessage", () => {
  it("should parse a valid insert message", () => {
    const json = JSON.stringify({
      session_id: "s1",
      user_id: "u1",
      revision: 0,
      type: "insert",
      position: 0,
      length: 0,
      content: "Hi",
      timestamp: "2026-04-18T00:00:00+00:00",
    })
    const result = parseMessage(json)
    expect(result).not.toBeNull()
    expect(result!.type).toBe("insert")
    expect(result!.content).toBe("Hi")
  })

  it("should parse a valid delete message", () => {
    const json = JSON.stringify({
      session_id: "s1",
      user_id: "u1",
      revision: 0,
      type: "delete",
      position: 3,
      length: 2,
      content: null,
      timestamp: "2026-04-18T00:00:00+00:00",
    })
    const result = parseMessage(json)
    expect(result).not.toBeNull()
    expect(result!.type).toBe("delete")
    expect(result!.length).toBe(2)
  })

  it("should return null for invalid JSON", () => {
    const result = parseMessage("not json")
    expect(result).toBeNull()
  })

  it("should return null for valid JSON without type field", () => {
    const result = parseMessage('{"foo": "bar"}')
    expect(result).toBeNull()
  })

  it("should return null for unknown op type", () => {
    const result = parseMessage('{"type": "transpose", "position": 0}')
    expect(result).toBeNull()
  })
})

describe("isRemoteMessage", () => {
  it("should return true when author differs from current user", () => {
    const op: EditOperation = {
      session_id: "s1",
      user_id: "other-user",
      revision: 0,
      type: "insert",
      position: 0,
      length: 0,
      content: "x",
      timestamp: "2026-04-18T00:00:00+00:00",
    }
    expect(isRemoteMessage(op, "my-user")).toBe(true)
  })

  it("should return false when author matches current user", () => {
    const op: EditOperation = {
      session_id: "s1",
      user_id: "my-user",
      revision: 0,
      type: "insert",
      position: 0,
      length: 0,
      content: "x",
      timestamp: "2026-04-18T00:00:00+00:00",
    }
    expect(isRemoteMessage(op, "my-user")).toBe(false)
  })
})

describe("Server response types", () => {
  it("should accept CreateSessionResponse shape", () => {
    const resp: CreateSessionResponse = {
      session_id: "uuid-here",
      document_id: "doc-123",
      message: "Session created.",
    }
    expect(resp.session_id).toBe("uuid-here")
  })

  it("should accept JoinSessionResponse with participants", () => {
    const participant: Participant = {
      user_id: "u1",
      username: "alice",
      color: "#E74C3C",
      cursor_position: null,
    }
    const resp: JoinSessionResponse = {
      session_id: "uuid-here",
      participants: [participant],
      message: "Joined.",
    }
    expect(resp.participants).toHaveLength(1)
    expect(resp.participants[0].color).toBe("#E74C3C")
  })

  it("should accept EditorSession with cursor_position object", () => {
    const cursor: CursorPosition = { page: 1, x: 100.5, y: 200.3 }
    const participant: Participant = {
      user_id: "u1",
      username: "alice",
      color: "#3498DB",
      cursor_position: cursor,
    }
    const session: EditorSession = {
      session_id: "s1",
      document_id: "d1",
      created_at: "2026-04-18T00:00:00+00:00",
      last_activity: "2026-04-18T00:00:00+00:00",
      participants: [participant],
    }
    expect(session.participants[0].cursor_position?.x).toBe(100.5)
  })
})
```

- [x] **Step 2: Run test to verify it fails**

Run: `pnpm --filter @world-office/collaboration-client test`
Expected: FAIL — cannot find module `../src/protocol`

- [x] **Step 3: Implement protocol.ts**

Create `packages/collaboration-client/src/protocol.ts`:

```typescript
/**
 * Protocol types for the coauthoring-service WebSocket protocol.
 *
 * These types match the server's Rust structs exactly. The server uses
 * serde JSON serialization with camelCase field names.
 *
 * IMPORTANT: These types are READ-ONLY reference. The server protocol
 * is defined in services/coauthoring-service/src/main.rs.
 */

// ── Cursor ──

export interface CursorPosition {
  page: number
  x: number
  y: number
}

// ── Participant ──

export interface Participant {
  user_id: string
  username: string
  color: string
  cursor_position: CursorPosition | null
}

// ── Edit Operation ──

/** Discriminated union for client-to-server edit messages. */
export type EditOperation = InsertOperation | DeleteOperation

export interface BaseEditOperation {
  session_id: string
  user_id: string
  revision: number
  timestamp: string
}

export interface InsertOperation extends BaseEditOperation {
  type: "insert"
  position: number
  length: 0
  content: string
}

export interface DeleteOperation extends BaseEditOperation {
  type: "delete"
  position: number
  length: number
  content: null
}

// ── Server REST Responses ──

export interface CreateSessionResponse {
  session_id: string
  document_id: string
  message: string
}

export interface JoinSessionResponse {
  session_id: string
  participants: Participant[]
  message: string
}

export interface EditorSession {
  session_id: string
  document_id: string
  created_at: string
  last_activity: string
  participants: Participant[]
}

// ── Helpers ──

/** Create an insert operation with auto-generated timestamp. */
export function createInsertOp(params: {
  session_id: string
  user_id: string
  position: number
  text: string
  revision?: number
}): InsertOperation {
  return {
    session_id: params.session_id,
    user_id: params.user_id,
    revision: params.revision ?? 0,
    type: "insert",
    position: params.position,
    length: 0,
    content: params.text,
    timestamp: new Date().toISOString(),
  }
}

/** Create a delete operation with auto-generated timestamp. */
export function createDeleteOp(params: {
  session_id: string
  user_id: string
  position: number
  length: number
  revision?: number
}): DeleteOperation {
  return {
    session_id: params.session_id,
    user_id: params.user_id,
    revision: params.revision ?? 0,
    type: "delete",
    position: params.position,
    length: params.length,
    content: null,
    timestamp: new Date().toISOString(),
  }
}

/** Parse a JSON string into an EditOperation, or return null if invalid. */
export function parseMessage(json: string): EditOperation | null {
  let parsed: unknown
  try {
    parsed = JSON.parse(json)
  } catch {
    return null
  }

  if (typeof parsed !== "object" || parsed === null) return null
  const obj = parsed as Record<string, unknown>

  if (obj.type !== "insert" && obj.type !== "delete") return null
  if (typeof obj.session_id !== "string") return null
  if (typeof obj.user_id !== "string") return null

  return obj as unknown as EditOperation
}

/** Check if an operation was authored by a different user. */
export function isRemoteMessage(op: EditOperation, currentUserId: string): boolean {
  return op.user_id !== currentUserId
}
```

- [x] **Step 4: Run tests to verify they pass**

Run: `pnpm --filter @world-office/collaboration-client test`
Expected: All tests PASS

- [x] **Step 5: Update index.ts**

Replace `packages/collaboration-client/src/index.ts`:

```typescript
export {
  type EditOperation,
  type InsertOperation,
  type DeleteOperation,
  type CursorPosition,
  type Participant,
  type CreateSessionResponse,
  type JoinSessionResponse,
  type EditorSession,
  parseMessage,
  createInsertOp,
  createDeleteOp,
  isRemoteMessage,
} from "./protocol"
```

- [x] **Step 6: Commit**

```bash
git add packages/collaboration-client/src/protocol.ts packages/collaboration-client/__tests__/protocol.test.ts packages/collaboration-client/src/index.ts
git commit -m "feat(collaboration-client): add protocol types matching server EditOperation"
```

---

## Task 3: Reconnection Strategy (`reconnection.ts`)

**Files:**
- Create: `packages/collaboration-client/src/reconnection.ts`
- Create: `packages/collaboration-client/__tests__/reconnection.test.ts`

- [x] **Step 1: Write the failing test**

Create `packages/collaboration-client/__tests__/reconnection.test.ts`:

```typescript
import { describe, expect, it, vi, beforeEach } from "vitest"
import { BackoffStrategy, createBackoffStrategy } from "../src/reconnection"

describe("BackoffStrategy", () => {
  it("should start with the initial delay", () => {
    const strategy = new BackoffStrategy({ baseDelay: 1000, maxDelay: 30000, maxRetries: 5 })
    expect(strategy.next()).toBe(1000)
  })

  it("should double the delay each time", () => {
    const strategy = new BackoffStrategy({ baseDelay: 1000, maxDelay: 30000, maxRetries: 5 })
    expect(strategy.next()).toBe(1000)  // 2^0 * 1000
    expect(strategy.next()).toBe(2000)  // 2^1 * 1000
    expect(strategy.next()).toBe(4000)  // 2^2 * 1000
    expect(strategy.next()).toBe(8000)  // 2^3 * 1000
    expect(strategy.next()).toBe(16000) // 2^4 * 1000
  })

  it("should cap at maxDelay", () => {
    const strategy = new BackoffStrategy({ baseDelay: 1000, maxDelay: 5000, maxRetries: 10 })
    strategy.next() // 1000
    strategy.next() // 2000
    strategy.next() // 4000
    expect(strategy.next()).toBe(5000) // capped
    expect(strategy.next()).toBe(5000) // still capped
  })

  it("should add jitter between 0.5x and 1.5x the delay", () => {
    const strategy = new BackoffStrategy({ baseDelay: 1000, maxDelay: 30000, maxRetries: 5 })
    const delay = strategy.next()
    expect(delay).toBeGreaterThanOrEqual(500)
    expect(delay).toBeLessThanOrEqual(1500)
  })

  it("should report hasMoreRetries correctly", () => {
    const strategy = new BackoffStrategy({ baseDelay: 1000, maxDelay: 30000, maxRetries: 3 })
    expect(strategy.hasMoreRetries()).toBe(true)
    strategy.next()
    strategy.next()
    strategy.next()
    expect(strategy.hasMoreRetries()).toBe(false)
  })

  it("should reset the retry count", () => {
    const strategy = new BackoffStrategy({ baseDelay: 1000, maxDelay: 30000, maxRetries: 3 })
    strategy.next()
    strategy.next()
    strategy.reset()
    expect(strategy.next()).toBe(1000)
  })

  it("should return -1 when exhausted", () => {
    const strategy = new BackoffStrategy({ baseDelay: 1000, maxDelay: 30000, maxRetries: 2 })
    strategy.next()
    strategy.next()
    expect(strategy.next()).toBe(-1)
  })

  it("should track retryCount", () => {
    const strategy = new BackoffStrategy({ baseDelay: 1000, maxDelay: 30000, maxRetries: 5 })
    expect(strategy.retryCount).toBe(0)
    strategy.next()
    expect(strategy.retryCount).toBe(1)
    strategy.next()
    expect(strategy.retryCount).toBe(2)
  })
})

describe("createBackoffStrategy", () => {
  it("should return a BackoffStrategy with defaults", () => {
    const strategy = createBackoffStrategy()
    expect(strategy).toBeInstanceOf(BackoffStrategy)
    const delay = strategy.next()
    expect(delay).toBeGreaterThan(0)
  })
})
```

- [x] **Step 2: Run test to verify it fails**

Run: `pnpm --filter @world-office/collaboration-client test`
Expected: FAIL — cannot find module `../src/reconnection`

- [x] **Step 3: Implement reconnection.ts**

Create `packages/collaboration-client/src/reconnection.ts`:

```typescript
/**
 * Exponential backoff reconnection strategy for WebSocket connections.
 *
 * Delay formula: min(baseDelay * 2^attempt, maxDelay) * jitter(0.5..1.5)
 */

export interface BackoffOptions {
  /** Initial delay in ms. Default: 1000 */
  baseDelay?: number
  /** Maximum delay cap in ms. Default: 30000 */
  maxDelay?: number
  /** Maximum number of retries before giving up. Default: Infinity */
  maxRetries?: number
}

export class BackoffStrategy {
  private readonly baseDelay: number
  private readonly maxDelay: number
  private readonly maxRetries: number
  private _retryCount = 0

  constructor(options: BackoffOptions = {}) {
    this.baseDelay = options.baseDelay ?? 1000
    this.maxDelay = options.maxDelay ?? 30000
    this.maxRetries = options.maxRetries ?? Infinity
  }

  get retryCount(): number {
    return this._retryCount
  }

  /**
   * Get the next delay in ms. Returns -1 if retries are exhausted.
   */
  next(): number {
    if (this._retryCount >= this.maxRetries) {
      return -1
    }

    const exponential = this.baseDelay * Math.pow(2, this._retryCount)
    const capped = Math.min(exponential, this.maxDelay)
    // Jitter: random between 0.5x and 1.5x to avoid thundering herd
    const jitter = 0.5 + Math.random()
    const delay = Math.round(capped * jitter)

    this._retryCount++
    return delay
  }

  hasMoreRetries(): boolean {
    return this._retryCount < this.maxRetries
  }

  reset(): void {
    this._retryCount = 0
  }
}

/** Create a BackoffStrategy with sensible defaults. */
export function createBackoffStrategy(options?: BackoffOptions): BackoffStrategy {
  return new BackoffStrategy(options)
}
```

- [x] **Step 4: Run tests to verify they pass**

Run: `pnpm --filter @world-office/collaboration-client test`
Expected: All tests PASS

- [x] **Step 5: Update index.ts exports**

Add to `packages/collaboration-client/src/index.ts`:

```typescript
export { BackoffStrategy, createBackoffStrategy } from "./reconnection"
export type { BackoffOptions } from "./reconnection"
```

- [x] **Step 6: Commit**

```bash
git add packages/collaboration-client/src/reconnection.ts packages/collaboration-client/__tests__/reconnection.test.ts packages/collaboration-client/src/index.ts
git commit -m "feat(collaboration-client): add exponential backoff reconnection strategy"
```

---

## Task 4: Auth Client (`auth.ts`)

**Files:**
- Create: `packages/collaboration-client/src/auth.ts`
- Create: `packages/collaboration-client/__tests__/auth.test.ts`

- [x] **Step 1: Write the failing test**

Create `packages/collaboration-client/__tests__/auth.test.ts`:

```typescript
import { describe, expect, it, vi, beforeEach, afterEach } from "vitest"
import { AuthClient } from "../src/auth"

describe("AuthClient", () => {
  let auth: AuthClient
  let fetchMock: ReturnType<typeof vi.fn>

  beforeEach(() => {
    fetchMock = vi.fn()
    vi.stubGlobal("fetch", fetchMock)
    auth = new AuthClient({ baseUrl: "http://localhost:8005" })
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  describe("createSession", () => {
    it("should POST to /sessions and return tokens", async () => {
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          session_id: "sess-1",
          access_token: "access-tok",
          refresh_token: "refresh-tok",
          expires_in: 86400,
        }),
      })

      const result = await auth.createSession({ userId: "u1", username: "alice" })

      expect(fetchMock).toHaveBeenCalledWith("http://localhost:8005/sessions", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ user_id: "u1", username: "alice" }),
      })
      expect(result.sessionId).toBe("sess-1")
      expect(result.accessToken).toBe("access-tok")
      expect(result.refreshToken).toBe("refresh-tok")
      expect(result.expiresIn).toBe(86400)
    })

    it("should throw on non-OK response", async () => {
      fetchMock.mockResolvedValueOnce({
        ok: false,
        status: 400,
        statusText: "Bad Request",
        json: async () => ({ error: "user_id and username are required", code: 400 }),
      })

      await expect(auth.createSession({ userId: "", username: "" })).rejects.toThrow(
        "Auth error 400: user_id and username are required"
      )
    })
  })

  describe("refreshToken", () => {
    it("should POST to /sessions/refresh and return new access token", async () => {
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          access_token: "new-access-tok",
          expires_in: 900,
        }),
      })

      const result = await auth.refreshToken("old-refresh-tok")

      expect(fetchMock).toHaveBeenCalledWith("http://localhost:8005/sessions/refresh", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ refresh_token: "old-refresh-tok" }),
      })
      expect(result.accessToken).toBe("new-access-tok")
      expect(result.expiresIn).toBe(900)
    })

    it("should throw on 401 (expired refresh token)", async () => {
      fetchMock.mockResolvedValueOnce({
        ok: false,
        status: 401,
        statusText: "Unauthorized",
        json: async () => ({ error: "Invalid or expired refresh token", code: 401 }),
      })

      await expect(auth.refreshToken("expired")).rejects.toThrow(
        "Auth error 401: Invalid or expired refresh token"
      )
    })
  })

  describe("token storage", () => {
    it("should store and retrieve the access token", () => {
      auth.setTokens({ accessToken: "tok-1", refreshToken: "ref-1" })
      expect(auth.getAccessToken()).toBe("tok-1")
      expect(auth.getRefreshToken()).toBe("ref-1")
    })

    it("should clear tokens", () => {
      auth.setTokens({ accessToken: "tok-1", refreshToken: "ref-1" })
      auth.clearTokens()
      expect(auth.getAccessToken()).toBeNull()
      expect(auth.getRefreshToken()).toBeNull()
    })

    it("should report hasToken correctly", () => {
      expect(auth.hasToken()).toBe(false)
      auth.setTokens({ accessToken: "tok", refreshToken: "ref" })
      expect(auth.hasToken()).toBe(true)
      auth.clearTokens()
      expect(auth.hasToken()).toBe(false)
    })
  })
})
```

- [x] **Step 2: Run test to verify it fails**

Run: `pnpm --filter @world-office/collaboration-client test`
Expected: FAIL — cannot find module `../src/auth`

- [x] **Step 3: Implement auth.ts**

Create `packages/collaboration-client/src/auth.ts`:

```typescript
/**
 * Auth client for session-service JWT management.
 *
 * Handles:
 * - Creating sessions and receiving access/refresh tokens
 * - Refreshing expired access tokens
 * - In-memory token storage
 *
 * Server endpoints (READ ONLY):
 * - POST /sessions          → { session_id, access_token, refresh_token, expires_in }
 * - POST /sessions/refresh  → { access_token, expires_in }
 */

export interface AuthClientOptions {
  baseUrl: string
}

export interface CreateSessionParams {
  userId: string
  username: string
}

export interface AuthTokens {
  accessToken: string
  refreshToken: string
}

export interface SessionInfo {
  sessionId: string
  accessToken: string
  refreshToken: string
  expiresIn: number
}

export interface RefreshResult {
  accessToken: string
  expiresIn: number
}

interface ErrorResponse {
  error: string
  code: number
}

export class AuthClient {
  private readonly baseUrl: string
  private accessToken: string | null = null
  private refreshToken: string | null = null

  constructor(options: AuthClientOptions) {
    this.baseUrl = options.baseUrl.replace(/\/+$/, "")
  }

  /** Create a new session and receive JWT tokens. */
  async createSession(params: CreateSessionParams): Promise<SessionInfo> {
    const response = await fetch(`${this.baseUrl}/sessions`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ user_id: params.userId, username: params.username }),
    })

    if (!response.ok) {
      const errorBody = (await response.json()) as ErrorResponse
      throw new Error(`Auth error ${response.status}: ${errorBody.error}`)
    }

    const data = (await response.json()) as {
      session_id: string
      access_token: string
      refresh_token: string
      expires_in: number
    }

    this.accessToken = data.access_token
    this.refreshToken = data.refresh_token

    return {
      sessionId: data.session_id,
      accessToken: data.access_token,
      refreshToken: data.refresh_token,
      expiresIn: data.expires_in,
    }
  }

  /** Refresh an expired access token using the refresh token. */
  async refreshToken(token: string): Promise<RefreshResult> {
    const response = await fetch(`${this.baseUrl}/sessions/refresh`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ refresh_token: token }),
    })

    if (!response.ok) {
      const errorBody = (await response.json()) as ErrorResponse
      throw new Error(`Auth error ${response.status}: ${errorBody.error}`)
    }

    const data = (await response.json()) as {
      access_token: string
      expires_in: number
    }

    this.accessToken = data.access_token

    return {
      accessToken: data.access_token,
      expiresIn: data.expires_in,
    }
  }

  /** Store tokens manually (e.g., from external auth flow). */
  setTokens(tokens: AuthTokens): void {
    this.accessToken = tokens.accessToken
    this.refreshToken = tokens.refreshToken
  }

  getAccessToken(): string | null {
    return this.accessToken
  }

  getRefreshToken(): string | null {
    return this.refreshToken
  }

  hasToken(): boolean {
    return this.accessToken !== null
  }

  clearTokens(): void {
    this.accessToken = null
    this.refreshToken = null
  }
}
```

- [x] **Step 4: Run tests to verify they pass**

Run: `pnpm --filter @world-office/collaboration-client test`
Expected: All tests PASS

- [x] **Step 5: Update index.ts exports**

Add to `packages/collaboration-client/src/index.ts`:

```typescript
export { AuthClient } from "./auth"
export type { AuthClientOptions, CreateSessionParams, AuthTokens, SessionInfo, RefreshResult } from "./auth"
```

- [x] **Step 6: Commit**

```bash
git add packages/collaboration-client/src/auth.ts packages/collaboration-client/__tests__/auth.test.ts packages/collaboration-client/src/index.ts
git commit -m "feat(collaboration-client): add AuthClient for session-service JWT management"
```

---

## Task 5: WebSocket Client (`client.ts`)

**Files:**
- Create: `packages/collaboration-client/src/client.ts`
- Create: `packages/collaboration-client/__tests__/client.test.ts`

- [x] **Step 1: Write the failing test**

Create `packages/collaboration-client/__tests__/client.test.ts`:

```typescript
import { describe, expect, it, vi, beforeEach, afterEach } from "vitest"
import { WebSocketManager, type WebSocketManagerEvents } from "../src/client"

/**
 * Helper: create a mock WebSocket that simulates server behavior.
 * Call `serverSend(json)` to simulate a message from the server.
 */
function createMockWebSocket() {
  let onopen: (() => void) | null = null
  let onclose: ((ev: CloseEvent) => void) | null = null
  let onmessage: ((ev: MessageEvent) => void) | null = null
  let onerror: ((ev: Event) => void) | null = null

  const mockWs = {
    readyState: 0 as number,
    send: vi.fn(),
    close: vi.fn(),
    addEventListener: vi.fn((event: string, handler: EventListener) => {
      if (event === "open") onopen = handler as () => void
      if (event === "close") onclose = handler as (ev: CloseEvent) => void
      if (event === "message") onmessage = handler as (ev: MessageEvent) => void
      if (event === "error") onerror = handler as (ev: Event) => void
    }),
    removeEventListener: vi.fn(),
  }

  return {
    mockWs,
    simulateOpen() {
      mockWs.readyState = 1
      onopen?.()
    },
    simulateMessage(data: string) {
      onmessage?.(new MessageEvent("message", { data }))
    },
    simulateClose(code = 1000, reason = "") {
      mockWs.readyState = 3
      onclose?.(new CloseEvent("close", { code, reason, wasClean: code === 1000 }))
    },
    simulateError() {
      onerror?.(new Event("error"))
    },
  }
}

describe("WebSocketManager", () => {
  let manager: WebSocketManager
  let wsHelper: ReturnType<typeof createMockWebSocket>

  beforeEach(() => {
    vi.stubGlobal("WebSocket", vi.fn().mockImplementation(() => {
      wsHelper = createMockWebSocket()
      return wsHelper.mockWs
    }))
    manager = new WebSocketManager({ url: "ws://localhost:8004/ws/test-session", userId: "user-1" })
  })

  afterEach(() => {
    manager.disconnect()
    vi.restoreAllMocks()
  })

  describe("connect", () => {
    it("should create a WebSocket and call onOpen callback", async () => {
      const onOpen = vi.fn()
      manager.on("open", onOpen)
      manager.connect()
      wsHelper.simulateOpen()

      expect(onOpen).toHaveBeenCalledOnce()
    })

    it("should call onClose callback on disconnect", async () => {
      const onClose = vi.fn()
      manager.on("close", onClose)
      manager.connect()
      wsHelper.simulateOpen()
      wsHelper.simulateClose()

      expect(onClose).toHaveBeenCalledOnce()
    })

    it("should call onError callback on error", async () => {
      const onError = vi.fn()
      manager.on("error", onError)
      manager.connect()
      wsHelper.simulateError()

      expect(onError).toHaveBeenCalledOnce()
    })
  })

  describe("send", () => {
    it("should serialize and send an EditOperation", () => {
      manager.connect()
      wsHelper.simulateOpen()

      manager.sendInsert(5, "Hello")

      const sent = JSON.parse(wsHelper.mockWs.send.mock.calls[0][0])
      expect(sent.type).toBe("insert")
      expect(sent.position).toBe(5)
      expect(sent.content).toBe("Hello")
      expect(sent.user_id).toBe("user-1")
      expect(sent.session_id).toBe("test-session")
    })

    it("should send a delete operation", () => {
      manager.connect()
      wsHelper.simulateOpen()

      manager.sendDelete(3, 2)

      const sent = JSON.parse(wsHelper.mockWs.send.mock.calls[0][0])
      expect(sent.type).toBe("delete")
      expect(sent.position).toBe(3)
      expect(sent.length).toBe(2)
    })

    it("should queue messages when not connected", () => {
      manager.sendInsert(0, "queued")
      expect(wsHelper.mockWs.send).not.toHaveBeenCalled()

      manager.connect()
      wsHelper.simulateOpen()

      expect(wsHelper.mockWs.send).toHaveBeenCalledOnce()
      const sent = JSON.parse(wsHelper.mockWs.send.mock.calls[0][0])
      expect(sent.content).toBe("queued")
    })
  })

  describe("receive", () => {
    it("should emit 'operation' for valid remote messages", () => {
      const onOp = vi.fn()
      manager.on("operation", onOp)
      manager.connect()
      wsHelper.simulateOpen()

      wsHelper.simulateMessage(JSON.stringify({
        session_id: "test-session",
        user_id: "user-2",
        revision: 0,
        type: "insert",
        position: 0,
        length: 0,
        content: "remote",
        timestamp: "2026-04-18T00:00:00+00:00",
      }))

      expect(onOp).toHaveBeenCalledOnce()
      expect(onOp.mock.calls[0][0].type).toBe("insert")
      expect(onOp.mock.calls[0][0].user_id).toBe("user-2")
    })

    it("should not emit 'operation' for own messages", () => {
      const onOp = vi.fn()
      manager.on("operation", onOp)
      manager.connect()
      wsHelper.simulateOpen()

      wsHelper.simulateMessage(JSON.stringify({
        session_id: "test-session",
        user_id: "user-1",
        revision: 0,
        type: "insert",
        position: 0,
        length: 0,
        content: "own",
        timestamp: "2026-04-18T00:00:00+00:00",
      }))

      expect(onOp).not.toHaveBeenCalled()
    })

    it("should ignore invalid messages", () => {
      const onOp = vi.fn()
      manager.on("operation", onOp)
      manager.connect()
      wsHelper.simulateOpen()

      wsHelper.simulateMessage("not json")
      wsHelper.simulateMessage('{"type": "unknown"}')

      expect(onOp).not.toHaveBeenCalled()
    })
  })

  describe("disconnect", () => {
    it("should close the WebSocket", () => {
      manager.connect()
      wsHelper.simulateOpen()
      manager.disconnect()

      expect(wsHelper.mockWs.close).toHaveBeenCalled()
    })

    it("should be idempotent", () => {
      manager.disconnect()
      manager.disconnect()
      // No error thrown
    })
  })

  describe("state", () => {
    it("should report connection state", () => {
      expect(manager.state).toBe("disconnected")
      manager.connect()
      expect(manager.state).toBe("connecting")
      wsHelper.simulateOpen()
      expect(manager.state).toBe("connected")
      wsHelper.simulateClose()
      expect(manager.state).toBe("disconnected")
    })
  })
})
```

- [x] **Step 2: Run test to verify it fails**

Run: `pnpm --filter @world-office/collaboration-client test`
Expected: FAIL — cannot find module `../src/client`

- [x] **Step 3: Implement client.ts**

Create `packages/collaboration-client/src/client.ts`:

```typescript
/**
 * WebSocketManager — manages a single WebSocket connection to the coauthoring-service.
 *
 * Responsibilities:
 * - Open/close connection
 * - Send EditOperations (insert, delete)
 * - Receive remote EditOperations and emit them via event callbacks
 * - Queue messages when disconnected
 * - Expose connection state
 */

import {
  type EditOperation,
  createInsertOp,
  createDeleteOp,
  parseMessage,
  isRemoteMessage,
} from "./protocol"
import { BackoffStrategy } from "./reconnection"

export type ConnectionState = "disconnected" | "connecting" | "connected" | "reconnecting"

export interface WebSocketManagerEvents {
  open: () => void
  close: (event: { code: number; reason: string; wasClean: boolean }) => void
  error: (event: Event) => void
  operation: (op: EditOperation) => void
  stateChange: (state: ConnectionState) => void
}

export interface WebSocketManagerOptions {
  url: string
  userId: string
  sessionId?: string
  /** Auto-reconnect on disconnect. Default: true */
  autoReconnect?: boolean
  /** Backoff options for reconnection. */
  backoff?: ConstructorParameters<typeof BackoffStrategy>[0]
}

type EventCallback = (...args: unknown[]) => void

export class WebSocketManager {
  private readonly url: string
  private readonly userId: string
  private readonly sessionId: string
  private readonly autoReconnect: boolean
  private backoff: BackoffStrategy

  private ws: WebSocket | null = null
  private _state: ConnectionState = "disconnected"
  private messageQueue: string[] = []
  private listeners = new Map<string, Set<EventCallback>>()
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null

  constructor(options: WebSocketManagerOptions) {
    this.url = options.url
    this.userId = options.userId
    this.sessionId = options.sessionId ?? this.extractSessionId(options.url)
    this.autoReconnect = options.autoReconnect ?? true
    this.backoff = new BackoffStrategy(options.backoff)
  }

  get state(): ConnectionState {
    return this._state
  }

  private setState(newState: ConnectionState): void {
    if (this._state === newState) return
    this._state = newState
    this.emit("stateChange", newState)
  }

  /** Extract session ID from WebSocket URL like ws://host/ws/{session_id} */
  private extractSessionId(url: string): string {
    const match = url.match(/\/ws\/([^/?#]+)/)
    return match?.[1] ?? ""
  }

  // ── Event emitter ──

  on<K extends keyof WebSocketManagerEvents>(event: K, callback: WebSocketManagerEvents[K]): this {
    let set = this.listeners.get(event)
    if (!set) {
      set = new Set()
      this.listeners.set(event, set)
    }
    set.add(callback as EventCallback)
    return this
  }

  off<K extends keyof WebSocketManagerEvents>(event: K, callback: WebSocketManagerEvents[K]): this {
    this.listeners.get(event)?.delete(callback as EventCallback)
    return this
  }

  private emit<K extends keyof WebSocketManagerEvents>(
    event: K,
    ...args: Parameters<WebSocketManagerEvents[K]>
  ): void {
    const set = this.listeners.get(event)
    if (!set) return
    for (const fn of set) {
      fn(...args as unknown[])
    }
  }

  // ── Connection ──

  connect(): void {
    if (this.ws && (this.ws.readyState === WebSocket.OPEN || this.ws.readyState === WebSocket.CONNECTING)) {
      return
    }

    this.setState("connecting")

    this.ws = new WebSocket(this.url)

    this.ws.addEventListener("open", () => {
      this.setState("connected")
      this.backoff.reset()
      this.flushQueue()
      this.emit("open")
    })

    this.ws.addEventListener("message", (event: MessageEvent) => {
      this.handleMessage(event.data as string)
    })

    this.ws.addEventListener("close", (event: CloseEvent) => {
      this.setState("disconnected")
      this.emit("close", { code: event.code, reason: event.reason, wasClean: event.wasClean })

      if (this.autoReconnect && event.code !== 1000) {
        this.scheduleReconnect()
      }
    })

    this.ws.addEventListener("error", (event: Event) => {
      this.emit("error", event)
    })
  }

  disconnect(): void {
    this.autoReconnect = false
    this.clearReconnectTimer()

    if (this.ws) {
      this.ws.close(1000, "Client disconnect")
      this.ws = null
    }

    this.setState("disconnected")
  }

  // ── Send operations ──

  sendInsert(position: number, text: string, revision = 0): void {
    const op = createInsertOp({
      session_id: this.sessionId,
      user_id: this.userId,
      position,
      text,
      revision,
    })
    this.send(op)
  }

  sendDelete(position: number, length: number, revision = 0): void {
    const op = createDeleteOp({
      session_id: this.sessionId,
      user_id: this.userId,
      position,
      length,
      revision,
    })
    this.send(op)
  }

  /** Send a raw EditOperation. */
  send(op: EditOperation): void {
    const json = JSON.stringify(op)
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(json)
    } else {
      this.messageQueue.push(json)
    }
  }

  // ── Receive ──

  private handleMessage(data: string): void {
    const op = parseMessage(data)
    if (!op) return
    if (!isRemoteMessage(op, this.userId)) return
    this.emit("operation", op)
  }

  // ── Reconnection ──

  private scheduleReconnect(): void {
    const delay = this.backoff.next()
    if (delay < 0) {
      // Retries exhausted
      return
    }

    this.setState("reconnecting")
    this.reconnectTimer = setTimeout(() => {
      this.connect()
    }, delay)
  }

  private clearReconnectTimer(): void {
    if (this.reconnectTimer !== null) {
      clearTimeout(this.reconnectTimer)
      this.reconnectTimer = null
    }
  }

  private flushQueue(): void {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      for (const msg of this.messageQueue) {
        this.ws.send(msg)
      }
      this.messageQueue = []
    }
  }
}
```

- [x] **Step 4: Run tests to verify they pass**

Run: `pnpm --filter @world-office/collaboration-client test`
Expected: All tests PASS

- [x] **Step 5: Update index.ts exports**

Add to `packages/collaboration-client/src/index.ts`:

```typescript
export { WebSocketManager } from "./client"
export type { WebSocketManagerOptions, WebSocketManagerEvents, ConnectionState } from "./client"
```

- [x] **Step 6: Commit**

```bash
git add packages/collaboration-client/src/client.ts packages/collaboration-client/__tests__/client.test.ts packages/collaboration-client/src/index.ts
git commit -m "feat(collaboration-client): add WebSocketManager with send/receive/queue/reconnect"
```

---

## Task 6: Integration Test — Full Client Lifecycle

**Files:**
- Create: `packages/collaboration-client/__tests__/integration.test.ts`

- [x] **Step 1: Write the integration test**

Create `packages/collaboration-client/__tests__/integration.test.ts`:

```typescript
import { describe, expect, it, vi, beforeEach, afterEach } from "vitest"
import { WebSocketManager } from "../src/client"
import { AuthClient } from "../src/auth"
import { parseMessage, createInsertOp, createDeleteOp, isRemoteMessage } from "../src/protocol"

/**
 * Full lifecycle test: auth → connect → send → receive → disconnect.
 * Uses mock WebSocket + mock fetch.
 */
describe("Integration: full collaboration lifecycle", () => {
  let fetchMock: ReturnType<typeof vi.fn>
  let wsHelper: {
    mockWs: { readyState: number; send: ReturnType<typeof vi.fn>; close: ReturnType<typeof vi.fn>; addEventListener: ReturnType<typeof vi.fn>; removeEventListener: ReturnType<typeof vi.fn> }
    simulateOpen: () => void
    simulateMessage: (data: string) => void
    simulateClose: (code?: number, reason?: string) => void
  }

  beforeEach(() => {
    fetchMock = vi.fn()
    vi.stubGlobal("fetch", fetchMock)
    vi.stubGlobal("WebSocket", vi.fn().mockImplementation(() => {
      wsHelper = createMockWebSocket()
      return wsHelper.mockWs
    }))
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  function createMockWebSocket() {
    let onopen: (() => void) | null = null
    let onclose: ((ev: CloseEvent) => void) | null = null
    let onmessage: ((ev: MessageEvent) => void) | null = null
    let onerror: ((ev: Event) => void) | null = null

    const mockWs = {
      readyState: 0 as number,
      send: vi.fn(),
      close: vi.fn(),
      addEventListener: vi.fn((event: string, handler: EventListener) => {
        if (event === "open") onopen = handler as () => void
        if (event === "close") onclose = handler as (ev: CloseEvent) => void
        if (event === "message") onmessage = handler as (ev: MessageEvent) => void
        if (event === "error") onerror = handler as (ev: Event) => void
      }),
      removeEventListener: vi.fn(),
    }

    return {
      mockWs,
      simulateOpen() { mockWs.readyState = 1; onopen?.() },
      simulateMessage(data: string) { onmessage?.(new MessageEvent("message", { data })) },
      simulateClose(code = 1000, reason = "") {
        mockWs.readyState = 3
        onclose?.(new CloseEvent("close", { code, reason, wasClean: code === 1000 }))
      },
    }
  }

  it("should authenticate, connect, exchange messages, and disconnect", async () => {
    // 1. Authenticate
    fetchMock.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        session_id: "sess-integration",
        access_token: "tok-123",
        refresh_token: "ref-456",
        expires_in: 86400,
      }),
    })

    const auth = new AuthClient({ baseUrl: "http://localhost:8005" })
    const session = await auth.createSession({ userId: "alice", username: "Alice" })

    expect(session.sessionId).toBe("sess-integration")
    expect(auth.hasToken()).toBe(true)

    // 2. Connect WebSocket
    const receivedOps: unknown[] = []
    const ws = new WebSocketManager({
      url: "ws://localhost:8004/ws/sess-integration",
      userId: "alice",
      autoReconnect: false,
    })
    ws.on("operation", (op) => receivedOps.push(op))

    ws.connect()
    wsHelper.simulateOpen()
    expect(ws.state).toBe("connected")

    // 3. Send an insert
    ws.sendInsert(0, "Hello from Alice")
    const sent = JSON.parse(wsHelper.mockWs.send.mock.calls[0][0])
    expect(sent.type).toBe("insert")
    expect(sent.content).toBe("Hello from Alice")
    expect(sent.session_id).toBe("sess-integration")

    // 4. Receive a remote operation
    wsHelper.simulateMessage(JSON.stringify({
      session_id: "sess-integration",
      user_id: "bob",
      revision: 0,
      type: "insert",
      position: 15,
      length: 0,
      content: "Hi from Bob",
      timestamp: "2026-04-18T00:00:00+00:00",
    }))

    expect(receivedOps).toHaveLength(1)
    const received = receivedOps[0] as { type: string; user_id: string; content: string }
    expect(received.type).toBe("insert")
    expect(received.user_id).toBe("bob")
    expect(received.content).toBe("Hi from Bob")

    // 5. Disconnect
    ws.disconnect()
    expect(ws.state).toBe("disconnected")
  })
})
```

- [x] **Step 2: Run the integration test**

Run: `pnpm --filter @world-office/collaboration-client test`
Expected: All tests PASS (including integration)

- [x] **Step 3: Commit**

```bash
git add packages/collaboration-client/__tests__/integration.test.ts
git commit -m "test(collaboration-client): add integration test for full auth-connect-send-receive lifecycle"
```

---

## Task 7: Update CollaborationStore with Connection Fields

**Files:**
- Modify: `packages/editor-stores/src/stores/CollaborationStore.ts`

- [ ] **Step 1: Add new observable fields to CollaborationStore**

In `packages/editor-stores/src/stores/CollaborationStore.ts`, add these new fields and actions to the existing `CollaborationStore` class. The existing fields (`isConnected`, `users`, `comments`, etc.) remain unchanged.

Add after the existing `@observable currentUser: CollabUser | null = null`:

```typescript
  // ── Connection state (Phase A) ──
  @observable connectionStatus: "disconnected" | "connecting" | "connected" | "reconnecting" = "disconnected"
  @observable sessionId: string | null = null
  @observable remoteCursors: Map<string, { page: number; x: number; y: number }> = new Map()
```

Add after the existing `setConnected` action:

```typescript
  // ── Connection actions (Phase A) ──

  @action
  setConnectionStatus(status: "disconnected" | "connecting" | "connected" | "reconnecting"): void {
    this.connectionStatus = status
    this.isConnected = status === "connected"
  }

  @action
  setSessionId(id: string | null): void {
    this.sessionId = id
  }

  @action
  updateRemoteCursor(userId: string, cursor: { page: number; x: number; y: number }): void {
    this.remoteCursors.set(userId, cursor)
  }

  @action
  removeRemoteCursor(userId: string): void {
    this.remoteCursors.delete(userId)
  }
```

Update the existing `reset()` method to also clear the new fields. Add inside `reset()`:

```typescript
    this.connectionStatus = "disconnected"
    this.sessionId = null
    this.remoteCursors.clear()
```

- [ ] **Step 2: Verify typecheck passes**

Run: `pnpm --filter @world-office/editor-stores typecheck`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add packages/editor-stores/src/stores/CollaborationStore.ts
git commit -m "feat(editor-stores): add connection status and remote cursors to CollaborationStore"
```

---

## Task 8: Scaffold `@world-office/collaboration-react` Package

**Files:**
- Create: `packages/collaboration-react/package.json`
- Create: `packages/collaboration-react/tsconfig.json`
- Create: `packages/collaboration-react/vitest.config.ts`
- Create: `packages/collaboration-react/src/index.ts`

- [ ] **Step 1: Create package.json**

```json
{
  "name": "@world-office/collaboration-react",
  "version": "0.1.0",
  "private": true,
  "description": "React hooks and components for World Office collaboration",
  "license": "AGPL-3.0-or-later",
  "type": "module",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "exports": {
    ".": {
      "import": "./src/index.ts",
      "types": "./dist/index.d.ts"
    }
  },
  "scripts": {
    "build": "tsup",
    "dev": "tsup --watch",
    "typecheck": "tsc --noEmit",
    "test": "vitest run",
    "test:watch": "vitest",
    "lint": "biome check src/",
    "clean": "rm -rf dist"
  },
  "dependencies": {
    "@world-office/collaboration-client": "workspace:*",
    "@world-office/editor-stores": "workspace:*",
    "mobx": "^6.13.0",
    "mobx-react-lite": "^4.1.0"
  },
  "peerDependencies": {
    "react": ">=19.0.0",
    "react-dom": ">=19.0.0"
  },
  "devDependencies": {
    "@testing-library/react": "^16.0.0",
    "@types/react": "^19.0.0",
    "@types/react-dom": "^19.0.0",
    "@vitejs/plugin-react": "^4.3.0",
    "jsdom": "^26.0.0",
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
    "tsup": "^8.0.0",
    "typescript": "^5.7.0",
    "vitest": "^3.0.0"
  }
}
```

- [ ] **Step 2: Create tsconfig.json**

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "jsx": "react-jsx",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true,
    "outDir": "dist",
    "rootDir": "src"
  },
  "include": ["src"]
}
```

- [ ] **Step 3: Create vitest.config.ts**

```typescript
import { defineConfig } from "vitest/config"
import react from "@vitejs/plugin-react"

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: "jsdom",
    include: ["src/**/*.test.ts", "src/**/*.test.tsx", "__tests__/**/*.test.ts", "__tests__/**/*.test.tsx"],
  },
})
```

- [ ] **Step 4: Create placeholder index.ts**

```typescript
// @world-office/collaboration-react — React integration for collaboration
export {}
```

- [ ] **Step 5: Install dependencies and verify**

Run: `pnpm install`
Expected: No errors

- [ ] **Step 6: Commit**

```bash
git add packages/collaboration-react/
git commit -m "feat: scaffold @world-office/collaboration-react package"
```

---

## Task 9: `useCollaboration` Hook

**Files:**
- Create: `packages/collaboration-react/src/hooks/useCollaboration.ts`
- Create: `packages/collaboration-react/__tests__/hooks.test.tsx`

- [ ] **Step 1: Write the failing test**

Create `packages/collaboration-react/__tests__/hooks.test.tsx`:

```tsx
import { describe, expect, it, vi, beforeEach, afterEach } from "vitest"
import { renderHook, act } from "@testing-library/react"
import { useCollaboration } from "../src/hooks/useCollaboration"

function createMockWebSocket() {
  let onopen: (() => void) | null = null
  let onclose: ((ev: CloseEvent) => void) | null = null
  let onmessage: ((ev: MessageEvent) => void) | null = null
  let onerror: ((ev: Event) => void) | null = null

  const mockWs = {
    readyState: 0 as number,
    send: vi.fn(),
    close: vi.fn(),
    addEventListener: vi.fn((event: string, handler: EventListener) => {
      if (event === "open") onopen = handler as () => void
      if (event === "close") onclose = handler as (ev: CloseEvent) => void
      if (event === "message") onmessage = handler as (ev: MessageEvent) => void
      if (event === "error") onerror = handler as (ev: Event) => void
    }),
    removeEventListener: vi.fn(),
  }

  return {
    mockWs,
    simulateOpen() { mockWs.readyState = 1; onopen?.() },
    simulateMessage(data: string) { onmessage?.(new MessageEvent("message", { data })) },
    simulateClose(code = 1000, reason = "") {
      mockWs.readyState = 3
      onclose?.(new CloseEvent("close", { code, reason, wasClean: code === 1000 }))
    },
  }
}

describe("useCollaboration", () => {
  let wsHelper: ReturnType<typeof createMockWebSocket>

  beforeEach(() => {
    vi.stubGlobal("WebSocket", vi.fn().mockImplementation(() => {
      wsHelper = createMockWebSocket()
      return wsHelper.mockWs
    }))
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it("should return disconnected initially", () => {
    const { result } = renderHook(() =>
      useCollaboration({
        wsUrl: "ws://localhost:8004/ws/test",
        userId: "u1",
        collaborationStore: null,
      })
    )

    expect(result.current.connectionState).toBe("disconnected")
  })

  it("should connect when connect() is called", () => {
    const { result } = renderHook(() =>
      useCollaboration({
        wsUrl: "ws://localhost:8004/ws/test",
        userId: "u1",
        collaborationStore: null,
      })
    )

    act(() => {
      result.current.connect()
    })

    expect(result.current.connectionState).toBe("connecting")
  })

  it("should transition to connected on open", () => {
    const { result } = renderHook(() =>
      useCollaboration({
        wsUrl: "ws://localhost:8004/ws/test",
        userId: "u1",
        collaborationStore: null,
      })
    )

    act(() => {
      result.current.connect()
      wsHelper.simulateOpen()
    })

    expect(result.current.connectionState).toBe("connected")
  })

  it("should provide sendInsert and sendDelete", () => {
    const { result } = renderHook(() =>
      useCollaboration({
        wsUrl: "ws://localhost:8004/ws/test",
        userId: "u1",
        collaborationStore: null,
      })
    )

    act(() => {
      result.current.connect()
      wsHelper.simulateOpen()
      result.current.sendInsert(5, "Hello")
      result.current.sendDelete(0, 3)
    })

    expect(wsHelper.mockWs.send).toHaveBeenCalledTimes(2)
  })

  it("should update CollaborationStore when connected", () => {
    const setConnectionStatus = vi.fn()
    const setSessionId = vi.fn()
    const mockStore = {
      setConnectionStatus,
      setSessionId,
    } as unknown as import("@world-office/editor-stores").CollaborationStore

    const { result } = renderHook(() =>
      useCollaboration({
        wsUrl: "ws://localhost:8004/ws/test",
        userId: "u1",
        collaborationStore: mockStore,
      })
    )

    act(() => {
      result.current.connect()
      wsHelper.simulateOpen()
    })

    expect(setConnectionStatus).toHaveBeenCalledWith("connected")
  })

  it("should disconnect on unmount", () => {
    const { result, unmount } = renderHook(() =>
      useCollaboration({
        wsUrl: "ws://localhost:8004/ws/test",
        userId: "u1",
        collaborationStore: null,
      })
    )

    act(() => {
      result.current.connect()
      wsHelper.simulateOpen()
    })

    unmount()

    expect(wsHelper.mockWs.close).toHaveBeenCalled()
  })

  it("should call onRemoteOperation callback", () => {
    const onRemoteOp = vi.fn()
    const { result } = renderHook(() =>
      useCollaboration({
        wsUrl: "ws://localhost:8004/ws/test",
        userId: "u1",
        collaborationStore: null,
        onRemoteOperation: onRemoteOp,
      })
    )

    act(() => {
      result.current.connect()
      wsHelper.simulateOpen()
      wsHelper.simulateMessage(JSON.stringify({
        session_id: "test",
        user_id: "u2",
        revision: 0,
        type: "insert",
        position: 0,
        length: 0,
        content: "remote",
        timestamp: "2026-04-18T00:00:00+00:00",
      }))
    })

    expect(onRemoteOp).toHaveBeenCalledOnce()
    expect(onRemoteOp.mock.calls[0][0].user_id).toBe("u2")
  })
})
```

- [ ] **Step 2: Run test to verify it fails**

Run: `pnpm --filter @world-office/collaboration-react test`
Expected: FAIL — cannot find module `../src/hooks/useCollaboration`

- [ ] **Step 3: Implement useCollaboration hook**

Create `packages/collaboration-react/src/hooks/useCollaboration.ts`:

```typescript
import { useCallback, useEffect, useRef, useState } from "react"
import { WebSocketManager, type ConnectionState, type EditOperation } from "@world-office/collaboration-client"
import type { CollaborationStore } from "@world-office/editor-stores"

export interface UseCollaborationOptions {
  wsUrl: string
  userId: string
  collaborationStore: CollaborationStore | null
  onRemoteOperation?: (op: EditOperation) => void
}

export interface UseCollaborationResult {
  connectionState: ConnectionState
  connect: () => void
  disconnect: () => void
  sendInsert: (position: number, text: string) => void
  sendDelete: (position: number, length: number) => void
}

export function useCollaboration(options: UseCollaborationOptions): UseCollaborationResult {
  const { wsUrl, userId, collaborationStore, onRemoteOperation } = options
  const managerRef = useRef<WebSocketManager | null>(null)
  const [connectionState, setConnectionState] = useState<ConnectionState>("disconnected")

  const getOrCreateManager = useCallback((): WebSocketManager => {
    if (!managerRef.current) {
      const manager = new WebSocketManager({
        url: wsUrl,
        userId,
        autoReconnect: true,
      })

      manager.on("stateChange", (state: ConnectionState) => {
        setConnectionState(state)
        collaborationStore?.setConnectionStatus(state)
      })

      manager.on("open", () => {
        collaborationStore?.setConnectionStatus("connected")
      })

      manager.on("close", () => {
        collaborationStore?.setConnectionStatus("disconnected")
      })

      manager.on("operation", (op: EditOperation) => {
        onRemoteOperation?.(op)
      })

      managerRef.current = manager
    }
    return managerRef.current
  }, [wsUrl, userId, collaborationStore, onRemoteOperation])

  const connect = useCallback(() => {
    const manager = getOrCreateManager()
    manager.connect()
  }, [getOrCreateManager])

  const disconnect = useCallback(() => {
    managerRef.current?.disconnect()
    managerRef.current = null
  }, [])

  const sendInsert = useCallback((position: number, text: string) => {
    managerRef.current?.sendInsert(position, text)
  }, [])

  const sendDelete = useCallback((position: number, length: number) => {
    managerRef.current?.sendDelete(position, length)
  }, [])

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      managerRef.current?.disconnect()
      managerRef.current = null
    }
  }, [])

  return {
    connectionState,
    connect,
    disconnect,
    sendInsert,
    sendDelete,
  }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `pnpm --filter @world-office/collaboration-react test`
Expected: All tests PASS

- [ ] **Step 5: Update index.ts**

Replace `packages/collaboration-react/src/index.ts`:

```typescript
export { useCollaboration } from "./hooks/useCollaboration"
export type { UseCollaborationOptions, UseCollaborationResult } from "./hooks/useCollaboration"
```

- [ ] **Step 6: Commit**

```bash
git add packages/collaboration-react/src/hooks/useCollaboration.ts packages/collaboration-react/__tests__/hooks.test.tsx packages/collaboration-react/src/index.ts
git commit -m "feat(collaboration-react): add useCollaboration hook with connect/send/receive/disconnect"
```

---

## Task 10: `useConnectionStatus` Hook

**Files:**
- Create: `packages/collaboration-react/src/hooks/useConnectionStatus.ts`

- [ ] **Step 1: Implement the derived hook**

Create `packages/collaboration-react/src/hooks/useConnectionStatus.ts`:

```typescript
import { useMemo } from "react"
import type { ConnectionState } from "@world-office/collaboration-client"

export interface ConnectionStatusInfo {
  state: ConnectionState
  isConnected: boolean
  isConnecting: boolean
  isReconnecting: boolean
  label: string
  color: string
}

/**
 * Derives human-readable connection status from raw ConnectionState.
 * Pure computation — no side effects.
 */
export function useConnectionStatus(state: ConnectionState): ConnectionStatusInfo {
  return useMemo(() => {
    switch (state) {
      case "connected":
        return { state, isConnected: true, isConnecting: false, isReconnecting: false, label: "Connected", color: "#2ECC71" }
      case "connecting":
        return { state, isConnected: false, isConnecting: true, isReconnecting: false, label: "Connecting...", color: "#F39C12" }
      case "reconnecting":
        return { state, isConnected: false, isConnecting: false, isReconnecting: true, label: "Reconnecting...", color: "#E67E22" }
      case "disconnected":
        return { state, isConnected: false, isConnecting: false, isReconnecting: false, label: "Disconnected", color: "#E74C3C" }
    }
  }, [state])
}
```

- [ ] **Step 2: Update index.ts**

Add to `packages/collaboration-react/src/index.ts`:

```typescript
export { useConnectionStatus } from "./hooks/useConnectionStatus"
export type { ConnectionStatusInfo } from "./hooks/useConnectionStatus"
```

- [ ] **Step 3: Commit**

```bash
git add packages/collaboration-react/src/hooks/useConnectionStatus.ts packages/collaboration-react/src/index.ts
git commit -m "feat(collaboration-react): add useConnectionStatus derived hook"
```

---

## Task 11: `useCollaboratorPresence` Hook

**Files:**
- Create: `packages/collaboration-react/src/hooks/useCollaboratorPresence.ts`

- [ ] **Step 1: Implement the presence hook**

Create `packages/collaboration-react/src/hooks/useCollaboratorPresence.ts`:

```typescript
import { useMemo } from "react"
import type { CollabUser } from "@world-office/editor-stores"

export interface PresenceInfo {
  users: CollabUser[]
  userCount: number
  currentUser: CollabUser | null
  otherUsers: CollabUser[]
  isConnected: boolean
}

/**
 * Derives presence information from CollaborationStore users.
 * Pure computation — no side effects.
 */
export function useCollaboratorPresence(
  users: CollabUser[],
  isConnected: boolean,
): PresenceInfo {
  return useMemo(() => {
    const currentUser = users.find((u) => u.isCurrentUser) ?? null
    const otherUsers = users.filter((u) => !u.isCurrentUser)

    return {
      users,
      userCount: users.length,
      currentUser,
      otherUsers,
      isConnected,
    }
  }, [users, isConnected])
}
```

- [ ] **Step 2: Update index.ts**

Add to `packages/collaboration-react/src/index.ts`:

```typescript
export { useCollaboratorPresence } from "./hooks/useCollaboratorPresence"
export type { PresenceInfo } from "./hooks/useCollaboratorPresence"
```

- [ ] **Step 3: Commit**

```bash
git add packages/collaboration-react/src/hooks/useCollaboratorPresence.ts packages/collaboration-react/src/index.ts
git commit -m "feat(collaboration-react): add useCollaboratorPresence derived hook"
```

---

## Task 12: `CollaborationStatus` Component

**Files:**
- Create: `packages/collaboration-react/src/components/CollaborationStatus.tsx`
- Create: `packages/collaboration-react/__tests__/CollaborationStatus.test.tsx`

- [ ] **Step 1: Write the failing test**

Create `packages/collaboration-react/__tests__/CollaborationStatus.test.tsx`:

```tsx
import { describe, expect, it } from "vitest"
import { render, screen } from "@testing-library/react"
import { CollaborationStatus } from "../src/components/CollaborationStatus"

describe("CollaborationStatus", () => {
  it("should show 'Connected' with green dot when connected", () => {
    render(<CollaborationStatus state="connected" userCount={3} />)
    expect(screen.getByText("Connected")).toBeDefined()
    const dot = screen.getByTestId("collab-status-dot")
    expect(dot.getAttribute("data-color")).toBe("#2ECC71")
  })

  it("should show 'Connecting...' with yellow dot", () => {
    render(<CollaborationStatus state="connecting" userCount={0} />)
    expect(screen.getByText("Connecting...")).toBeDefined()
  })

  it("should show 'Reconnecting...' with orange dot", () => {
    render(<CollaborationStatus state="reconnecting" userCount={0} />)
    expect(screen.getByText("Reconnecting...")).toBeDefined()
  })

  it("should show 'Disconnected' with red dot", () => {
    render(<CollaborationStatus state="disconnected" userCount={0} />)
    expect(screen.getByText("Disconnected")).toBeDefined()
  })

  it("should show user count when connected", () => {
    render(<CollaborationStatus state="connected" userCount={3} />)
    expect(screen.getByText("3")).toBeDefined()
  })

  it("should not show user count when disconnected", () => {
    render(<CollaborationStatus state="disconnected" userCount={0} />)
    expect(screen.queryByText("0")).toBeNull()
  })

  it("should have role='status' for accessibility", () => {
    render(<CollaborationStatus state="connected" userCount={1} />)
    expect(screen.getByRole("status")).toBeDefined()
  })
})
```

- [ ] **Step 2: Run test to verify it fails**

Run: `pnpm --filter @world-office/collaboration-react test`
Expected: FAIL — cannot find module `../src/components/CollaborationStatus`

- [ ] **Step 3: Implement CollaborationStatus component**

Create `packages/collaboration-react/src/components/CollaborationStatus.tsx`:

```tsx
import type { ConnectionState } from "@world-office/collaboration-client"
import { useConnectionStatus } from "../hooks/useConnectionStatus"

export interface CollaborationStatusProps {
  state: ConnectionState
  userCount: number
  className?: string
}

const STATUS_CONFIG: Record<ConnectionState, { label: string; color: string }> = {
  connected: { label: "Connected", color: "#2ECC71" },
  connecting: { label: "Connecting...", color: "#F39C12" },
  reconnecting: { label: "Reconnecting...", color: "#E67E22" },
  disconnected: { label: "Disconnected", color: "#E74C3C" },
}

export function CollaborationStatus({ state, userCount, className }: CollaborationStatusProps) {
  const status = useConnectionStatus(state)

  return (
    <div
      className={`wo-collaboration-status ${className ?? ""}`}
      role="status"
      aria-label={`Collaboration: ${status.label}`}
      style={{ display: "inline-flex", alignItems: "center", gap: 6, fontSize: 12, lineHeight: 1 }}
    >
      <span
        data-testid="collab-status-dot"
        data-color={status.color}
        style={{
          width: 8,
          height: 8,
          borderRadius: "50%",
          backgroundColor: status.color,
          flexShrink: 0,
        }}
      />
      <span>{status.label}</span>
      {status.isConnected && userCount > 0 && (
        <span
          style={{
            backgroundColor: "#3498DB",
            color: "#fff",
            borderRadius: "50%",
            width: 16,
            height: 16,
            display: "inline-flex",
            alignItems: "center",
            justifyContent: "center",
            fontSize: 10,
            fontWeight: 600,
          }}
        >
          {userCount}
        </span>
      )}
    </div>
  )
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `pnpm --filter @world-office/collaboration-react test`
Expected: All tests PASS

- [ ] **Step 5: Update index.ts**

Add to `packages/collaboration-react/src/index.ts`:

```typescript
export { CollaborationStatus } from "./components/CollaborationStatus"
export type { CollaborationStatusProps } from "./components/CollaborationStatus"
```

- [ ] **Step 6: Commit**

```bash
git add packages/collaboration-react/src/components/CollaborationStatus.tsx packages/collaboration-react/__tests__/CollaborationStatus.test.tsx packages/collaboration-react/src/index.ts
git commit -m "feat(collaboration-react): add CollaborationStatus toolbar indicator component"
```

---

## Task 13: `CollaboratorList` Component

**Files:**
- Create: `packages/collaboration-react/src/components/CollaboratorList.tsx`
- Create: `packages/collaboration-react/__tests__/CollaboratorList.test.tsx`

- [ ] **Step 1: Write the failing test**

Create `packages/collaboration-react/__tests__/CollaboratorList.test.tsx`:

```tsx
import { describe, expect, it } from "vitest"
import { render, screen } from "@testing-library/react"
import { CollaboratorList } from "../src/components/CollaboratorList"
import type { CollabUser } from "@world-office/editor-stores"

const mockUsers: CollabUser[] = [
  { id: "u1", name: "Alice", color: "#E74C3C", isCurrentUser: true },
  { id: "u2", name: "Bob", color: "#3498DB", isCurrentUser: false },
  { id: "u3", name: "Carol", color: "#2ECC71", isCurrentUser: false },
]

describe("CollaboratorList", () => {
  it("should render all users", () => {
    render(<CollaboratorList users={mockUsers} />)
    expect(screen.getByText("A")).toBeDefined() // Alice initial
    expect(screen.getByText("B")).toBeDefined() // Bob initial
    expect(screen.getByText("C")).toBeDefined() // Carol initial
  })

  it("should show tooltip with full name on hover", () => {
    render(<CollaboratorList users={mockUsers} />)
    const alice = screen.getByTitle("Alice (you)")
    expect(alice).toBeDefined()
  })

  it("should mark current user avatar with a border", () => {
    render(<CollaboratorList users={mockUsers} />)
    const currentUserAvatar = screen.getByTitle("Alice (you)")
    expect(currentUserAvatar.style.border).toContain("2px solid")
  })

  it("should render empty state when no users", () => {
    render(<CollaboratorList users={[]} />)
    expect(screen.queryByTitle("Alice")).toBeNull()
  })

  it("should limit displayed avatars to maxDisplay", () => {
    render(<CollaboratorList users={mockUsers} maxDisplay={2} />)
    expect(screen.getByText("+1")).toBeDefined()
  })
})
```

- [ ] **Step 2: Run test to verify it fails**

Run: `pnpm --filter @world-office/collaboration-react test`
Expected: FAIL — cannot find module `../src/components/CollaboratorList`

- [ ] **Step 3: Implement CollaboratorList component**

Create `packages/collaboration-react/src/components/CollaboratorList.tsx`:

```tsx
import type { CollabUser } from "@world-office/editor-stores"

export interface CollaboratorListProps {
  users: CollabUser[]
  maxDisplay?: number
  className?: string
}

function getInitial(name: string): string {
  return name.charAt(0).toUpperCase()
}

export function CollaboratorList({ users, maxDisplay = 5, className }: CollaboratorListProps) {
  if (users.length === 0) return null

  const displayed = users.slice(0, maxDisplay)
  const overflow = users.length - maxDisplay

  return (
    <div
      className={`wo-collaborator-list ${className ?? ""}`}
      style={{ display: "inline-flex", alignItems: "center", gap: 4 }}
    >
      {displayed.map((user) => (
        <span
          key={user.id}
          title={`${user.name}${user.isCurrentUser ? " (you)" : ""}`}
          style={{
            width: 24,
            height: 24,
            borderRadius: "50%",
            backgroundColor: user.color,
            color: "#fff",
            display: "inline-flex",
            alignItems: "center",
            justifyContent: "center",
            fontSize: 11,
            fontWeight: 600,
            border: user.isCurrentUser ? "2px solid #fff" : "2px solid transparent",
            boxShadow: "0 0 0 1px rgba(0,0,0,0.1)",
            cursor: "default",
            userSelect: "none",
          }}
          data-testid={`collaborator-${user.id}`}
        >
          {getInitial(user.name)}
        </span>
      ))}
      {overflow > 0 && (
        <span
          style={{
            fontSize: 11,
            color: "#666",
            marginLeft: 2,
          }}
        >
          +{overflow}
        </span>
      )}
    </div>
  )
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `pnpm --filter @world-office/collaboration-react test`
Expected: All tests PASS

- [ ] **Step 5: Update index.ts**

Add to `packages/collaboration-react/src/index.ts`:

```typescript
export { CollaboratorList } from "./components/CollaboratorList"
export type { CollaboratorListProps } from "./components/CollaboratorList"
```

- [ ] **Step 6: Commit**

```bash
git add packages/collaboration-react/src/components/CollaboratorList.tsx packages/collaboration-react/__tests__/CollaboratorList.test.tsx packages/collaboration-react/src/index.ts
git commit -m "feat(collaboration-react): add CollaboratorList avatar component"
```

---

## Task 14: `CollaboratorCursors` Component

**Files:**
- Create: `packages/collaboration-react/src/components/CollaboratorCursors.tsx`
- Create: `packages/collaboration-react/__tests__/CollaboratorCursors.test.tsx`

- [ ] **Step 1: Write the failing test**

Create `packages/collaboration-react/__tests__/CollaboratorCursors.test.tsx`:

```tsx
import { describe, expect, it } from "vitest"
import { render, screen } from "@testing-library/react"
import { CollaboratorCursors } from "../src/components/CollaboratorCursors"

describe("CollaboratorCursors", () => {
  it("should render a cursor for each remote user", () => {
    const cursors = new Map([
      ["user-2", { page: 1, x: 100, y: 200 }],
      ["user-3", { page: 1, x: 300, y: 150 }],
    ])
    const userColors = new Map([
      ["user-2", "#3498DB"],
      ["user-3", "#2ECC71"],
    ])
    const userNames = new Map([
      ["user-2", "Bob"],
      ["user-3", "Carol"],
    ])

    render(
      <CollaboratorCursors
        cursors={cursors}
        userColors={userColors}
        userNames={userNames}
      />
    )

    expect(screen.getByText("Bob")).toBeDefined()
    expect(screen.getByText("Carol")).toBeDefined()
  })

  it("should position cursors at correct coordinates", () => {
    const cursors = new Map([["user-2", { page: 1, x: 150, y: 250 }]])
    const userColors = new Map([["user-2", "#3498DB"]])
    const userNames = new Map([["user-2", "Bob"]])

    render(
      <CollaboratorCursors
        cursors={cursors}
        userColors={userColors}
        userNames={userNames}
      />
    )

    const cursor = screen.getByTestId("cursor-user-2")
    expect(cursor.style.left).toBe("150px")
    expect(cursor.style.top).toBe("250px")
  })

  it("should render nothing when no cursors", () => {
    const { container } = render(
      <CollaboratorCursors
        cursors={new Map()}
        userColors={new Map()}
        userNames={new Map()}
      />
    )
    expect(container.firstChild).toBeNull()
  })
})
```

- [ ] **Step 2: Run test to verify it fails**

Run: `pnpm --filter @world-office/collaboration-react test`
Expected: FAIL

- [ ] **Step 3: Implement CollaboratorCursors component**

Create `packages/collaboration-react/src/components/CollaboratorCursors.tsx`:

```tsx
export interface CollaboratorCursorsProps {
  cursors: Map<string, { page: number; x: number; y: number }>
  userColors: Map<string, string>
  userNames: Map<string, string>
  className?: string
}

export function CollaboratorCursors({
  cursors,
  userColors,
  userNames,
  className,
}: CollaboratorCursorsProps) {
  if (cursors.size === 0) return null

  return (
    <div className={`wo-collaborator-cursors ${className ?? ""}`} style={{ position: "absolute", inset: 0, pointerEvents: "none", overflow: "hidden" }}>
      {Array.from(cursors.entries()).map(([userId, cursor]) => {
        const color = userColors.get(userId) ?? "#999"
        const name = userNames.get(userId) ?? "Unknown"

        return (
          <div
            key={userId}
            data-testid={`cursor-${userId}`}
            style={{
              position: "absolute",
              left: cursor.x,
              top: cursor.y,
              zIndex: 10,
              transition: "left 0.15s ease, top 0.15s ease",
            }}
          >
            {/* Cursor caret */}
            <svg width="16" height="20" viewBox="0 0 16 20" fill="none" style={{ display: "block" }}>
              <path d="M0 0L16 12L8 12L4 20L0 0Z" fill={color} />
            </svg>
            {/* Name label */}
            <span
              style={{
                backgroundColor: color,
                color: "#fff",
                fontSize: 10,
                padding: "1px 4px",
                borderRadius: 2,
                marginLeft: 12,
                marginTop: -4,
                whiteSpace: "nowrap",
                position: "absolute",
                top: 12,
              }}
            >
              {name}
            </span>
          </div>
        )
      })}
    </div>
  )
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `pnpm --filter @world-office/collaboration-react test`
Expected: All tests PASS

- [ ] **Step 5: Update index.ts**

Add to `packages/collaboration-react/src/index.ts`:

```typescript
export { CollaboratorCursors } from "./components/CollaboratorCursors"
export type { CollaboratorCursorsProps } from "./components/CollaboratorCursors"
```

- [ ] **Step 6: Commit**

```bash
git add packages/collaboration-react/src/components/CollaboratorCursors.tsx packages/collaboration-react/__tests__/CollaboratorCursors.test.tsx packages/collaboration-react/src/index.ts
git commit -m "feat(collaboration-react): add CollaboratorCursors overlay component"
```

---

## Task 15: Wire CollaborationStatus into Document Editor Toolbar

**Files:**
- Modify: `apps/web/apps/documenteditor-react/package.json`
- Modify: `apps/web/apps/documenteditor-react/src/components/Toolbar/Toolbar.tsx`

- [ ] **Step 1: Add dependency to documenteditor-react**

In `apps/web/apps/documenteditor-react/package.json`, add to `dependencies`:

```json
"@world-office/collaboration-react": "workspace:*"
```

- [ ] **Step 2: Install the new dependency**

Run: `pnpm install`
Expected: No errors

- [ ] **Step 3: Add CollaborationStatus to Toolbar**

Modify `apps/web/apps/documenteditor-react/src/components/Toolbar/Toolbar.tsx` to add the status indicator in the extra-right area:

```tsx
import { observer } from "mobx-react-lite"
import { documentStore } from "../../stores/DocumentStore"
import { FileTab } from "./FileTab"
import { FormsTab } from "./FormsTab"
import { HeaderFooterTab } from "./HeaderFooterTab"
import { HomeTab } from "./HomeTab"
import { InsertTab } from "./InsertTab"
import { LayoutTab } from "./LayoutTab"
import { ReferencesTab } from "./ReferencesTab"
import { ViewTab } from "./ViewTab"
import { CollaborationStatus } from "@world-office/collaboration-react"

const ObservedToolbar = observer(function ObservedToolbar() {
  const isEditMode = documentStore.isEditMode

  return (
    <div className="de-toolbar">
      <div className="de-toolbar-tabs">
        <div className="de-toolbar-extra-left" />
        <FileTab />
        <HomeTab />
        {isEditMode && <InsertTab />}
        {isEditMode && <LayoutTab />}
        <ReferencesTab />
        <ViewTab />
        {isEditMode && <FormsTab />}
        {isEditMode && <HeaderFooterTab />}
        <div className="de-toolbar-extra-right">
          <CollaborationStatus state="disconnected" userCount={0} />
        </div>
      </div>
      <section className="de-toolbar-controls" role="tabpanel">
        <section className="de-toolbar-static" />
        <section className="de-toolbar-panels" />
      </section>
    </div>
  )
})

export { ObservedToolbar as Toolbar }
```

- [ ] **Step 4: Verify typecheck passes**

Run: `pnpm --filter @world-office/documenteditor typecheck`
Expected: No errors

- [ ] **Step 5: Verify build passes**

Run: `pnpm --filter @world-office/documenteditor build`
Expected: Build succeeds

- [ ] **Step 6: Commit**

```bash
git add apps/web/apps/documenteditor-react/
git commit -m "feat(documenteditor): add CollaborationStatus indicator to toolbar"
```

---

## Task 16: Build Verification for Both Packages

**Files:** No new files — verification only.

- [ ] **Step 1: Typecheck collaboration-client**

Run: `pnpm --filter @world-office/collaboration-client typecheck`
Expected: No errors

- [ ] **Step 2: Typecheck collaboration-react**

Run: `pnpm --filter @world-office/collaboration-react typecheck`
Expected: No errors

- [ ] **Step 3: Build collaboration-client**

Run: `pnpm --filter @world-office/collaboration-client build`
Expected: dist/ created with index.js and index.d.ts

- [ ] **Step 4: Build collaboration-react**

Run: `pnpm --filter @world-office/collaboration-react build`
Expected: dist/ created

- [ ] **Step 5: Verify no regressions in editor-stores**

Run: `pnpm --filter @world-office/editor-stores typecheck`
Expected: No errors

- [ ] **Step 6: Verify no regressions in documenteditor**

Run: `pnpm --filter @world-office/documenteditor build`
Expected: Build succeeds

---

## Final Verification Wave

### F1: Full Test Suite

- [ ] **F1.1: Run all collaboration-client tests**

Run: `pnpm --filter @world-office/collaboration-client test`
Expected: All tests PASS (protocol, reconnection, auth, client, integration)

- [ ] **F1.2: Run all collaboration-react tests**

Run: `pnpm --filter @world-office/collaboration-react test`
Expected: All tests PASS (hooks, CollaborationStatus, CollaboratorList, CollaboratorCursors)

- [ ] **F1.3: Verify workspace install is clean**

Run: `pnpm install`
Expected: No warnings or errors

### F2: TypeScript Strict Mode

- [ ] **F2.1: collaboration-client typecheck**

Run: `pnpm --filter @world-office/collaboration-client typecheck`
Expected: 0 errors

- [ ] **F2.2: collaboration-react typecheck**

Run: `pnpm --filter @world-office/collaboration-react typecheck`
Expected: 0 errors

- [ ] **F2.3: editor-stores typecheck (modified file)**

Run: `pnpm --filter @world-office/editor-stores typecheck`
Expected: 0 errors

- [ ] **F2.4: documenteditor typecheck (modified file)**

Run: `pnpm --filter @world-office/documenteditor typecheck`
Expected: 0 errors

### F3: Build

- [ ] **F3.1: Build all new packages**

Run: `pnpm --filter @world-office/collaboration-client --filter @world-office/collaboration-react build`
Expected: Both build successfully

- [ ] **F3.2: Build documenteditor (integration check)**

Run: `pnpm --filter @world-office/documenteditor build`
Expected: Build succeeds

### F4: No Regressions

- [ ] **F4.1: Ensure no server files were modified**

Run: `git diff --name-only HEAD~N` (review all commits in this plan)
Expected: No files under `services/` or `core/`

- [ ] **F4.2: Ensure wo-pdf was not touched**

Run: `git diff --name-only HEAD~N | grep wo-pdf`
Expected: No output

---

## Definition of Done

- [ ] Two new packages exist: `@world-office/collaboration-client` and `@world-office/collaboration-react`
- [ ] `collaboration-client` has: protocol types, WebSocketManager, AuthClient, BackoffStrategy, integration test
- [ ] `collaboration-react` has: useCollaboration, useConnectionStatus, useCollaboratorPresence hooks, CollaborationStatus, CollaboratorList, CollaboratorCursors components
- [ ] All tests pass in both packages (15+ test files)
- [ ] TypeScript strict mode passes for all modified/new packages
- [ ] Builds succeed for all packages including documenteditor
- [ ] CollaborationStatus indicator is wired into the document editor toolbar
- [ ] CollaborationStore has new fields for connection status
- [ ] Zero modifications to server-side code (coauthoring-service, session-service)
- [ ] Zero modifications to wo-pdf
- [ ] Native WebSocket used (no socket.io)
