---
title: "Fix matrix-sdk 0.8.0 Compilation Implementation Plan"
design_ref: "docs/maestro/plans/2026-04-04-matrix-sdk-080-upgrade-design.md"
created: "2026-04-04T12:00:00Z"
status: "draft"
total_phases: 3
estimated_files: 3
task_complexity: "medium"
---

# Fix matrix-sdk 0.8.0 Compilation Implementation Plan

## Plan Overview

- **Total phases**: 3
- **Agents involved**: coder, refactor
- **Estimated effort**: Medium. Focuses on updating authentication paths, sync service configurations, and UI subscription logic to match the 0.8.0 SDK.

## Dependency Graph

```
[Phase 1: Core SDK & Auth]
       |
       v
[Phase 2: Matrix Tests]
       |
       v
[Phase 3: UI Shell Integration]
```

## Execution Strategy

| Stage | Phases | Execution | Agent Count | Notes |
|-------|--------|-----------|-------------|-------|
| 1     | Phase 1 | Sequential | 1 | Foundation: Fix src/matrix/mod.rs |
| 2     | Phase 2 | Sequential | 1 | Validation: Fix src/matrix/tests.rs |
| 3     | Phase 3 | Sequential | 1 | Integration: Fix src/main.rs |

## Phase 1: Core SDK & Auth Refactor

### Objective
Restore compilation of the core Matrix engine by updating authentication module paths and client builder calls to version 0.8.0.

### Agent: coder
### Parallel: No

### Files to Modify

- `src/matrix/mod.rs` — Update `matrix_auth` imports to `authentication::matrix`, refactor `client.matrix_auth()` to `client.authentication().matrix()`, and update session restoration logic.

### Implementation Details
- Update all imports of `matrix_sdk::matrix_auth` to `matrix_sdk::authentication::matrix`.
- Update `MatrixEngine::login` and `MatrixEngine::restore_session` to use the new API structure.
- Ensure the `SqliteStateStore` initialization aligns with 0.8.0 configuration patterns.

### Validation
- `cargo check` (expect failures in `main.rs` and `tests.rs`, but `mod.rs` should resolve).

### Dependencies
- Blocked by: None
- Blocks: Phase 2, Phase 3

---

## Phase 2: Matrix Tests Refactor

### Objective
Update unit tests in `src/matrix/tests.rs` to match the new authentication and session types.

### Agent: refactor
### Parallel: No

### Files to Modify

- `src/matrix/tests.rs` — Update manual session construction and mock client setup to use the 0.8.0 authentication types.

### Implementation Details
- Align `MatrixSession` and `MatrixSessionTokens` usage with the new module paths.
- Update any test-specific client builders to match the engine refactor from Phase 1.

### Validation
- `cargo check` (expect failures in `main.rs` only).

### Dependencies
- Blocked by: Phase 1
- Blocks: Phase 3

---

## Phase 3: UI Shell Integration

### Objective
Update the UI shell in `src/main.rs` to correctly subscribe to the new `RoomListService` and handle `SyncService` state transitions.

### Agent: coder
### Parallel: No

### Files to Modify

- `src/main.rs` — Update the room list subscription logic and state mapping to align with `matrix-sdk-ui` 0.8.0.

### Implementation Details
- Replace `entries_with_dynamic_adapters` with the simplified 0.8.0 `entries()` or equivalent subscription method.
- Update the `VectorDiff` mapping to handle changes in how room data is streamed from the `RoomListService`.
- Verify the `SyncService` state machine handling in the UI bridge.

### Validation
- `cargo check` (must pass for the entire project).
- `cargo test` (all Matrix-related tests should pass).

### Dependencies
- Blocked by: Phase 2
- Blocks: None

---

## File Inventory

| # | File | Phase | Purpose |
|---|------|-------|---------|
| 1 | `src/matrix/mod.rs` | 1 | Core SDK integration and Auth |
| 2 | `src/matrix/tests.rs` | 2 | Unit tests for Matrix engine |
| 3 | `src/main.rs` | 3 | UI shell and Subscription logic |

## Risk Classification

| Phase | Risk | Rationale |
|-------|------|-----------|
| 1     | MEDIUM | Authentication is a critical path; breaking session restoration could lock out users. |
| 2     | LOW | Test-only changes; low impact on runtime behavior. |
| 3     | MEDIUM | UI state transitions in 0.8.0 are more reactive; improper handling could lead to UI hangs. |

## Execution Profile

```
Execution Profile:
- Total phases: 3
- Parallelizable phases: 0 (in 0 batches)
- Sequential-only phases: 3
- Estimated parallel wall time: N/A
- Estimated sequential wall time: ~20 minutes

Note: Native subagents currently run without user approval gates.
All tool calls are auto-approved without user confirmation.
```
