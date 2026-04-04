---
title: "Fix matrix-sdk 0.8.0 Compilation Errors"
created: "2026-04-04T12:00:00Z"
status: "approved"
authors: ["TechLead", "User"]
type: "design"
design_depth: "quick"
task_complexity: "medium"
---

# Fix matrix-sdk 0.8.0 Compilation Errors Design Document

## Problem Statement

The `claw` Matrix client currently fails to compile after updating to `matrix-sdk` version 0.8.0. This update introduced significant breaking changes to authentication module paths, the `SlidingSync` configuration, and the high-level UI component APIs. Beyond compilation, existing logs indicate a protocol mismatch (404 / M_UNRECOGNIZED) when using the previous sliding sync configuration, suggesting the application is attempting to use a deprecated sync endpoint. The objective is to restore compilation by refactoring the `MatrixEngine` to follow 0.8.0 idiomatic patterns and migrating to the high-level `SyncService` and `RoomListService` to ensure reliable, future-proof synchronization.

## Requirements

### Functional Requirements

1. **REQ-1**: Restore compilation of the `src/matrix/` module by updating all `matrix-sdk` imports and API calls to version 0.8.0.
2. **REQ-2**: Implement session restoration using the new `authentication().matrix()` API structure.
3. **REQ-3**: Migrate synchronization logic to the high-level `SyncService` and `RoomListService`.
4. **REQ-4**: Resolve the 404 sync protocol issue by ensuring native sliding sync (MSC4186) is correctly prioritized over the deprecated proxy endpoint (MSC3575).

### Non-Functional Requirements

1. **REQ-5**: Maintain the existing "Engine and Shell" architecture where `src/matrix/` handles the SDK and `src/main.rs` handles the UI.

## Approach

### Selected Approach

**Modern UI-First Refactor**

The refactor will systematically migrate the `MatrixEngine` logic in `src/matrix/mod.rs` to the new `matrix-sdk-ui` 0.8.0 abstractions. 

Key implementation steps include:
- **Authentication**: Update imports from `matrix_auth` to `authentication::matrix` and use the updated client builder methods.
- **Sync Logic**: Replace the manual `SlidingSync` builder loop with the managed `SyncService` and `RoomListService`.
- **Protocol Alignment**: Explicitly configure the sync service to use native sliding sync (MSC4186) where supported, resolving the 404 errors observed with the older proxy-based endpoint (MSC3575).
- **UI Bridge**: Update the `Subscription` and `MatrixEvent` mapping logic to consume the new `RoomList` and `Timeline` streams.

### Alternatives Considered

#### Targeted Path Refactor

- **Description**: A localized update of module paths while keeping manual sync logic.
- **Pros**: Faster initial build recovery.
- **Cons**: Fails to address the underlying protocol deprecation and leaves the project on a divergent path from the SDK's recommended patterns.
- **Rejected Because**: It fails to resolve the protocol issues surfaced in the investigation and creates significant future technical debt.

## Risk Assessment

| Risk | Severity | Likelihood | Mitigation |
|------|----------|------------|------------|
| SDK API Instability | MEDIUM | MEDIUM | Follow official migration guides and use strict type checks for all state transitions. |
| Protocol Mismatch (MSC3575 vs MSC4186) | HIGH | MEDIUM | Detect server capabilities to ensure the correct sync protocol is selected. |
| Persistence Regressions | MEDIUM | LOW | Verify `SqliteStateStore` initialization matches 0.8.0 requirements. |

## Success Criteria

1. The project compiles without errors via `cargo check`.
2. Session restoration works correctly with existing credentials.
3. The synchronization loop stabilizes and successfully reaches the `Running` state without 404 errors.
4. Room list updates correctly propagate to the UI shell.
