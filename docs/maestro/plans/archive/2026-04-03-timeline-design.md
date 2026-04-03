# Design Document: claw - Phase 3: The Timeline (Chat View)

## 1. Objective
Implement a robust, real-time chat timeline for 'claw', supporting back-pagination and rich message rendering using COSMIC-native widgets.

## 2. Scope & Boundary
- **In Scope**:
    - `Timeline` initialization and management.
    - Subscribing to timeline events using the `matrix-sdk-ui` API.
    - Implementing message bubbles with sender and timestamp.
    - Basic back-pagination (fetching older messages).
- **Out of Scope**:
    - Media attachments (images, files) rendering.
    - Message reactions and replies UI.
    - End-to-end encryption details (handled by the SDK, but no specialized UI for keys).

## 3. Proposed Solution
- **Data Layer**: Extend `MatrixEngine` to manage active timelines and handle pagination requests asynchronously.
- **Bridge**: Utilize `Claw::subscription` to pipe timeline diffs into the application state using `VectorDiff`.
- **UI**: Develop a message bubble widget that renders Markdown using `pulldown-cmark`. Use a standard `scrollable` `column` with an auto-loading or "Load More" trigger for pagination.

## 4. Trade-offs
- **Virtualized List**: If a specialized `VerticalList` is unavailable, we'll use a `scrollable` `column` but ensure efficient state updates via `VectorDiff` to minimize re-renders.
- **Image Previews**: Postponed to favor core message and history functionality.

## 5. Verification
- `cargo test`: Verify timeline management in the engine.
- `cargo build`: Verify UI integration and responsiveness.
