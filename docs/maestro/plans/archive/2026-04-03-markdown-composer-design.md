# Design Document: claw - Phase 4: The Markdown Composer

## 1. Objective
Implement a robust Markdown composer for 'claw' with multi-line editing support, live preview, and formatted message sending.

## 2. Scope & Boundary
- **In Scope**:
    - `cosmic-edit` integration for multi-line editing.
    - `segmented_button` for the Edit/Preview toggle.
    - Rich Markdown preview using `pulldown-cmark`.
    - Plaintext and HTML body sending via `matrix.send_message`.
- **Out of Scope**:
    - Image uploading.
    - Message drafting (auto-save).

## 3. Proposed Solution
- **State Layer**: Continue using `composer_text` as the source of truth.
- **Toggle Layer**: Use a `SegmentedButton` to provide a native toggle experience.
- **Editor Layer**: Integrate `cosmic-edit` (or its `libcosmic` wrapper) to replace the single-line `text_input`.
- **Preview Layer**: Develop a `view_preview` helper that handles headings, lists, bold, italic, and code blocks by mapping `pulldown-cmark` events to COSMIC text widgets.

## 4. Trade-offs
- **Custom Markdown View**: We'll use a `Column` of text widgets for the preview initially, which is simpler but less flexible than a full webview or specialized renderer.
- **Editor Complexity**: If `cosmic-edit` proves too complex for the current `libcosmic` version, we'll fall back to a standard `scrollable` `text_input` if available.

## 5. Verification
- `cargo check`: Ensure `cosmic-edit` is correctly linked.
- `cargo build`: Verify UI integration and preview rendering.
