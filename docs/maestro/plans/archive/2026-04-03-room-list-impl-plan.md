# Implementation Plan: claw - Phase 2: The Room List

## 1. Objective
Implement a functional, COSMIC-native navigation sidebar for 'claw' that displays the Matrix room list with rich metadata.

## 2. Execution Strategy

| Phase | ID | Agent | Objective | Parallel | Blocked By |
|---|---|---|---|---|---|
| Foundation | 1 | `coder` | Extend `RoomData` and Matrix events for rich metadata. | No | - |
| Integration | 2 | `coder` | Update `subscription` to handle full `VectorDiff` stream. | No | 1 |
| UI Shell | 3 | `coder` | Implement `cosmic::widget::nav` sidebar. | No | 2 |

## 3. Phase Details

### Phase 1: Rich Room Metadata
- **Objective**: Enhance `RoomData` with unread counts and last message snippets.
- **Agent**: `coder`
- **Files to Modify**:
    - `src/matrix/mod.rs`:
        - Update `RoomData` struct with `unread_count` and `last_message`.
        - Add `MatrixEvent::RoomUpdated(usize, RoomData)`.
        - Implement internal helpers for room metadata extraction.
- **Validation**: `cargo test`

### Phase 2: Enhanced Stream Subscription
- **Objective**: Robustly bridge all `VectorDiff` events into the UI update loop.
- **Agent**: `coder`
- **Files to Modify**:
    - `src/main.rs`:
        - Update `subscription` logic to handle all `VectorDiff` variants.
        - Update `update` logic for precise list synchronization.
- **Validation**: `cargo check`

### Phase 3: COSMIC Navigation Sidebar
- **Objective**: Render the room list using `cosmic::widget::nav`.
- **Agent**: `coder`
- **Files to Modify**:
    - `src/main.rs`:
        - Replace current sidebar with `cosmic::widget::nav::Nav`.
        - Implement selection logic and styling for room entries.
- **Validation**: `cargo build`
