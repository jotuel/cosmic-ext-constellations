---
title: "Support for Matrix Spaces Implementation Plan"
design_ref: "docs/maestro/plans/2026-04-06-spaces-support-design.md"
created: "2026-04-06T13:00:00Z"
status: "draft"
total_phases: 5
estimated_files: 3
task_complexity: "complex"
---

# Support for Matrix Spaces Implementation Plan

## Plan Overview

- **Total phases**: 5
- **Agents involved**: architect, coder, ux_designer, tester, code_reviewer
- **Estimated effort**: Moderate to High. Involves updating the core Matrix engine and the primary UI shell.

## Dependency Graph

```
Phase 1: Foundation (SpaceHierarchy)
    |
Phase 2: Core (Sync & Events)
    |
    +-------------------+
    |                   |
Phase 3: UI (Switcher)  Phase 5: Quality (Tests)  [Parallel]
    |                   |
    +-------------------+
    |
Phase 4: Integration (Filtering)
```

## Execution Strategy

| Stage | Phases | Execution | Agent Count | Notes |
|-------|--------|-----------|-------------|-------|
| 1     | Phase 1 | Sequential | 1 | Foundation (Types & Models) |
| 2     | Phase 2 | Sequential | 1 | Core Logic (Sync Loop) |
| 3     | Phase 3, 5| Parallel  | 2 | UI Design & Verification |
| 4     | Phase 4 | Sequential | 1 | Final Integration |

## Phase 1: Foundation (SpaceHierarchy Logic)

### Objective
Implement the `SpaceHierarchy` graph structure and update `RoomData` to include space-specific metadata.

### Agent: coder
### Parallel: No

### Files to Create
None.

### Files to Modify
- `src/matrix/mod.rs` — Define `SpaceHierarchy` struct, update `RoomData`, and add `is_space` / `parent_id` fields.

### Implementation Details
- `SpaceHierarchy` uses `HashMap<OwnedRoomId, Vec<OwnedRoomId>>` for parent and child mappings.
- Implement `SpaceHierarchy::is_in_space(room_id, space_id)` with cycle detection.
- Update `RoomData` to include `room_type: RoomType` (from `matrix-sdk`).

### Validation
- `cargo check` to ensure type safety.
- Verify `SpaceHierarchy` compiles as part of the `MatrixEngineInner`.

### Dependencies
- Blocked by: None
- Blocks: Phase 2, Phase 3, Phase 5

---

## Phase 2: Core (Sync & State Events)

### Objective
Update the `MatrixEngine` sync loop to observe `m.space.child` and `m.space.parent` state events and maintain the hierarchy graph.

### Agent: coder
### Parallel: No

### Files to Modify
- `src/matrix/mod.rs` — Update the event listener in the sync loop to handle space-related state events.

### Implementation Details
- Use `matrix_sdk::room::Room::get_state_event` or listen to the state event stream.
- Update `SpaceHierarchy` in real-time when these events are received.
- Ensure `MatrixEngineInner` holds an instance of `SpaceHierarchy`.

### Validation
- `cargo check`.
- Verify sync loop handles events without crashing or infinite recursion.

### Dependencies
- Blocked by: Phase 1
- Blocks: Phase 3, Phase 4, Phase 5

---

## Phase 3: UI (SpaceSwitcher Widget)

### Objective
Design and implement the vertical `SpaceSwitcher` bar in the `libcosmic` shell.

### Agent: ux_designer
### Parallel: Yes (with Phase 5)

### Files to Modify
- `src/main.rs` — Add the `SpaceSwitcher` component to the `NavBody` layout.

### Implementation Details
- Create a vertical `Column` or `NavBody::sidebar` for space icons.
- Add "Global" (All Rooms) icon as the first entry.
- Implement `Message::SelectSpace(Option<OwnedRoomId>)` to handle clicks.

### Validation
- `cargo run` (if possible in environment) or visual code review for layout consistency.
- Verify icons are rendered correctly in the new navigation bar.

### Dependencies
- Blocked by: Phase 2
- Blocks: Phase 4

---

## Phase 4: Integration (Filtering & Navigation)

### Objective
Connect the UI `SelectSpace` messages to the Engine and apply `RoomListService` filters.

### Agent: coder
### Parallel: No

### Files to Modify
- `src/matrix/mod.rs` — Implement `active_space_id` handling and `RoomListService` filtering.
- `src/main.rs` — Finalize message passing between UI and Engine.

### Implementation Details
- In `MatrixEngine::update`, handle `SelectSpace`.
- Use `RoomListService::all_rooms().filter(...)` to apply space-based filtering.
- Ensure the sidebar refreshes automatically when the filter changes.

### Validation
- `cargo check`.
- Manual verification of filtering logic (e.g., clicking space A only shows rooms in A).

### Dependencies
- Blocked by: Phase 3, Phase 5
- Blocks: None

---

## Phase 5: Quality (Unit Tests)

### Objective
Add unit tests in `src/matrix/tests.rs` to verify the `SpaceHierarchy` and filtering logic using mocked events.

### Agent: tester
### Parallel: Yes (with Phase 3)

### Files to Modify
- `src/matrix/tests.rs` — Add mock hierarchy and recursive filtering tests.

### Implementation Details
- Create mock room objects with `m.space.child` events.
- Test `is_in_space` with nested and circular hierarchies.
- Verify filter logic returns the correct set of rooms for a given space ID.

### Validation
- `cargo test matrix::tests`.
- Ensure all tests pass with high coverage of hierarchy edge cases.

### Dependencies
- Blocked by: Phase 2
- Blocks: Phase 4

---

## File Inventory

| # | File | Phase | Purpose |
|---|------|-------|---------|
| 1 | `src/matrix/mod.rs` | 1, 2, 4 | Hierarchy logic, Sync loop, and Filtering integration. |
| 2 | `src/main.rs` | 3, 4 | UI Layout, SpaceSwitcher widget, and Message handling. |
| 3 | `src/matrix/tests.rs` | 5 | Unit tests for hierarchy and filtering. |

## Risk Classification

| Phase | Risk | Rationale |
|-------|------|-----------|
| 1     | LOW | Foundation types are well-defined in the SDK. |
| 2     | MEDIUM | Managing state event streams can be complex. |
| 3     | MEDIUM | Layout changes in libcosmic/iced require careful alignment. |
| 4     | HIGH | Integration of filtering with the sync loop is the most complex step. |
| 5     | LOW | Unit tests are isolated and low-risk. |

## Execution Profile

```
Execution Profile:
- Total phases: 5
- Parallelizable phases: 2 (Phase 3 and Phase 5)
- Sequential-only phases: 3
- Estimated parallel wall time: ~4-6 hours (excluding review)
- Estimated sequential wall time: ~6-8 hours

Note: Native subagents currently run without user approval gates.
All tool calls are auto-approved without user confirmation.
```

## Cost Estimation

| Phase | Agent | Model | Est. Input | Est. Output | Est. Cost |
|-------|-------|-------|-----------|------------|----------|
| 1 | coder | Pro | 5k | 1k | $0.09 |
| 2 | coder | Pro | 6k | 1.5k | $0.12 |
| 3 | ux_designer | Pro | 8k | 2k | $0.16 |
| 4 | coder | Pro | 10k | 2k | $0.18 |
| 5 | tester | Pro | 6k | 2k | $0.14 |
| **Total** | | | **35k** | **8.5k** | **$0.69** |
