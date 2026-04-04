# Design Document: claw - Media Decryption & UI Polish

## 1. Objective
Complete the Matrix media pipeline with E2EE decryption, implement image rendering in the timeline, and add a formal UI error reporting system.

## 2. Scope & Boundary
- **In Scope**:
    - Update `MatrixEngine::fetch_media` to accept `MediaSource` and handle decryption.
    - Convert cached bytes to `iced::widget::image::Handle` for rendering.
    - Add an error status area to the main UI.
- **Out of Scope**:
    - Persistent disk-based media cache.
    - File system downloads (saving to ~/Downloads).

## 3. Proposed Solution
- **Decryption**: Leverage `matrix-sdk`'s `Media` service which handles both plain and encrypted sources transparently when `get_media_content` is called with the appropriate flag.
- **Rendering**: Update the `media_cache` in `Claw` to store `Handle` objects. Use the `cosmic::widget::image` widget to display these in the timeline.
- **Feedback**: Introduce an `error: Option<String>` field in the application state and render a dismissible banner in the view.

## 4. Verification
- `cargo build`: Ensure all widgets and engine methods link correctly.
- Manual test: Trigger a media download and verify the image appears in the bubble.
