---
title: "Support for Matrix Spaces"
created: "2026-04-06T12:00:00Z"
status: "draft"
authors: ["TechLead", "User"]
type: "design"
design_depth: "deep"
task_complexity: "complex"
---

# Support for Matrix Spaces Design Document

## Problem Statement

The `cosmic-ext-constellations` Matrix client currently lacks support for Matrix Spaces (MSC1772), presenting a flat list of rooms that can become overwhelming for users in many communities. This design aims to implement a scalable, real-time "Space Switcher" that allows users to organize and filter their rooms through space hierarchies.

- **Design Choice**: A dedicated **Space Switcher** component.
- **Rationale**: Provides the best visual separation and scalability for power users with complex hierarchies. [Traces To: REQ-1]
- **Design Choice**: **Recursive Space Selection**.
- **Rationale**: Ensures a flexible UX where selecting a parent space shows all relevant rooms in its sub-spaces. [Traces To: REQ-2]

## Requirements

### Functional Requirements

1. **REQ-1**: A vertical **Space Switcher** must be added to the far left of the UI.
2. **REQ-2**: Selecting a space must filter the room list to show rooms in that space and its sub-spaces (Recursive Selection).
3. **REQ-3**: The Engine must maintain a real-time **Space Hierarchy** model by listening for `m.space.child`/`m.space.parent` events.
4. **REQ-4**: A "Global" space (All Rooms) must be the first entry in the space switcher.
5. **REQ-5**: The room list must be paginated via **Sliding Sync** to handle thousands of rooms efficiently.

### Non-Functional Requirements

1. **NFR-1 (Performance)**: Switching spaces should update the room list in under 200ms for locally cached rooms.
2. **NFR-2 (Scalability)**: The hierarchy graph must support at least 1,000 spaces without noticeable lag in UI updates.
3. **NFR-3 (Reliability)**: The hierarchy must recover automatically from sync errors or state-event gaps.

### Constraints

- **CON-1**: Use the existing `matrix-sdk` (0.16.0) and `libcosmic` with `iced`.
- **CON-2**: Adhere to the "Engine and Shell" pattern to keep UI and logic decoupled.
- **CON-3**: Maintain compatibility with homeservers supporting Sliding Sync (MSC3575).

## Approach

### Selected Approach: Hierarchical Sliding Sync

This approach fully integrates Matrix Spaces into the `MatrixEngine` by maintaining a real-time `SpaceHierarchy` and applying filters to the `RoomListService`.

- **Design Choice**: **Engine-Side Active Space State**.
- **Rationale**: The Engine handles the active space and filters the room list for the UI to ensure efficiency. [Traces To: REQ-2, REQ-5]
- **Design Choice**: **State Event Listeners** for Hierarchy Updates.
- **Rationale**: Provides real-time updates for the best UX when spaces are added or renamed. [Traces To: REQ-3, NFR-3]
- **Design Choice**: **Pagination** via Sliding Sync's `RoomListService`.
- **Rationale**: Scales to thousands of rooms by only loading visible entries. [Traces To: REQ-5, NFR-2]

### Alternatives Considered

#### Metadata-Filtered Room List (Pragmatic)

- **Description**: A simpler implementation that avoids deep hierarchy traversal and Sliding Sync filter manipulation. It uses a flat mapping of rooms to their immediate parent spaces and performs filtering at the UI layer.
- **Pros**: Much faster to implement; avoids complex graph logic and custom SDK filters.
- **Cons**: No recursive selection support; less flexible for power users.
- **Rejected Because**: Does not meet the requirements for recursive selection and real-time hierarchy sync requested by the user.

### Decision Matrix

| Criterion | Weight | Approach 1: Hierarchical | Approach 2: Metadata-Filtered |
| :--- | :--- | :--- | :--- |
| **Scalability** | 30% | **5**: Sliding Sync handles large lists perfectly. | **3**: UI-side filtering may lag with 1k+ rooms. |
| **UX Quality** | 30% | **5**: Recursive selection and real-time updates. | **2**: No recursive selection; less flexible. |
| **Impl. Speed** | 20% | **2**: Requires custom filters and graph logic. | **5**: Simple flat mapping and basic filtering. |
| **Maintainability**| 20% | **4**: Solid architecture but more code. | **4**: Simple code but limited for future growth. |
| **Weighted Total**| | **4.2** | **3.3** |

## Architecture

### Component Diagram

```
[ MatrixEngine (Engine) ]
      |
      |--- [ SpaceHierarchy Graph ]
      |--- [ active_space_id ]
      |--- [ RoomListService (Filtered) ]
      |
[ Constellations (UI Shell) ]
      |
      |--- [ SpaceSwitcher (NavBody) ]
      |--- [ Sidebar (Filtered List) ]
```

- **SpaceHierarchy Graph** — Tracks parent-child relations for spaces. * rationale: Efficient traversal for recursive selection. [Traces To: REQ-3] *(considered: flat mapping — rejected because it doesn't support deep hierarchies)*
- **active_space_id** — Tracks the user's current space selection. * rationale: Centralized state for Engine-side filtering. [Traces To: REQ-2]
- **SpaceSwitcher (NavBody)** — A new vertical icon bar for space navigation. * rationale: Best scalability for many spaces. [Traces To: REQ-1] *(considered: nested tree — rejected because of clutter)*

### Data Flow

1. **Sync**: `MatrixEngine` receives `m.space.child`/`m.space.parent` events and updates `SpaceHierarchy`. [Traces To: REQ-3]
2. **Navigation**: User clicks a space icon in `SpaceSwitcher`. [Traces To: REQ-1]
3. **Filtering**: UI sends `Message::SelectSpace(id)` -> Engine updates `active_space_id` and recalculates the `RoomListService` filter. [Traces To: REQ-2, REQ-5]
4. **UI Update**: Engine emits `MatrixEvent::RoomDiff` to the UI, which updates the `Sidebar`. [Traces To: REQ-5, NFR-1]

### Key Interfaces

```rust
// SpaceHierarchy Graph
pub struct SpaceHierarchy {
    parent_map: HashMap<OwnedRoomId, Vec<OwnedRoomId>>,
    child_map: HashMap<OwnedRoomId, Vec<OwnedRoomId>>,
}

impl SpaceHierarchy {
    pub fn is_in_space(&self, room_id: &RoomId, space_id: &RoomId) -> bool;
    pub fn update_from_event(&mut self, event: &StateEvent);
}

// MatrixEngine Updates
pub enum MatrixEvent {
    SpaceHierarchyChanged,
    RoomDiff(RoomListDiff),
}

pub enum Message {
    SelectSpace(Option<OwnedRoomId>), // None for "Global" view
}
```

## Agent Team

| Phase | Agent(s) | Parallel | Deliverables |
|-------|----------|----------|--------------|
| 1     | architect, coder | No | SpaceHierarchy implementation and MatrixEngine updates. |
| 2     | ux_designer, coder | No | SpaceSwitcher UI and sidebar filtering logic. |
| 3     | tester | Yes | Unit tests for hierarchy and filtering in src/matrix/tests.rs. |

## Risk Assessment

| Risk | Severity | Likelihood | Mitigation |
|------|----------|------------|------------|
| Circular Space Hierarchies | MEDIUM | LOW | Implement cycle detection or max recursion depth. [Traces To: NFR-2] |
| Performance Lag | HIGH | MEDIUM | Use a flattened room-to-space cache updated on state events. [Traces To: NFR-1] |
| Sliding Sync Compatibility | MEDIUM | MEDIUM | Graceful server capability handling by matrix-sdk-ui. [Traces To: CON-3] |

## Success Criteria

1. SC-1: The UI contains a vertical **Space Switcher** with icons for all spaces. [Traces To: REQ-1]
2. SC-2: Selecting a space correctly filters the room list (recursive and real-time). [Traces To: REQ-2]
3. SC-3: The Engine passes **all new unit tests** in `src/matrix/tests.rs`. [Traces To: REQ-3, NFR-3]
4. SC-4: Performance benchmarks show switching spaces is **under 200ms**. [Traces To: NFR-1]
5. SC-5: Handles **circular space references** gracefully without crashing. [Traces To: NFR-2, NFR-3]
