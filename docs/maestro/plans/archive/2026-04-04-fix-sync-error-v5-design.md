---
title: "Fix Sync Error V5"
created: "2026-04-04T00:00:00Z"
status: "approved"
authors: ["TechLead", "User"]
type: "design"
design_depth: "deep"
task_complexity: "complex"
---

# Fix Sync Error V5 Design Document

## Problem Statement

The `cosmic-ext-claw` application suffers from a recurring "Sync Error" that prevents synchronization with the Matrix homeserver. Despite four previous attempts to fix this—including adding capability checks for Sliding Sync (MSC4186) and mapping errors to the UI—the underlying cause remains hidden from both the user and the developer. The current implementation relies on a generic `SyncService` state transition (`Error`) without surfacing the actual SDK-level failure reason (e.g., HTTP 404, TLS handshake failure, proxy misconfiguration). *[Given the failure of previous superficial fixes, the priority must shift from user-facing UI messages to deep, developer-centric diagnostics.]* The objective of this fifth attempt is to implement verbose, console-based instrumentation to capture and expose the internal `matrix-sdk` logs and state transitions to `stderr`, enabling definitive root-cause analysis during authorized debugging sessions.

## Requirements

### Functional Requirements

1. **REQ-1**: The application MUST configure a global `tracing` subscriber that outputs to `stderr` with filtering set to capture `matrix_sdk=debug` or `error` level events, along with `matrix_sdk_ui` logs — *[This bypasses the UI and ensures deep SDK events are visible during debugging]* *(considered: writing to a log file — rejected because an in-memory or console approach provides faster, authorized feedback without file management overhead)*.
2. **REQ-2**: The `MatrixEngine::start_sync` background loop MUST be instrumented to log every retry attempt, including the current backoff duration and the specific `Result` or state transition that caused the retry — *[Verbose logging of the backoff state is essential to differentiate between transient network blips and permanent capability failures]*.
3. **REQ-3**: The UI MUST preserve the existing "Sync Error" status but append a brief instruction, e.g., "(see console for details)" when an error occurs — *[This bridges the gap between the UI failure state and the deep console diagnostics without cluttering the UI]*.

### Non-Functional Requirements

- **REQ-4**: The instrumentation MUST NOT block or crash the `iced` event loop or the background synchronization task; it must be purely observational.

### Constraints

- **REQ-5**: The logging configuration MUST rely on the existing `tracing` and `tracing-subscriber` ecosystem already prevalent in Rust Matrix applications.

## Approach

### Selected Approach

**Tracing Bridge to Stderr**

We will configure `tracing-subscriber` in `src/main.rs` at application startup to emit logs to `stderr`, filtering specifically for `matrix_sdk` and `matrix_sdk_ui` events (e.g., `info` or `debug` level) — *[This approach provides the highest resolution of data without modifying the matrix-sdk-ui crate directly]* *(considered: wrapping the `SyncService` in a dedicated polling task — rejected because it might miss internal SDK errors that don't trigger a full state change)*. `Traces To: REQ-1, REQ-4, REQ-5`

The `MatrixEngine::start_sync` method will be updated to include explicit `tracing::error!` and `tracing::info!` calls inside the retry loop, capturing the exact transition reason and backoff state. The UI will maintain the generic "Sync Error" status but append a brief console instruction. `Traces To: REQ-2, REQ-3`

### Alternatives Considered

#### Instrumented Sync State Wrapper

- **Description**: A dedicated `tokio` task to explicitly capture transitions and print a full state dump to `stderr`.
- **Pros**: High signal-to-noise ratio, explicit tracking.
- **Cons**: Misses internal SDK "play-by-play" logs that don't trigger state changes.
- **Rejected Because**: It misses the deep internal traces that are critical after four failed attempts.

### Decision Matrix

| Criterion | Weight | Approach 1 (Tracing) | Approach 2 (Wrapper) |
|-----------|--------|----------------------|----------------------|
| **Diagnostic Resolution** | 40% | 5: Full SDK internal visibility | 3: Only high-level state changes |
| **Ease of Implementation** | 30% | 4: Uses standard `tracing` patterns | 3: Custom monitoring task logic |
| **Maintainability** | 20% | 5: Standard, non-invasive logging | 4: Modular but custom |
| **Performance Impact** | 10% | 4: Minimal (filter-based) | 5: Zero until error occurs |
| **Weighted Total** | | **4.6** | **3.4** |

## Architecture

The `tracing-subscriber` will be initialized in `src/main.rs` (or equivalent entry point) configured with an `EnvFilter` pointing to `stderr`. The `MatrixEngine::start_sync` in `src/matrix/mod.rs` will be modified to include explicit `tracing::error!` calls when handling a `SyncServiceState::Error` transition or failing the initial capability probe. — *[A global subscriber ensures that all internal SDK events (like network timeouts or auth errors) are captured regardless of where they originate]* `Traces To: REQ-1`

## Agent Team

| Phase | Agent(s) | Parallel | Deliverables |
|-------|----------|----------|--------------|
| 1     | coder    | No       | Instrumentation in `main.rs` and `mod.rs` |
| 2     | debugger | No       | Review and manual test execution |

## Risk Assessment

| Risk | Severity | Likelihood | Mitigation |
|------|----------|------------|------------|
| Verbose logging to stderr obscures other application output | LOW | MEDIUM | The `tracing` configuration will be specifically filtered for `matrix_sdk` and `matrix_sdk_ui` errors and debug events, preventing unrelated noise — *[Filtering balances deep visibility with usability during debugging]* `Traces To: REQ-1`. |

## Success Criteria

1. The application emits detailed `matrix_sdk` internal logs to `stderr` when a Sync Error occurs.
2. The retry loop in `start_sync` logs the iteration count and backoff duration explicitly.
3. A developer can run the application, intentionally trigger the error, and read the root cause from the console.