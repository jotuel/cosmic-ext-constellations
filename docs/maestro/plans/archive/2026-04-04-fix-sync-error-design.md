---
design_depth: quick
task_complexity: medium
---

# Design: Fix Sync Error in Claw Matrix Client

## 1. Problem Statement
The Claw Matrix client currently reports a "Permanent Error Status" when its synchronization service fails. The root cause is a lack of error handling and recovery logic in the sync loop within `src/matrix/mod.rs`. When the `SyncService` enters an `Error` state, the background task spawned by `start_sync` stops, and the UI remains stuck in the "Sync Error" state. This prevents automatic recovery from intermittent network disruptions or transient server-side errors, requiring a full application restart to resume synchronization.

## 2. Requirements
- Functional:
    - Automatically retry the synchronization process when a "Permanent Error Status" is detected.
    - Implement an exponential backoff strategy for retries to avoid overloading the Matrix homeserver.
    - Log the specific sync failure for easier troubleshooting and diagnostic visibility.
- Non-Functional:
    - Retries must be performed in a background task without blocking the iced UI event loop.
    - Maintain a consistent `SyncStatus` in the `MatrixEngine` state that correctly reflects the current retry attempt status.
    - Preserve existing sync status reporting and room data management in `src/matrix/mod.rs`.

## 3. Approach
### Selected Approach: Background Task Retry Loop
- Wrap the `sync_service.start().await` call within a `loop` in the background task spawned by `start_sync`.
- Use `tokio::time::sleep` with exponential backoff to handle retries.
- Log each retry attempt and the result of the `start()` future if possible.

### Alternatives Considered:
- UI-driven Recovery Task: Monitor the `SyncStatus::Error` in the `Claw::update` method and dispatch a new `Message` to call `start_sync` again.
    - Rejected: This would spread sync logic across layers and make the UI logic more complex.
- Manual Reconnect Button: Require a user action to manually retry when it fails.
    - Rejected: This would lead to a poorer user experience for intermittent issues.

## 4. Architecture
- The `MatrixEngine::start_sync` method will spawn a new `tokio::task`.
- This task will maintain an internal `backoff` state.
- Inside a `loop`, it will call `sync_service.start().await`.
- If the future completes (indicating a stop or failure), it will check the `SyncService` state.
- If an error state is detected, the task will wait for an increasing duration (e.g., 2, 4, 8, 16 seconds) before retrying the `start()` call.
- The `SyncService::state()` stream in `src/main.rs` will continue to monitor the sync status as it transitions from `Syncing` to `Error` and back to `Syncing` during the retry process.

## 5. Agent Team
- `coder`: Implement the background retry loop in `src/matrix/mod.rs`.
- `tester`: Add a unit test to verify the retry mechanism.

## 6. Risk Assessment
- Overlapping Sync Tasks: If `start_sync` is called multiple times without properly cleaning up old tasks, it could lead to multiple concurrent sync loops.
- Solution: Ensure that only one background task is active for the sync service at any given time.
- Improper Backoff: An incorrectly implemented backoff could result in too many requests to the homeserver or an excessively long delay between retries.
- Solution: Implement a well-tested exponential backoff strategy with a reasonable maximum wait time (e.g., 60 seconds).
- Unhandled Termination: If the sync service is deliberately stopped, the retry loop could prevent a clean shutdown.
- Solution: Monitor the `SyncServiceState::Terminated` to correctly break the loop when synchronization is intended to end.

## 7. Success Criteria
- Sync automatically recovers from transient network failures.
- The UI transitions from "Sync Error" back to "Syncing" or "Connected" without a restart.
- Exponential backoff correctly delays retries after successive failures.
