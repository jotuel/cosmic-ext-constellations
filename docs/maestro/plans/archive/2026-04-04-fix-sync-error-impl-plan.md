---
title: "Fix Sync Error Implementation Plan"
design_ref: "docs/maestro/plans/2026-04-04-fix-sync-error-design.md"
created: "2026-04-04T00:00:00Z"
status: "approved"
total_phases: 3
estimated_files: 4
task_complexity: "medium"
---

# Fix Sync Error Implementation Plan

## Plan Overview

- **Total phases**: 3
- **Agents involved**: coder, tester
- **Estimated effort**: Medium. Adds `thiserror`, extends enum in core loop, updates UI handling, and adds testing.

## Dependency Graph

```
[Phase 1: coder]
       |
       v
[Phase 2: coder]
       |
       v
[Phase 3: tester]
```

## Execution Strategy

| Stage | Phases | Execution | Agent Count | Notes |
|-------|--------|-----------|-------------|-------|
| 1     | Phase 1 | Sequential | 1 | Foundation: Error Types & Probing |
| 2     | Phase 2 | Sequential | 1 | UI State Integration |
| 3     | Phase 3 | Sequential | 1 | Validation & Testing |

## Phase 1: Error Types & Probing Logic

### Objective
Define the `SyncError` enum using `thiserror` and update `MatrixEngine` to probe for MSC4186 before syncing.

### Agent: coder
### Parallel: No

### Files to Modify

- `Cargo.toml` — Add `thiserror` dependency.
- `src/matrix/mod.rs` — Define `SyncError`, add `MissingSlidingSyncSupport` to `SyncStatus`, and implement proactive MSC4186 probing in `start_sync` or engine initialization.

### Implementation Details

- Add `thiserror` to `Cargo.toml`.
- Create `SyncError` enum with `thiserror` attributes for formatted error messages.
- Update `SyncStatus` to include a new variant for missing capability.
- Make `MatrixEngine` query supported versions on start.

### Validation

- `cargo check` and `cargo clippy`

### Dependencies

- Blocked by: None
- Blocks: [2]

---

## Phase 2: UI State Integration

### Objective
Handle the new `SyncStatus::MissingSlidingSyncSupport` variant in the main application loop and display appropriate diagnostic messages.

### Agent: coder
### Parallel: No

### Files to Modify

- `src/main.rs` — Update the `SyncStatusChanged` match arm to handle the new variant and present it correctly in the UI.

### Implementation Details

- Ensure the UI maps the `MissingSlidingSyncSupport` to a clear, actionable message that indicates the homeserver needs to be updated.

### Validation

- `cargo check` and `cargo clippy`

### Dependencies

- Blocked by: [1]
- Blocks: [3]

---

## Phase 3: Validation & Testing

### Objective
Add tests for the new error enum variants and verify parsing correctness.

### Agent: tester
### Parallel: No

### Files to Modify

- `src/matrix/tests.rs` — Add unit tests for the error formatting and ensuring that `SyncStatus` mapping works correctly.

### Implementation Details

- Mock or instantiate the `SyncError` and verify the `Display` output.

### Validation

- `cargo test`

### Dependencies

- Blocked by: [2]
- Blocks: None

---

## File Inventory

| # | File | Phase | Purpose |
|---|------|-------|---------|
| 1 | `Cargo.toml` | 1 | Add `thiserror` |
| 2 | `src/matrix/mod.rs` | 1 | Core logic |
| 3 | `src/main.rs` | 2 | UI Integration |
| 4 | `src/matrix/tests.rs` | 3 | Testing |

## Risk Classification

| Phase | Risk | Rationale |
|-------|------|-----------|
| 1     | LOW  | Clean integration |
| 2     | LOW  | Standard UI state update |
| 3     | LOW  | Standard testing |

## Execution Profile

```
Execution Profile:
- Total phases: 3
- Parallelizable phases: 0 (in 0 batches)
- Sequential-only phases: 3
- Estimated parallel wall time: N/A
- Estimated sequential wall time: ~10 minutes
```
