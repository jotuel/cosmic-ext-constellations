# Design Document: claw - Phase 2: The Room List

## 1. Objective
Implement a functional, COSMIC-native navigation sidebar for 'claw' that displays the Matrix room list with rich metadata (unread counts, message snippets).

## 2. Scope & Boundary
- **In Scope**:
    - Extending `RoomData` with `unread_count` and `last_message`.
    - Bridging all `VectorDiff` events into the UI shell.
    - Implementing `cosmic::widget::nav::Nav` for the sidebar.
    - Basic selection logic for switching rooms.
- **Out of Scope**:
    - Avatar loading and image decryption (deferred).
    - Room categorization or folders.

## 3. Proposed Solution
- **Data Layer**: Use `matrix_sdk_ui`'s `RoomListService` to extract metadata. Update `MatrixEngine` to emit `RoomUpdated` events.
- **Bridge**: Enhance the `Subscription` in `main.rs` to correctly apply insertions, deletions, and updates to the local `room_list` vector.
- **UI**: Employ `libcosmic`'s `Nav` widget for a native look and feel, using placeholders for room icons.

## 4. Trade-offs
- Deferred full avatar support to prioritize list synchronization performance.
- Use of generic placeholders for icons.

## 5. Verification
- `cargo test`: Verify data extraction and diff mapping.
- `cargo build`: Verify UI integration.
