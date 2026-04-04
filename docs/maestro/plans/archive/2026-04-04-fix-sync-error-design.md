---
title: "Fix Sync Error"
created: "2026-04-04T00:00:00Z"
status: "approved"
authors: ["TechLead", "User"]
type: "design"
design_depth: "deep"
task_complexity: "medium"
---

# Fix Sync Error Design Document

## Problem Statement

The `cosmic-ext-claw` application currently experiences a "Sync Error" that is not adequately diagnosed for the user. When the underlying `matrix-sdk-ui` `SyncService` encounters an error—most notably when the Matrix homeserver lacks support for Sliding Sync (MSC4186)—the application surfaces a generic or static error message. This prevents users from distinguishing between transient network issues and permanent server capability limitations — *[Prioritizing diagnostic reporting over fallback mechanisms ensures users understand exactly why their client cannot connect, aligning with the goal of clarity]*. The objective is to implement preemptive capability probing and structured error reporting so the UI can clearly inform the user of the specific cause of the sync failure.

## Requirements

### Functional Requirements

1. **REQ-1**: The application MUST verify if the Matrix homeserver supports Sliding Sync (MSC4186) during the login or initialization phase, before launching the `SyncService` — *[A proactive probe prevents false starts and immediate failure in the sync loop]* *(considered: relying solely on `SyncService` error mapping — rejected because it delays feedback to the user)*.
2. **REQ-2**: The application MUST define structured error types (`SyncError`) using `thiserror` to categorize sync failures — *[Using standard ecosystem tools ensures idiomatic and maintainable error definitions]*.

### Non-Functional Requirements

1. **REQ-3**: The startup performance impact of the capability probe MUST be minimal, ideally confined to a single asynchronous API request.

### Constraints

- **REQ-4**: The error definitions and propagation mechanisms MUST integrate smoothly with the existing `matrix-sdk-ui` crate usage and the UI's `Message::Matrix(event)` enum.

## Approach

### Selected Approach

**Early Probing & Extended `SyncStatus`**

The application will issue a `get_supported_versions` API call before starting the `SyncService`. If the MSC4186 (Sliding Sync) feature is absent, the `MatrixEngine` will yield a new `SyncStatus::MissingSlidingSyncSupport` variant instead of attempting to connect — *[This fail-fast design ensures the user is immediately informed of the true cause of the error]* *(considered: mapping `SyncServiceState::Error` purely at runtime — rejected because it delays user feedback and makes parsing error payloads more complex)*. `Traces To: REQ-1, REQ-3`

We will implement a `SyncError` enum backed by `thiserror` to model these failure states, and expand the existing `SyncStatus` enum to encompass these detailed errors. `Traces To: REQ-2, REQ-4`

### Alternatives Considered

#### Runtime Error Mapping

- **Description**: Rely on the underlying SDK's error output to infer a missing capability.
- **Pros**: Zero additional startup overhead.
- **Cons**: Slower diagnostic feedback to the user and requires more complex UI-side parsing logic.
- **Rejected Because**: It delays user feedback and complicates error parsing compared to proactive enum-based state updates.

### Decision Matrix

| Criterion | Weight | Early Probing | Runtime Mapping |
|-----------|--------|---------------|-----------------|
| **User Clarity** | 40% | 5: Immediate, preemptive diagnostics | 3: User informed only after failure |
| **Ease of Implementation** | 30% | 4: Clean integration with existing enum pattern | 3: Requires more complex parsing logic |
| **Performance (Startup)** | 20% | 3: Adds a single startup capability check | 5: Zero additional startup overhead |
| **Maintainability** | 10% | 5: Idiomatic enum-based state management | 4: Slightly more complex mapping |
| **Weighted Total** | | **4.3** | **3.6** |

## Risk Assessment

| Risk | Severity | Likelihood | Mitigation |
|------|----------|------------|------------|
| The extra startup request for server capabilities could delay application responsiveness. | LOW | HIGH | Execute the `get_supported_versions` call concurrently where possible or implement a minimal timeout, returning a `NetworkError` variant to avoid indefinite hangs — *[Timeouts prevent degraded UX from unreachable servers]* `Traces To: REQ-3`. |
| Existing `match` statements across the codebase that rely on `SyncStatus` may become non-exhaustive once new variants are added. | LOW | HIGH | The Rust compiler will flag these exhaustiveness failures, guiding the developer to update UI handling gracefully. We will rely on compiler checks to ensure complete coverage. `Traces To: REQ-4` |