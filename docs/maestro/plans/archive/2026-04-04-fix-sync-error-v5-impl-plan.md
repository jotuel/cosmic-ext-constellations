---
title: "Fix Sync Error V5 Implementation Plan"
design_ref: "docs/maestro/plans/2026-04-04-fix-sync-error-v5-design.md"
created: "2026-04-04T00:00:00Z"
status: "draft"
total_phases: 2
estimated_files: 3
task_complexity: "complex"
---

# Fix Sync Error V5 Implementation Plan

## Plan Overview

- **Total phases**: 2
- **Agents involved**: coder, debugger
- **Estimated effort**: Medium to High. Involves setting up tracing infrastructure and instrumenting the async sync loop for deep debugging.

## Dependency Graph

```
[Phase 1: Instrumentation (coder)]
       |
       v
[Phase 2: Verification (debugger)]
```

## Execution Strategy

| Stage | Phases | Execution | Agent Count | Notes |
|-------|--------|-----------|-------------|-------|
| 1     | Phase 1 | Sequential | 1 | Setup tracing and log explicitly in the sync loop |
| 2     | Phase 2 | Sequential | 1 | Run the application to capture the diagnostic output |

## Phase 1: Instrumentation

### Objective
Configure a global `tracing` subscriber outputting to `stderr` and instrument the `start_sync` loop with explicit error logs.

### Agent: coder
### Parallel: No

### Files to Modify

- `Cargo.toml` — Ensure `tracing-subscriber` with `env-filter` feature is present.
- `src/main.rs` — Initialize `tracing_subscriber::fmt` early in the application startup.
- `src/matrix/mod.rs` — Add explicit `tracing::error!` and `tracing::info!` logs to `MatrixEngine::start_sync` to capture transition reasons and backoff state.

### Implementation Details
- Add `tracing-subscriber = { version = "0.3", features = ["env-filter"] }` to `Cargo.toml` if it's missing.
- In `src/main.rs` (likely in `main` function before running the iced app), configure: `tracing_subscriber::fmt().with_env_filter("matrix_sdk=debug,matrix_sdk_ui=debug,cosmic_ext_claw=debug").with_writer(std::io::stderr).init();`
- In `src/matrix/mod.rs` inside `start_sync`, log when `sync_service.start()` yields an error, printing the error. Log every retry iteration and the backoff duration.

### Validation
- `cargo check` and `cargo clippy`

### Dependencies
- Blocked by: None
- Blocks: [2]

---

## Phase 2: Verification

### Objective
Run the application to verify that deep SDK logs are being captured and printed to `stderr` when a Sync Error occurs.

### Agent: debugger
### Parallel: No

### Files to Modify
- None

### Implementation Details
- Run the application using `cargo run`.
- Observe the `stderr` output to identify the exact cause of the "Sync Error" that has been failing previously.
- Document the findings in the Task Report to conclude the investigation.

### Validation
- Manual inspection of `stderr` output from `cargo run`.

### Dependencies
- Blocked by: [1]
- Blocks: None

---

## File Inventory

| # | File | Phase | Purpose |
|---|------|-------|---------|
| 1 | `Cargo.toml` | 1 | Dependencies |
| 2 | `src/main.rs` | 1 | Tracing subscriber initialization |
| 3 | `src/matrix/mod.rs` | 1 | Sync loop instrumentation |

## Risk Classification

| Phase | Risk | Rationale |
|-------|------|-----------|
| 1     | LOW  | Standard tracing setup, non-invasive to business logic. |
| 2     | LOW  | Purely observational execution. |

## Execution Profile

```
Execution Profile:
- Total phases: 2
- Parallelizable phases: 0 (in 0 batches)
- Sequential-only phases: 2
- Estimated parallel wall time: N/A
- Estimated sequential wall time: ~10 minutes
```

## Plan-Level Cost Summary

| Phase | Agent | Model | Est. Input | Est. Output | Est. Cost |
|-------|-------|-------|-----------|------------|----------|
| 1 | coder | Pro | 2000 | 500 | $0.04 |
| 2 | debugger | Pro | 3000 | 200 | $0.04 |
| **Total** | | | **5000** | **700** | **$0.08** |