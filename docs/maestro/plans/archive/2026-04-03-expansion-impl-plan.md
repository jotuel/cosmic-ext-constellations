# Implementation Plan: claw - Refinement & Expansion (Phases 5-7)

## 1. Objective
Enhance 'claw' with rich UI/UX (avatars, timestamps, reactions), media support (images/files), and advanced Matrix features (room creation, OIDC).

## 2. Execution Strategy

| Phase | ID | Agent | Objective | Parallel | Blocked By |
|---|---|---|---|---|---|
| Refinement | 1 | `coder` | Implement avatars, timestamps, and message reactions. | No | - |
| Media | 2 | `coder` | Implement media (images/files) fetching and rendering. | No | 1 |
| Advanced | 3 | `coder` | Implement room creation and OIDC login flow. | No | 2 |

## 3. Phase Details

### Phase 1: Refinement (Avatars & Reactions)
- **Objective**: Add avatars to the room list and timeline, format timestamps, and support reactions.
- **Files to Modify**:
    - `src/matrix/mod.rs`:
        - Update `RoomData` and `TimelineItem` extraction to include avatar URLs and timestamps.
        - Handle `Reaction` events.
    - `src/main.rs`:
        - Render avatars using `cosmic::widget::image` (with fallback).
        - Display formatted timestamps and reaction chips.
- **Validation**: `cargo build`

### Phase 2: Media Support (Images & Files)
- **Objective**: Fetch, decrypt, and render images and file attachments.
- **Files to Modify**:
    - `src/matrix/mod.rs`: Implement `fetch_media` and `decrypt_media` helpers.
    - `src/main.rs`: 
        - Add `MediaPreview` component to `view_timeline`.
        - Handle async media loading state.
- **Validation**: `cargo check`

### Phase 3: Advanced Features (Room Creation & OIDC)
- **Objective**: Implement the OIDC login flow and a room creation dialog.
- **Files to Modify**:
    - `src/matrix/mod.rs`: Implement `login_oidc` and `create_room`.
    - `src/main.rs`:
        - Add "Create Room" button and dialog.
        - Handle OIDC redirect messages.
- **Validation**: `cargo build`

## 4. Cost Estimation

| Phase | Agent | Model | Est. Input | Est. Output | Est. Cost |
|-------|-------|-------|-----------|------------|----------|
| 1 | coder | Flash | 3000 | 1200 | $0.008 |
| 2 | coder | Flash | 3500 | 1500 | $0.010 |
| 3 | coder | Flash | 4000 | 1800 | $0.012 |
| **Total** | | | **10500** | **4500** | **$0.030** |

Approve this implementation plan to begin execution?

1. Approve plan
2. Revise plan
3. Abort execution
