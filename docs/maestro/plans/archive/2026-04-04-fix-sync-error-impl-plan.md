---
title: "Fix Sync Error Implementation Plan"
design_ref: "docs/maestro/plans/2026-04-04-fix-sync-error-design.md"
created: "2026-04-04T00:00:00Z"
status: "draft"
total_phases: 3
estimated_files: 3
task_complexity: "medium"
---

# Fix Sync Error Implementation Plan

## Plan Overview

- **Total phases**: 3
- **Agents involved**: coder, tester
- **Estimated effort**: Medium. Implementation involves updating core Matrix engine logic and UI state mapping.

## Dependency Graph

```
[Phase 1: Core Logic]
       |
       v
[Phase 2: UI Integration]
       |
       v
[Phase 3: Validation]
```

## Execution Strategy

| Stage | Phases | Execution | Agent Count | Notes |
|-------|--------|-----------|-------------|-------|
| 1     | Phase 1 | Sequential | 1 | Foundation: SyncStatus and Probing logic |
| 2     | Phase 2 | Sequential | 1 | Integration: UI error mapping |
| 3     | Phase 3 | Sequential | 1 | Quality: Unit tests |

## Phase 1: Core Logic Update

### Objective
Update the `SyncStatus` enum and implement the proactive capability probe in the `MatrixEngine`.

### Agent: coder
### Parallel: No

### Files to Modify

- `src/matrix/mod.rs` — 
    - Add `MissingSlidingSyncSupport` variant to `SyncStatus`.
    - Implement the `get_supported_versions` probe within `MatrixEngine::start_sync`.
    - Ensure the probe respects the existing `Backoff` retry logic.

### Implementation Details
- Use `matrix_sdk::ruma::api::client::discovery::get_supported_versions::Request` for the probe.
- Identify Sliding Sync support by checking `unstable_features` for `org.matrix.msc4186` or the Matrix version `v1.11`.
- Update the background loop to yield `SyncStatus::MissingSlidingSyncSupport` if the probe fails the capability check.

### Validation
- `cargo check` to ensure type safety.
- `cargo clippy` for code quality.

### Dependencies
- Blocked by: None
- Blocks: Phase 2

---

## Phase 2: UI Integration

### Objective
Update the application UI to handle and display the new `MissingSlidingSyncSupport` error state.

### Agent: coder
### Parallel: No

### Files to Modify

- `src/main.rs` — 
    - Update the `Subscription` and `update` logic to handle the new `SyncStatus` variant.
    - Add a user-facing error message in the `view` function for the missing capability state.

### Implementation Details
- Map `SyncStatus::MissingSlidingSyncSupport` to a clear diagnostic message: "Error: Your homeserver does not support Sliding Sync (MSC4186), which is required by Claw."
- Ensure the error is displayed prominently in the status area when triggered.

### Validation
- `cargo check` to verify match arm exhaustiveness.
- Manual visual check of the UI message (if possible).

### Dependencies
- Blocked by: Phase 1
- Blocks: Phase 3

---

## Phase 3: Validation & Testing

### Objective
Verify the correctness of the new synchronization state transitions and error reporting.

### Agent: tester
### Parallel: No

### Files to Modify

- `src/matrix/tests.rs` — 
    - Add test cases for the `SyncStatus` mapping.
    - Mock or simulate a missing capability response to verify the engine's behavior.

### Implementation Details
- Add a test `test_sync_status_missing_support` to verify that the enum is correctly handled in `MatrixEvent`.
- Ensure `test_sync_status_equality` is updated for the new variant.

### Validation
- `cargo test`

### Dependencies
- Blocked by: Phase 2
- Blocks: None

---

## File Inventory

| # | File | Phase | Purpose |
|---|------|-------|---------|
| 1 | `src/matrix/mod.rs` | 1 | Core engine logic and enum definitions |
| 2 | `src/main.rs` | 2 | UI state mapping and error display |
| 3 | `src/matrix/tests.rs` | 3 | Unit tests for sync states |

## Risk Classification

| Phase | Risk | Rationale |
|-------|------|-----------|
| 1 | LOW | Localized change to the sync loop, leverages existing backoff. |
| 2 | LOW | Standard update to iced message handling and view rendering. |
| 3 | LOW | Standard testing phase. |

## Execution Profile

```
Execution Profile:
- Total phases: 3
- Parallelizable phases: 0
- Sequential-only phases: 3
- Estimated parallel wall time: N/A
- Estimated sequential wall time: ~10 minutes
```
