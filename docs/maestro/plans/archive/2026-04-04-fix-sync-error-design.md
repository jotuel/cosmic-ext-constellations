---
title: "Fix Sync Error"
created: "2026-04-04T00:00:00Z"
status: "approved"
authors: ["TechLead", "User"]
type: "design"
design_depth: "standard"
task_complexity: "medium"
---

# Fix Sync Error Design Document

## Problem Statement

The `cosmic-ext-claw` application currently experiences a "Sync Error" that is not adequately diagnosed for the user. While the project implements a background synchronization loop with exponential backoff, it lacks a robust pre-flight check to verify if the Matrix homeserver supports Sliding Sync (MSC4186), which is required by the `matrix-sdk-ui` v0.7.0 `SyncService` — *[Prioritizing pre-flight capability checks over generic runtime error mapping ensures that users on incompatible servers are immediately informed of the true cause of the error, rather than being stuck in an ambiguous "Sync Error" state]*. The objective is to implement a proactive capability probe during initialization to provide clear, actionable feedback when the server lacks necessary features.

## Requirements

### Functional Requirements

1. **REQ-1**: The application MUST verify if the Matrix homeserver supports Sliding Sync (MSC4186) or Matrix v1.11 before initiating the synchronization loop — *[A proactive probe prevents false starts and immediate failure in the sync loop]* *(considered: relying solely on `SyncStatus::Error` mapping — rejected because it delays user feedback and makes parsing error context more complex)*.
2. **REQ-2**: The `SyncStatus` enum in `src/matrix/mod.rs` MUST include a specific variant for `MissingSlidingSyncSupport` to allow the UI to display a targeted error message.
3. **REQ-3**: The `MatrixEngine` MUST propagate the specific capability error result to the UI via the existing `iced::Subscription`.

### Non-Functional Requirements

1. **REQ-4**: The capability probe MUST be performed in a single asynchronous request using `get_supported_versions` to minimize impact on startup performance.

### Constraints

- **REQ-5**: The implementation MUST integrate with the existing `Backoff` logic in `src/matrix/mod.rs` to handle transient network errors during the pre-flight check.

## Approach

### Selected Approach

**Proactive Pre-flight & Fail-Fast**

The application will issue a single `get_supported_versions` request through the `MatrixEngine` before starting the `SyncService`. If the MSC4186 (Sliding Sync) feature is absent, the `SyncStatus` will transition to `MissingSlidingSyncSupport` — *[This proactive design ensures the user is immediately informed of the true cause of the sync failure, rather than seeing a generic error after a delay]* *(considered: mapping `SyncServiceState::Error` purely at runtime — rejected because it delays feedback to the user and complicates error parsing)*. `Traces To: REQ-1, REQ-2, REQ-4`

We will implement this check as a robust pre-flight asynchronous probe within `MatrixEngine::start_sync`, leveraging the existing `Backoff` for resilience. `Traces To: REQ-3, REQ-5`

### Alternatives Considered

#### On-Demand Diagnostic Probe

- **Description**: Query server capabilities only after a `SyncStatus::Error` is detected.
- **Pros**: Zero additional startup overhead.
- **Cons**: User learns about incompatibility only after a generic failure and delay.
- **Rejected Because**: It provides a poorer user experience for incompatible servers compared to a proactive check.

### Decision Matrix

| Criterion | Weight | Approach 1 (Proactive) | Approach 2 (On-Demand) |
|-----------|--------|------------------------|------------------------|
| **User Experience** | 40% | 5: Immediate, fail-fast clarity | 3: User sees generic error first |
| **Simplicity** | 30% | 4: Centralized in the sync loop | 3: Requires more complex event handling |
| **Performance (Startup)** | 20% | 4: Minimal overhead (1 request) | 5: Zero additional startup overhead |
| **Maintainability** | 10% | 5: Clear state transitions | 4: Slightly more complex mapping |
| **Weighted Total** | | **4.5** | **3.6** |

## Risk Assessment

| Risk | Severity | Likelihood | Mitigation |
|------|----------|------------|------------|
| Startup delay from extra capability probe request. | LOW | HIGH | Asynchronous execution with timeouts and retry logic — *[Timeouts prevent application hangs on slow servers]* `Traces To: REQ-4, REQ-5`. |
| Non-exhaustive match arms in UI when `SyncStatus` changes. | LOW | HIGH | The Rust compiler will naturally flag missing match arms for the new variant. `Traces To: REQ-2, REQ-3` |

## Success Criteria

1. `MatrixEngine` proactively identifies and reports missing Sliding Sync (MSC4186) support before starting the sync service.
2. The application surfaces a specific diagnostic message instead of a generic "Sync Error" when incompatibility is detected.
3. Synchronization remains resilient to transient network errors during the pre-flight check.
