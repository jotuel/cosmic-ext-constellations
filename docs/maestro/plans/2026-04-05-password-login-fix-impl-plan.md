---
title: "Fix Password Login Regression Implementation Plan"
design_ref: "docs/maestro/plans/2026-04-05-password-login-fix-design.md"
created: "2026-04-05T14:30:00Z"
status: "draft"
total_phases: 3
estimated_files: 2
task_complexity: "medium"
---

# Fix Password Login Regression Implementation Plan

## Plan Overview

- **Total phases**: 3
- **Agents involved**: coder, tester
- **Estimated effort**: Medium. Targeted fix in core auth logic with validation.

## Dependency Graph

```
Phase 1 (Core Fix) --> Phase 2 (Logging) --> Phase 3 (Verification)
```

## Execution Strategy

| Stage | Phases | Execution | Agent Count | Notes |
|-------|--------|-----------|-------------|-------|
| 1     | Phase 1 | Sequential | 1 | Core logic fix |
| 2     | Phase 2 | Sequential | 1 | Observability |
| 3     | Phase 3 | Sequential | 1 | Validation |

## Phase 1: Login State Restoration

### Objective
Update `MatrixEngine::login` to correctly persist authenticated services to the shared state.

### Agent: coder
### Parallel: No

### Files to Modify

- `src/matrix/mod.rs` — Update `login()` method.

### Implementation Details
- Build `SyncService` and `RoomListService` using the authenticated client.
- Acquire a write lock on `inner` via `self.inner.write().await`.
- Update `inner.client`, `inner.sync_service`, and `inner.room_list_service` fields.
- Call `spawn_session_change_handler(client)` to monitor token refreshes.

### Validation
- Run `cargo check` to ensure compilation.

### Dependencies
- Blocked by: None
- Blocks: Phase 2

---

## Phase 2: Enhanced Authentication Logging

### Objective
Improve diagnostic visibility in the password login flow.

### Agent: coder
### Parallel: No

### Files to Modify

- `src/matrix/mod.rs` — Add tracing logs to `login()`.

### Implementation Details
- Add `tracing::info!` logs for key login milestones (e.g., "Starting password login", "Authentication successful", "Engine state updated").
- Add `tracing::error!` for specific failure points in the login process.

### Validation
- Run `cargo check`.

### Dependencies
- Blocked by: Phase 1
- Blocks: Phase 3

---

## Phase 3: Verification & Regression Testing

### Objective
Confirm the fix and ensure no regressions in existing flows.

### Agent: tester
### Parallel: No

### Files to Modify

- `src/matrix/tests.rs` — Add or update unit tests for login state transitions (if possible with mocks).

### Implementation Details
- Verify password login flow (manual check of UI transition).
- Verify sync activation post-login.
- Verify OIDC login continues to work.
- Verify session persistence after restart.

### Validation
- Run `cargo test`.
- Successful manual login with test account.

### Dependencies
- Blocked by: Phase 2
- Blocks: None

---

## File Inventory

| # | File | Phase | Purpose |
|---|------|-------|---------|
| 1 | `src/matrix/mod.rs` | 1, 2 | Core engine logic and state management |
| 2 | `src/matrix/tests.rs` | 3 | Verification tests |

## Risk Classification

| Phase | Risk | Rationale |
|-------|------|-----------|
| 1 | MEDIUM | Touches core auth path; lock contention potential. |
| 2 | LOW | Purely observability. |
| 3 | LOW | Non-intrusive validation. |

## Execution Profile

```
Execution Profile:
- Total phases: 3
- Parallelizable phases: 0
- Sequential-only phases: 3
- Estimated parallel wall time: N/A
- Estimated sequential wall time: ~10-15 turns total

Note: Native subagents currently run without user approval gates.
All tool calls are auto-approved without user confirmation.
```
