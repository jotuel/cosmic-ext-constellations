---
title: "Synchronized Session Restoration Implementation Plan"
design_ref: "docs/maestro/plans/2026-04-05-session-restoration-fix-design.md"
created: "2026-04-05T12:30:00Z"
status: "draft"
total_phases: 4
estimated_files: 3
task_complexity: "complex"
---

# Synchronized Session Restoration Implementation Plan

## Plan Overview

- **Total phases**: 4
- **Agents involved**: coder, debugger, tester
- **Estimated effort**: Complex. Involves refactoring the core startup path and handling sensitive asynchronous state transitions.

## Dependency Graph

```
Phase 1 (Engine Refactor) --> Phase 2 (UI Synchronization)
                                     |
                                     v
Phase 4 (Verification) <--- Phase 3 (Self-Healing)
```

## Execution Strategy

| Stage | Phases | Execution | Agent Count | Notes |
|-------|--------|-----------|-------------|-------|
| 1     | Phase 1 | Sequential | 1 | Foundation |
| 2     | Phase 2 | Sequential | 1 | Integration |
| 3     | Phase 3 | Sequential | 1 | Polish |
| 4     | Phase 4 | Sequential | 1 | Validation |

## Phase 1: Core Engine Refactor & Guards

### Objective
Provide immediate session feedback from `MatrixEngine::new` and prevent duplicate store openings.

### Agent: coder
### Parallel: No

### Files to Modify

- `src/matrix/mod.rs` — Update `new()` return type and add guards to `login()`.

### Implementation Details
- Change `MatrixEngine::new` return type to `Result<(Self, Option<String>)>`.
- Move the `restored_user_id` retrieval logic into the `new()` method before it returns.
- In `login()` and `complete_oidc_login()`, add an atomic check: if `sync_service` is already initialized, return `Err(anyhow!("Session already active"))`.
- Add `tracing::info!` and `tracing::error!` blocks to every branch of the `Keyring` and `restore_session` logic.

### Validation
- `cargo check`
- `cargo test matrix::tests`

### Dependencies
- Blocked by: None
- Blocks: Phase 2

---

## Phase 2: UI Lifecycle Synchronization

### Objective
Ensure the UI accurately reflects the engine's authentication state during startup.

### Agent: coder
### Parallel: No

### Files to Modify

- `src/main.rs` — Update `Message::EngineReady` and initialization handlers.

### Implementation Details
- Update `Message::EngineReady` to carry the new `Result<(MatrixEngine, Option<String>)>` payload.
- In the `update()` handler for `EngineReady`, set `self.user_id = user_id` *immediately* before starting any background tasks.
- Simplify `UserReady` to only handle clearing the `is_initializing` flag and updating the sync status bar.
- Audit all `update()` branches to ensure no message resets `user_id` to `None` if authentication was successful.

### Validation
- `cargo check`

### Dependencies
- Blocked by: Phase 1
- Blocks: Phase 3

---

## Phase 3: Self-Healing & Diagnostic Visibility

### Objective
Add manual recovery and improve error reporting for stuck sessions.

### Agent: coder
### Parallel: No

### Files to Modify

- `src/main.rs` — Add `ResetSession` button and logic.
- `src/matrix/mod.rs` — Implement session deletion.

### Implementation Details
- Add `Message::ResetSession`.
- Implement `MatrixEngine::reset_session()` which deletes the `matrix-session` item from the `oo7` keyring.
- Add a "Reset Saved Session" button to the error view shown when `MatrixEngine::new` fails or restoration hangs.
- Add a 10-second timeout to the `oo7` keyring search using `tokio::time::timeout`.

### Validation
- `cargo check`

### Dependencies
- Blocked by: Phase 2
- Blocks: Phase 4

---

## Phase 4: Race Condition Verification

### Objective
Verify the fix against the reported asynchronous failure modes.

### Agent: tester
### Parallel: No

### Files to Create

- `tests/restoration_race.rs` — Mock Secret Service timing tests.

### Implementation Details
- Implement a test that simulates a slow keyring response (3 seconds).
- Trigger a manual login while the slow restoration is pending.
- Verify that the manual login is blocked by the guard and the restoration eventually completes and transitions the UI correctly.

### Validation
- `cargo test`

### Dependencies
- Blocked by: Phase 3
- Blocks: None

---

## File Inventory

| # | File | Phase | Purpose |
|---|------|-------|---------|
| 1 | `src/matrix/mod.rs` | 1, 3 | Core engine refactor and self-healing logic |
| 2 | `src/main.rs` | 2, 3 | UI synchronization and Reset UI |
| 3 | `tests/restoration_race.rs` | 4 | Integration testing |

## Risk Classification

| Phase | Risk | Rationale |
|-------|------|-----------|
| 1 | MEDIUM | Refactoring core constructor is sensitive but mitigated by type-safety. |
| 2 | MEDIUM | Async state transitions in Iced can be subtle. |
| 3 | LOW | Adds safety valves for the user. |
| 4 | MEDIUM | Mocking Secret Service timing can be flaky in CI. |

## Execution Profile

```
Execution Profile:
- Total phases: 4
- Parallelizable phases: 0
- Sequential-only phases: 4
- Estimated parallel wall time: N/A
- Estimated sequential wall time: ~20-30 turns total

Note: Native subagents currently run without user approval gates.
All tool calls are auto-approved without user confirmation.
```
