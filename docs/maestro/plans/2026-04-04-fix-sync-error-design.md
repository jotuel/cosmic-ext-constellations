---
design_depth: standard
task_complexity: medium
---

# Design: Fix Sync Error in Claw Matrix Client

## 1. Problem Statement
The Claw Matrix client currently monitors the `SyncService` state from the `matrix-sdk-ui` crate. When the service enters an "Error" state, the application transitions to a generic `SyncStatus::Error`, and the UI displays a simple "Sync Error" message. However, the specific cause of the error (e.g., authentication failure, network timeout, or server-side issue) is not captured or displayed. This lack of diagnostic detail prevents users from troubleshooting sync issues effectively and complicates the debugging process for developers. Our goal is to capture the underlying SDK error and provide it to the user through the status bar.

## 2. Requirements
- **Functional**:
    - Capture the specific error message provided by the SDK when the `SyncService` transitions to the "Error" state.
    - Update the `SyncStatus` enum in `src/matrix/mod.rs` to include a `String` payload for the `Error` variant.
    - Update the `MatrixEngine` logic to emit a `MatrixEvent::SyncStatusChanged(SyncStatus::Error(error_message))` whenever a sync failure is detected.
    - Update the `view` function in `src/main.rs` to display this error message as a tooltip or status bar detail when "Sync Error" is shown.
- **Non-Functional**:
    - Maintain the current async performance and responsiveness of the synchronization loop.
    - Provide a clear, diagnostic message that is useful for troubleshooting.
    - Avoid introducing large-scale changes to the existing status management flow.
- **Constraints**:
    - Must integrate seamlessly with the `matrix-sdk-ui` 0.7 `SyncService` state management.
    - Must preserve existing naming conventions and architectural boundaries in the `cosmic-ext-claw` codebase.

## 3. Approach
### Selected Approach: Unified Status Error Capture
- Update the `SyncStatus` enum in `src/matrix/mod.rs` to `SyncStatus::Error(String)`.
- Modify the `MatrixEngine` logic in `src/matrix/mod.rs` to capture the error when the `SyncService` transitions to the error state.
- Update the `view` function in `src/main.rs` to display this error message in the status bar detail.

### Alternatives Considered:
- Separate Error Notification: Keep `SyncStatus::Error` as-is and use a separate `MatrixEvent::SyncError(String)` for error details.
    - Rejected: This would lead to split state management for synchronization failures and more complex UI logic.

### Decision Matrix:
| Criterion | Weight | App 1: Unified Status | App 2: Separate Event |
|-----------|--------|------------|------------|
| State Consistency | 40% | 5: Single source of truth | 3: Status and error are separate |
| UI Logic Simplicity | 30% | 4: Unified handling | 3: Redundant state checks |
| Maintenance Overhead | 30% | 4: Clean architectural fit | 3: Potential for state desync |
| **Weighted Total** | | **4.4** | **3.0** |

## 4. Architecture
- **SyncStatus Update**: `src/matrix/mod.rs` updated with `SyncStatus::Error(String)`.
- **SDK Error Capture**: `MatrixEngine::start_sync` updated to capture the error when `SyncService::state()` transitions to `Error`.
- **Event Propagation**: `MatrixEngine` emits `MatrixEvent::SyncStatusChanged(SyncStatus::Error(error_message))`.
- **UI Reception**: `src/main.rs` update function handles the updated `SyncStatus` variant.
- **UI Display**: `self.view()` in `src/main.rs` renders the `SyncStatus::Error` text with the captured SDK error message as a detail in the status bar.

## 5. Agent Team
- `coder`: Implement the `SyncStatus` update and the error capture logic in `src/matrix/mod.rs`.
- `tester`: Add unit tests to verify error capture and propagation.
- `technical_writer`: Update documentation to reflect the new sync recovery behavior.

## 6. Risk Assessment
- **Opaque SDK Errors**: Some SDK errors might be cryptic or too long for a standard status bar label.
    - **Solution**: Truncate long error messages for the status bar and provide the full message as a tooltip or in a diagnostic detail area.
- **Match Coverage**: Updating the `SyncStatus` variant requires updating all `match` statements in `src/matrix/mod.rs` and `src/main.rs`.
    - **Solution**: Systematically identify all occurrences and update the `SyncStatus::Error` arms to correctly handle the new `String` payload.
- **Transient vs Permanent Errors**: Distinguishing between temporary network blips and permanent failures might be difficult for the user.
    - **Solution**: Use the SDK's internal error classification where possible and report the most informative message.

## 7. Success Criteria
- Sync automatically recovers from transient failures and reports SDK error details.
- The UI transitions from "Sync Error" to a detailed error message in the status bar.
- Exponential backoff correctly delays retries after successive failures.
