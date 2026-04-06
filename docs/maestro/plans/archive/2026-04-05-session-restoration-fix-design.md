---
title: "Synchronized Session Restoration"
created: "2026-04-05T12:00:00Z"
status: "approved"
authors: ["TechLead", "User"]
type: "design"
design_depth: "deep"
task_complexity: "complex"
---

# Synchronized Session Restoration Design Document

## Problem Statement

The `cosmic-ext-claw` application suffers from a critical session management bug where already-authenticated users are incorrectly presented with the login screen on startup. When these users attempt to log in again, the operation fails with a generic "Failed to login" error, despite the backend console reporting success.

The root cause is a **database locking conflict** and **asynchronous race condition**:
1. `MatrixEngine::new` initializes a `matrix_sdk::Client` and opens the SQLite store.
2. If session restoration is slow or fails silently (e.g., due to keyring timeouts), the UI incorrectly clears its "Initializing" state and displays the login screen.
3. Manual login attempts try to create a *new* client and open the *same* SQLite store, which fails because the first client still holds the lock.
4. The login task incorrectly treats any synchronization error as a total authentication failure, leading to the misleading "Failed to login" status in the UI when the console shows authentication success.

## Requirements

### Functional Requirements

1. **REQ-1: Deterministic Initialization** — The application must remain in the "Initializing..." state until the `MatrixEngine` has definitively reported its session restoration status (Authenticated or Unauthenticated).
2. **REQ-2: Resilient Authentication** — The `MatrixEngine::login` method must re-use the existing `matrix_sdk::Client` instance to prevent SQLite store conflicts.
3. **REQ-3: Smart Client Re-initialization** — The client must only be re-created (after properly dropping the old instance) if the user provides a different homeserver URL in the login UI.
4. **REQ-4: Positive Auth Transition** — Upon successful authentication, the UI must immediately transition to the main room list view, even if the initial background synchronization task returns an error.
5. **REQ-5: Manual Keyring Reset** — The UI must provide a "Reset Saved Session" button if initialization fails or hangs, allowing users to manually clear corrupted keyring data.

### Non-Functional Requirements

1. **REQ-6: Observability** — Add granular `tracing` logs to every step of the keyring retrieval and session restoration process.
2. **REQ-7: Stability** — Ensure that the `SqliteStateStore` is never opened twice by overlapping client instances.

### Constraints

- **Linux Secret Service** — Must maintain compatibility with `oo7` and standard desktop keyrings.

## Approach

### Selected Approach

**Synchronous Session Readiness**

We will refactor the engine initialization to block (locally) until the session status is definitive and update the login path to re-use the existing client.

### Alternatives Considered

#### Clean Slate Refresh

- **Description**: Always drop the old client before starting any login.
- **Pros**: Simple logic.
- **Cons**: Still allows UI flickers during the transition gap.
- **Rejected Because**: Does not address the root cause of the incorrect login screen display on startup.

### Decision Matrix

| Criterion | Weight | Readiness Fix (Recommended) | Clean Slate Refresh |
|-----------|--------|----------------------------|---------------------|
| **Bug Resolution** | 40% | 5: Addresses both root causes definitively. | 3: Fixes conflicts but not the UI "gap". |
| **UX Polish** | 30% | 5: Zero flicker, accurate success reporting. | 2: Potential flickering during startup. |
| **Complexity** | 20% | 4: Localized changes to engine/UI. | 3: Requires full lifecycle teardown. |
| **Stability** | 10% | 5: Eliminates all SQLite lock vectors. | 4: Safe, but more moving parts. |
| **Weighted Total** | | **4.8** | **3.0** |

## Architecture

### Component Diagram

```
[User Action] 
      |
      v
[Claw::update] <------------+
      |                     | (Status/User Messages)
      v                     |
[MatrixEngine] (Shared) ----+
      |
      +--> [Keyring (oo7)] -- (Restoration)
      |
      +--> [SDK Client] ----- (Auth/Sync)
      |
      +--> [SQLite Store] --- (Persistent Data)
```

### Data Flow

1. **Init**: `MatrixEngine::new` loads tokens from keyring and calls `client.restore_session`.
2. **Ready**: Engine returns the confirmed `user_id` to `Claw::update` via `EngineReady`.
3. **UI Switch**: `Claw` sets `self.user_id` immediately, ensuring the *first* render after `is_initializing = false` shows the room list.

### Key Interfaces

```rust
impl MatrixEngine {
    // Returns the engine and the user_id if restored
    pub async fn new(data_dir: PathBuf) -> Result<(Self, Option<String>)>;
    
    // Re-uses existing client if URL matches
    pub async fn login(&self, homeserver: &str, username: &str, password: &str) -> Result<()>;
}
```

## Agent Team

| Phase | Agent(s) | Parallel | Deliverables |
|-------|----------|----------|--------------|
| 1     | Debugger | No       | Root cause trace report |
| 2     | Coder    | No       | Engine and UI implementation |
| 3     | Tester   | No       | Race condition verification |

## Risk Assessment

| Risk | Severity | Likelihood | Mitigation |
|------|----------|------------|------------|
| Initialization Hang | HIGH | LOW | 10s timeout on keyring wait. |
| Store Lock Race | MEDIUM | LOW | Atomic checks within engine locks. |

## Success Criteria

1. Startup with valid tokens bypasses the login screen.
2. Manual login alignment (console success == UI success).
3. Collision immunity (no database lock errors).
