---
title: "Fix Sync Error Implementation Plan"
design_ref: "docs/maestro/plans/2026-04-04-fix-sync-error-design.md"
created: "2026-04-04T12:00:00Z"
status: "draft"
total_phases: 4
estimated_files: 3
task_complexity: "medium"
---

# Fix Sync Error Implementation Plan

## Plan Overview

- **Total phases**: 4
- **Agents involved**: `coder`, `tester`, `technical_writer`
- **Estimated effort**: Moderate. Requires careful updating of enum variants and matching logic across the engine and UI layers.

## Dependency Graph

```
Phase 1 (Core Logic) --> Phase 2 (UI Integration) --> Phase 3 (Testing) --> Phase 4 (Documentation)
```

## Execution Strategy

| Stage | Phases | Execution | Agent Count | Notes |
|-------|--------|-----------|-------------|-------|
| 1     | Phase 1 | Sequential | 1 | Core Engine & Types |
| 2     | Phase 2 | Sequential | 1 | UI Layout & Bindings |
| 3     | Phase 3 | Sequential | 1 | Verification |
| 4     | Phase 4 | Sequential | 1 | Documentation |

## Phase 1: Core Logic & Types

### Objective
Update the `SyncStatus` enum and `MatrixEngine` logic to capture and propagate sync errors.

### Agent: coder
### Parallel: No

### Files to Modify

- `src/matrix/mod.rs` — 
    - Update `SyncStatus::Error` to `SyncStatus::Error(String)`.
    - Modify `MatrixEngine::start_sync` loop to capture errors from `sync_service.start()` or the state stream.
    - Update all internal `match` statements for `SyncStatus`.

### Implementation Details
- Ensure the `Error` variant correctly carries the payload.
- In `start_sync`, ensure the error message is extracted from the `SyncService` when it enters the `Error` state.

### Validation
- `cargo check` to verify type safety and exhaustive matches in the engine.

### Dependencies
- Blocked by: None
- Blocks: Phase 2

---

## Phase 2: UI Integration

### Objective
Update the `Claw` application to handle the new `SyncStatus` variant and display the error in the UI.

### Agent: coder
### Parallel: No

### Files to Modify

- `src/main.rs` — 
    - Update the `subscription` to pass the error string into the `SyncStatusChanged` event.
    - Update the `view` function to render the error message (e.g., as a tooltip or updated status label).
    - Update all `match` statements for `SyncStatus` in the UI layer.

### Implementation Details
- Use `cosmic` widgets to display the error message in the status bar area.
- Ensure the error is cleared or updated appropriately when sync restarts.

### Validation
- `cargo build` and manual check of the UI layout.

### Dependencies
- Blocked by: Phase 1
- Blocks: Phase 3

---

## Phase 3: Testing & Verification

### Objective
Add unit tests to verify that sync errors are correctly captured and propagated.

### Agent: tester
### Parallel: No

### Files to Modify

- `src/matrix/tests.rs` — 
    - Add a test case that mocks a sync error and verifies the `SyncStatus` contains the expected message.
    - Verify that the `MatrixEvent` carries the correct payload.

### Validation
- `cargo test`

### Dependencies
- Blocked by: Phase 2
- Blocks: Phase 4

---

## Phase 4: Documentation Update

### Objective
Update project documentation to reflect the new diagnostic capabilities.

### Agent: technical_writer
### Parallel: No

### Files to Modify

- `SPEC.md` — Update the "Synchronization & Recovery" section to document the error reporting behavior.

### Validation
- Review updated documentation for accuracy.

### Dependencies
- Blocked by: Phase 3
- Blocks: None

---

## File Inventory

| # | File | Phase | Purpose |
|---|------|-------|---------|
| 1 | `src/matrix/mod.rs` | 1 | Core types and engine logic |
| 2 | `src/main.rs` | 2 | UI integration and event handling |
| 3 | `src/matrix/tests.rs` | 3 | Unit testing for error propagation |
| 4 | `SPEC.md` | 4 | Documentation update |

## Risk Classification

| Phase | Risk | Rationale |
|-------|------|-----------|
| 1     | MEDIUM | Updating a public-facing enum variant requires careful exhaustive match updates. |
| 2     | LOW | UI layout changes are additive and localized to the status bar. |
| 3     | LOW | Purely verification-focused. |
| 4     | LOW | Documentation only. |

## Execution Profile

```
Execution Profile:
- Total phases: 4
- Parallelizable phases: 0
- Sequential-only phases: 4
- Estimated parallel wall time: N/A
- Estimated sequential wall time: 20-30 minutes

Note: Native subagents currently run without user approval gates.
All tool calls are auto-approved without user confirmation.
```
