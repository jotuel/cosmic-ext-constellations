# Implementation Plan: claw - Phase 3: The Timeline (Chat View)

## 1. Objective
Implement a functional, real-time chat timeline for 'claw' with support for back-pagination and rich message rendering.

## 2. Execution Strategy

| Phase | ID | Agent | Objective | Parallel | Blocked By |
|---|---|---|---|---|---|
| Foundation | 1 | `coder` | Implement `Timeline` management and back-pagination logic. | No | - |
| UI & View | 2 | `coder` | Implement message bubbles and the timeline list view. | No | 1 |

## 3. Phase Details

### Phase 1: Foundation & Data Flow
- **Objective**: Ensure `MatrixEngine` can manage active timelines and perform back-pagination.
- **Agent**: `coder`
- **Files to Modify**:
    - `src/matrix/mod.rs`:
        - Store active `Timeline` in `MatrixEngineInner` (as a `HashMap<RoomId, Arc<Timeline>>` or similar).
        - Implement `MatrixEngine::paginate_backwards(room_id, limit)`.
- **Validation**: `cargo test`

### Phase 2: UI & View
- **Objective**: Render message bubbles and the timeline list with back-pagination support.
- **Agent**: `coder`
- **Files to Modify**:
    - `src/main.rs`:
        - Implement `view_timeline()` helper to render message bubbles.
        - Add "Load More" button to the top of the scrollable list.
        - Map `TimelineItem` variants (Event, Virtual) correctly.
- **Validation**: `cargo build`

## 4. Cost Estimation

| Phase | Agent | Model | Est. Input | Est. Output | Est. Cost |
|-------|-------|-------|-----------|------------|----------|
| 1 | coder | Flash | 2500 | 1000 | $0.007 |
| 2 | coder | Flash | 3000 | 1500 | $0.010 |
| **Total** | | | **5500** | **2500** | **$0.017** |

Approve this implementation plan to begin execution?

1. Approve plan
2. Revise plan
3. Abort execution
