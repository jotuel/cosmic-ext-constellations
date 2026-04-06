---
title: "Fix Password Login Regression Implementation Plan (Revised)"
design_ref: "docs/maestro/plans/2026-04-05-password-login-fix-v2-design.md"
created: "2026-04-05T15:30:00Z"
status: "draft"
total_phases: 3
estimated_files: 1
task_complexity: "medium"
---

# Fix Password Login Regression Implementation Plan (Revised)

## Plan Overview

- **Total phases**: 3
- **Agents involved**: coder, tester
- **Estimated effort**: Medium. Refactoring async command logic in the main application loop.

## Dependency Graph

```
Phase 1 (Refactor Tasks) --> Phase 2 (Consistency Check) --> Phase 3 (Validation)
```

## Execution Strategy

| Stage | Phases | Execution | Agent Count | Notes |
|-------|--------|-----------|-------------|-------|
| 1     | Phase 1 | Sequential | 1 | Non-blocking sync tasks |
| 2     | Phase 2 | Sequential | 1 | OIDC flow alignment |
| 3     | Phase 3 | Sequential | 1 | End-to-end verification |

## Phase 1: Resilient Password Login Task

### Objective
Update the `SubmitLogin` task in `src/main.rs` to ensure it returns the `user_id` even if synchronization fails.

### Agent: coder
### Parallel: No

### Files to Modify

- `src/main.rs` — Refactor `Message::SubmitLogin` update logic.

### Implementation Details
- In the `Task::perform` block for `SubmitLogin`:
    - Perform authentication: `matrix.login(...).await?`.
    - Retrieve user ID: `let user_id = matrix.client().await.user_id()...`.
    - Initiate sync resiliently: `let _ = matrix.start_sync().await;` (Remove the `?`).
    - Return `Ok(user_id)`.

### Validation
- Run `cargo check`.

### Dependencies
- Blocked by: None
- Blocks: Phase 2

---

## Phase 2: OIDC Flow Alignment

### Objective
Apply the same non-blocking synchronization logic to the OIDC callback handler.

### Agent: coder
### Parallel: No

### Files to Modify

- `src/main.rs` — Refactor `Message::OidcCallback` update logic.

### Implementation Details
- In the `Task::perform` block for `OidcCallback`:
    - Complete login: `matrix.complete_oidc_login(url).await?`.
    - Retrieve user ID: `let user_id = matrix.client().await.user_id()...`.
    - Initiate sync resiliently: `let _ = matrix.start_sync().await;`.
    - Return `Ok(user_id)`.

### Validation
- Run `cargo check`.

### Dependencies
- Blocked by: Phase 1
- Blocks: Phase 3

---

## Phase 3: Verification & Regression Testing

### Objective
Verify the fix across different homeserver scenarios.

### Agent: tester
### Parallel: No

### Implementation Details
- Verify password login transitions the UI immediately.
- Verify OIDC login transitions the UI immediately.
- Verify that if sync fails (mock or real incompatible server), the error is shown in the status bar.
- Verify that valid sessions are still stored in the keyring.

### Validation
- Successful manual verification of the login flow.

### Dependencies
- Blocked by: Phase 2
- Blocks: None

---

## File Inventory

| # | File | Phase | Purpose |
|---|------|-------|---------|
| 1 | `src/main.rs` | 1, 2 | Application update loop and command handling |

## Risk Classification

| Phase | Risk | Rationale |
|-------|------|-----------|
| 1 | MEDIUM | Touches core login path; affects UI state transitions. |
| 2 | LOW | Consistent application of Phase 1 pattern. |
| 3 | LOW | Standard verification. |

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
