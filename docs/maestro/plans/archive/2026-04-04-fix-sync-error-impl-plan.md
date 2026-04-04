---
title: "Fix Sync Error Implementation Plan"
design_ref: "docs/maestro/plans/2026-04-04-fix-sync-error-design.md"
created: "2026-04-04T12:00:00Z"
status: "draft"
total_phases: 3
estimated_files: 2
task_complexity: "medium"
---

# Fix Sync Error Implementation Plan

## Plan Overview

- **Total phases**: 3
- **Agents involved**: `coder`, `tester`, `technical_writer`
- **Estimated effort**: Moderate. Implementation requires careful handling of async tasks and backoff logic.

## Dependency Graph

```
Phase 1 (Implementation) --> Phase 2 (Testing) --> Phase 3 (Documentation)
```

## Execution Strategy

| Stage | Phases | Execution | Agent Count | Notes |
|-------|--------|-----------|-------------|-------|
| 1     | Phase 1 | Sequential | 1 | MatrixEngine implementation |
| 2     | Phase 2 | Sequential | 1 | Unit tests |
| 3     | Phase 3 | Sequential | 1 | Documentation |

## Phase 1: Implement Retry Loop & Backoff

### Objective
Modify `MatrixEngine::start_sync` to automatically retry synchronization with exponential backoff when an error occurs.

### Agent: coder
### Parallel: No

### Files to Modify

- `src/matrix/mod.rs` — 
    - Add a `sync_handle` field to `MatrixEngineInner` to track the active sync task and prevent duplicates.
    - Implement a `Backoff` struct or utility for exponential delays (2s, 4s, 8s, up to 60s).
    - Update `start_sync` to wrap `sync_service.start()` in a retry loop.
    - Log retry attempts using `tracing` or `println!` (consistent with the project's logging style).

### Implementation Details
- Track the `JoinHandle` of the spawned sync task in `MatrixEngineInner`.
- In `start_sync`, if a handle already exists and is not finished, skip spawning a new one.
- The loop should check if the engine is being dropped or if sync is deliberately stopped (e.g., via `SyncServiceState::Terminated`).

### Validation
- Run `cargo check` to ensure no syntax errors.
- Manual verification of the logic via code review.

### Dependencies
- Blocked by: None
- Blocks: Phase 2

---

## Phase 2: Unit Testing

### Objective
Add unit tests to verify the retry mechanism and backoff logic.

### Agent: tester
### Parallel: No

### Files to Modify

- `src/matrix/tests.rs` — 
    - Add a test case that simulates a `SyncService` failure (if possible via mocking, or by checking the loop logic directly).
    - Verify that the backoff duration increases as expected.
    - Ensure that multiple calls to `start_sync` do not spawn multiple background tasks.

### Validation
- Run `cargo test` and ensure all tests pass.

### Dependencies
- Blocked by: Phase 1
- Blocks: Phase 3

---

## Phase 3: Documentation & Polish

### Objective
Update project documentation to reflect the new automatic sync recovery behavior.

### Agent: technical_writer
### Parallel: No

### Files to Modify

- `SPEC.md` — Update the Synchronization section to mention automatic recovery and exponential backoff.

### Validation
- Verify the documentation is clear and accurate.

### Dependencies
- Blocked by: Phase 2
- Blocks: None

---

## File Inventory

| # | File | Phase | Purpose |
|---|------|-------|---------|
| 1 | `src/matrix/mod.rs` | 1 | Core sync logic and retry implementation |
| 2 | `src/matrix/tests.rs` | 2 | Verification of retry logic |
| 3 | `SPEC.md` | 3 | Documentation of the new behavior |

## Risk Classification

| Phase | Risk | Rationale |
|-------|------|-----------|
| 1 | MEDIUM | Risk of resource leaks if tasks are not correctly managed or if the loop doesn't terminate. |
| 2 | LOW | Purely additive testing phase. |
| 3 | LOW | Documentation only. |

## Execution Profile

```
Execution Profile:
- Total phases: 3
- Parallelizable phases: 0
- Sequential-only phases: 3
- Estimated parallel wall time: N/A
- Estimated sequential wall time: 15-20 minutes

Note: Native subagents currently run without user approval gates.
All tool calls are auto-approved without user confirmation.
```
