---
title: "Synchronized Session Restoration Implementation Plan"
design_ref: "docs/maestro/plans/2026-04-05-session-restoration-fix-v2-design.md"
created: "2026-04-05T14:30:00Z"
status: "draft"
total_phases: 2
estimated_files: 2
task_complexity: "complex"
---

# Synchronized Session Restoration Implementation Plan

## Plan Overview

- **Total phases**: 2
- **Agents involved**: coder, debugger
- **Estimated effort**: Complex. Involves refactoring core authentication paths and handling database lock sequencing.

## Dependency Graph

```
Phase 1 (Engine Refactor) --> Phase 2 (UI Integration)
```

## Execution Strategy

| Stage | Phases | Execution | Agent Count | Notes |
|-------|--------|-----------|-------------|-------|
| 1     | Phase 1 | Sequential | 1 | Engine & Locking |
| 2     | Phase 2 | Sequential | 1 | UI & Diagnostics |

---

## Phase 1: Engine Synchronicity & Lock Mitigation

### Objective
Ensure `MatrixEngine` provides immediate auth feedback and prevents database locking during re-authentication.

### Agent: coder
### Parallel: No

### Files to Modify

- `src/matrix/mod.rs` — Refactor `new()` and `login()` methods.

### Implementation Details
- Change `MatrixEngine::new()` return type to `Result<(Self, Option<String>)>`.
- Move the `restored_user_id` retrieval logic into the `new()` method before returning.
- Refactor `login()` to reuse the existing `matrix_sdk::Client` handle if the homeserver URL matches.
- If the URL differs, ensure the old client is explicitly dropped (releasing the store lock) before initializing the new one.

### Validation
- `cargo check`
- `cargo test matrix::tests`

### Dependencies
- Blocked by: None
- Blocks: Phase 2

---

## Phase 2: UI State Alignment & Diagnostics

### Objective
Synchronize the application loop with the engine and provide granular logging for session restoration.

### Agent: coder
### Parallel: No

### Files to Modify

- `src/main.rs` — Update `EngineReady` handler and UI gating.
- `src/matrix/mod.rs` — Add exhaustive tracing.

### Implementation Details
- Update `Message::EngineReady` to carry the synchronized `user_id`.
- In `Claw::update`, set `self.user_id` immediately upon receiving `EngineReady`.
- Ensure `is_initializing` is only cleared once the session status is definitive.
- Add `tracing::info!` and `tracing::error!` blocks to every branch of the `Keyring` retrieval logic.

### Validation
- `cargo check`
- Manual verification of startup behavior.

### Dependencies
- Blocked by: Phase 1
- Blocks: None

---

## File Inventory

| # | File | Phase | Purpose |
|---|------|-------|---------|
| 1 | `src/matrix/mod.rs` | 1, 2 | Core engine logic and tracing |
| 2 | `src/main.rs` | 2 | UI state machine and initialization |

## Risk Classification

| Phase | Risk | Rationale |
|-------|------|-----------|
| 1 | MEDIUM | Touches core auth path; database lock handling is critical. |
| 2 | LOW | Standard UI synchronization and observability. |

## Execution Profile

```
Execution Profile:
- Total phases: 2
- Parallelizable phases: 0
- Sequential-only phases: 2
- Estimated parallel wall time: N/A
- Estimated sequential wall time: ~15-20 turns total

Note: Native subagents currently run without user approval gates.
All tool calls are auto-approved without user confirmation.
```
