# Implementation Plan: claw - Media Decryption & UI Polish

## 1. Objective
Complete the Matrix media pipeline with E2EE decryption, implement image rendering in the timeline, and add a formal UI error reporting system.

## 2. Execution Strategy

| Phase | ID | Agent | Objective | Parallel | Blocked By |
|---|---|---|---|---|---|
| Engine | 1 | `coder` | Implement E2EE media decryption in `MatrixEngine`. | No | - |
| UI Rendering | 2 | `coder` | Implement image rendering and UI status feedback. | No | 1 |

## 3. Phase Details

### Phase 1: Engine Refinement (Decryption)
- **Objective**: Update `fetch_media` to support transparent decryption of encrypted files.
- **Agent**: `coder`
- **Files to Modify**:
    - `src/matrix/mod.rs`:
        - Update `fetch_media` signature: `pub async fn fetch_media(&self, source: MediaSource) -> Result<Vec<u8>>`.
        - Use `client.media().get_media_content(&source, true)`.
- **Validation**: `cargo test`

### Phase 2: UI Rendering & Feedback
- **Objective**: Display fetched images in the timeline and implement an error notification area.
- **Agent**: `coder`
- **Files to Modify**:
    - `src/main.rs`:
        - Add `error: Option<String>` to `Claw`.
        - Implement `Message::DismissError`.
        - Convert `media_cache` bytes to `iced::widget::image::Handle`.
        - Update `view_timeline` to render `Image` widgets for downloaded content.
        - Add a status bar/toast at the bottom of the layout to display `error`.
- **Validation**: `cargo build`

## 4. Cost Estimation

| Phase | Agent | Model | Est. Input | Est. Output | Est. Cost |
|-------|-------|-------|-----------|------------|----------|
| 1 | coder | Flash | 2000 | 500 | $0.004 |
| 2 | coder | Flash | 3000 | 1200 | $0.008 |
| **Total** | | | **5000** | **1700** | **$0.012** |

Approve this implementation plan to begin execution?

1. Approve plan
2. Revise plan
3. Abort execution
