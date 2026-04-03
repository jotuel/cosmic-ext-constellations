---
design_depth: standard
task_complexity: medium
---

# Design Document: Phase 2 - Room List

## 1. Problem Statement
The objective of Phase 2 is to implement the Room List (Sidebar) for the `claw` Matrix client, transitioning from the Phase 1 shell to a fully functional chat interface sidebar. The primary challenge is effectively bridging the asynchronous `matrix-sdk-ui` stream of room updates into the synchronous, reactive `libcosmic` UI loop without blocking or causing excessive memory consumption. We must establish a pattern for handling dynamic lists in the application state and rendering them using native widgets, specifically `cosmic::widget::Nav` — *[chosen to ensure the application maintains a native, consistent appearance within the COSMIC desktop environment]*. The room list must initialize all synced rooms at once — *[selected for simplicity in early development over complex lazy-loading]* — and must remain synchronized with the underlying SDK state as new messages arrive or rooms are updated.

## 2. Requirements

**Functional Requirements:**
1. Initialize the `matrix_sdk_ui::RoomListService` within the `MatrixEngine` during application startup.
2. Bridge the SDK's `RoomListDiff` stream into the `libcosmic` application update loop.
3. Manage a local collection (e.g., `Vec<RoomData>`) representing the ordered list of rooms within the `Claw` application state.
4. Update the local list strictly via index-based operations mapping to `Insert`, `Update`, `Remove`, and `Reset` — *[chosen to ensure efficient UI rendering without re-cloning the entire list on every message]*.
5. Render the local list using a `cosmic::widget::Nav` sidebar — *[chosen to provide native COSMIC theming, selection logic, and standard list interactions]*.

**Non-Functional Requirements:**
6. Maintain the decoupling between the async background `MatrixEngine` tasks and the reactive synchronous `Claw` shell.
7. Prevent locking or blocking the `matrix-sdk` internal state store during UI renders.

**Constraints:**
8. The design must accommodate the constraints of the `libcosmic` component model, requiring any custom items inside the `Nav` widget to be composed of available basic primitives.

## 3. Approach

**Selected Approach: Event-Sourced Room List**
The `MatrixEngine` will initialize the `matrix_sdk_ui::RoomListService` and subscribe to its default stream. The background `libcosmic` subscription task will map the resulting `RoomListDiff` items into corresponding `Message::Matrix` variants. The `Claw` application state will maintain a local `Vec<RoomData>`, and its `update` function will directly apply these diff operations to the vector — *[chosen to optimize performance and prevent allocating a new list on every Matrix event]*. 

The `view` function will iterate over this local vector to construct a `cosmic::widget::Nav` sidebar — *[chosen to leverage native COSMIC styling for the sidebar]*.

**Alternatives Considered:**
We considered a Full List Sync strategy where the background task sends a full clone of the room list to the UI thread on any change. This was rejected because it scales poorly for dynamic lists, forcing an allocation for every typing event or message. We also considered lazy loading but rejected it for this phase — *[prioritizing functional synchronization and simplicity over complex pagination logic in early development]*.

**Decision Matrix:**
| Criterion | Weight | Approach 1: Event-Sourced | Approach 2: Full Sync |
|-----------|--------|---------------------------|-----------------------|
| Performance | 50% | 5: Updates only changed items | 2: Allocates/clones entire list |
| Complexity | 25% | 3: Requires index diff logic in UI | 5: Trivial state replacement |
| Memory Usage | 25% | 4: Single UI copy | 2: Duplicate copies across threads|
| **Weighted Total** | | **4.25** | **2.75** |

## 4. Risk Assessment

1. Index Desynchronization (Medium Risk): If the local `Vec<RoomData>` falls out of sync with the SDK's internal state, index-based `RoomListDiff` operations could panic or modify the wrong room.
*Mitigation*: The UI `update()` loop must carefully handle all diff variants, especially `Reset` events, by completely replacing the vector with a fresh snapshot from the background stream.

2. Layout Rigidity (Low Risk): `cosmic::widget::Nav` items may lack the layout flexibility required to construct complex multi-line snippets with unread badges and timestamps natively.
*Mitigation*: We will attempt to compose standard text and icon primitives inside the nav item properties. If constraints are too rigid, we will fall back to a custom `Scrollable` `Column`.

3. Blocking the UI (Low Risk): Iterating over a massive list to build `libcosmic` UI nodes during `view()` may drop frames.
*Mitigation*: For Phase 2, we assume typical user room counts. If performance degrades later, we will explore virtualization or lazy loading in subsequent phases.