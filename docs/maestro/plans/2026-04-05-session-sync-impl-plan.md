---
title: "Synchronized Session Restoration Implementation Plan"
design_ref: "docs/maestro/plans/2026-04-05-session-sync-design.md"
created: "2026-04-05T16:30:00Z"
status: "draft"
total_phases: 3
estimated_files: 2
task_complexity: "medium"
---

# Synchronized Session Restoration Implementation Plan

## Plan Overview

- **Total phases**: 3
- **Agents involved**: coder, tester
- **Estimated effort**: Medium. Focuses on core engine refactoring and UI lifecycle synchronization.

## Dependency Graph

```
Phase 1 (Engine) --> Phase 2 (UI) --> Phase 3 (Verification)
```

## Execution Strategy

| Stage | Phases | Execution | Agent Count | Notes |
|-------|--------|-----------|-------------|-------|
| 1     | Phase 1 | Sequential | 1 | Engine core refactor |
| 2     | Phase 2 | Sequential | 1 | UI state alignment |
| 3     | Phase 3 | Sequential | 1 | Validation |

---

## Phase 1: Engine Initialization & Guards

### Objective
Synchronize session restoration status with engine readiness and add guards against redundant logins.

### Agent: coder
### Parallel: No

### Files to Modify

- `src/matrix/mod.rs` — Refactor `MatrixEngine::new` and add guards to `login`/`login_oidc`.

### Implementation Details
- Update `MatrixEngine::new()` to return `Result<(Self, Option<String>)>`, where the string is the user ID if a session was successfully restored.
- In `MatrixEngine::login` and `MatrixEngine::login_oidc`, check if `sync_service` is already initialized. If so, return an error (e.g., `anyhow!("Session already active")`).
- Ensure the write lock is used to make the session check and login initiation atomic.

### Validation
- Run `cargo check`.

### Dependencies
- Blocked by: None
- Blocks: Phase 2

---

## Phase 2: UI State Synchronization

### Objective
Update the `Claw` application to utilize the synchronized engine readiness state.

### Agent: coder
### Parallel: No

### Files to Modify

- `src/main.rs` — Update `Message::EngineReady` and initialization logic.

### Implementation Details
- Update `Message::EngineReady` to carry `Result<(MatrixEngine, Option<String>)>`.
- In the `EngineReady` handler in `update()`, if a `user_id` is present, set `self.user_id = Some(id)` immediately.
- Simplify `UserReady` to only handle clearing `is_initializing` and updating the final sync status.
- Ensure `self.user_id` is never reset to `None` if the engine reports a valid session.

### Validation
- Run `cargo check`.

### Dependencies
- Blocked by: Phase 1
- Blocks: Phase 3

---

## Phase 3: Verification & Regression Testing

### Objective
Verify that automatic login works on startup and redundant logins are blocked.

### Agent: tester
### Parallel: No

### Files to Modify

- `src/matrix/tests.rs` — Add unit tests for engine guards and initialization.

### Implementation Details
- **Unit Test**: Verify that `MatrixEngine::new` correctly returns the `user_id` when a valid session exists in the keyring.
- **Unit Test**: Verify that calling `login` when a session is already restored returns an error.
- **Manual Verification**: Confirm that launching the app with a valid session bypasses the login screen immediately.

### Validation
- Run `cargo test`.
- Pass manual verification checklist.

### Dependencies
- Blocked by: Phase 2
- Blocks: None

---

## File Inventory

| # | File | Phase | Purpose |
|---|------|-------|---------|
| 1 | `src/matrix/mod.rs` | 1 | Engine logic and session guards |
| 2 | `src/main.rs` | 2 | UI state gating and initialization |
| 3 | `src/matrix/tests.rs` | 3 | Verification logic |

## Risk Classification

| Phase | Risk | Rationale |
|-------|------|-----------|
| 1 | MEDIUM | Refactors core constructor and affects all login paths. |
| 2 | LOW | Standard UI state updates. |
| 3 | LOW | Non-intrusive testing. |

## Execution Profile

```
Execution Profile:
- Total phases: 3
- Parallelizable phases: 0
- Sequential-only phases: 3
- Estimated parallel wall time: N/A
- Estimated sequential wall time: ~15-20 turns total

Note: Native subagents currently run without user approval gates.
All tool calls are auto-approved without user confirmation.
```
