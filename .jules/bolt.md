## 2024-04-24 - [Reduced RwLock contention on `SpaceHierarchy`]
**Learning:** `update_filtered_rooms` heavily iterated through `self.room_list` calling `MatrixEngine::is_in_space_sync` iteratively. The underlying state access acquired an `RwLock` for each query.
**Action:** Created `is_in_space_bulk` to retrieve all boolean checks in a single read lock. This reduces locking overhead N-fold. It is important to structure `bulk` checks so they correctly handle iterators locally while retaining safe short-circuit mechanisms, and only extract string matches or object references once to avoid string parsing deduplications.

## 2024-04-27 - [Optimized Case-Insensitive String Filtering]
**Learning:** In hot loops like `update_filtered_rooms`, calling `.to_lowercase()` unconditionally on every string just to do case-insensitive filtering (`.contains()`) allocates strings on the heap unnecessarily, slowing down the application during searches. We can utilize an ASCII-only fast path by using `haystack.as_bytes().windows(len).any(|w| w.eq_ignore_ascii_case(needle_bytes))` to dramatically decrease allocations.
**Action:** When performing simple case-insensitive searches in loops, check if the query string is ASCII. If it is, use byte-wise windowed comparisons instead of converting strings to lowercase globally, only falling back for non-ASCII text.

## 2024-04-27 - [Correct Fallback Handling in Loop Filtering optimizations]
**Learning:** When trying to prevent `.to_lowercase()` string allocations inside a hot loop (like a case-insensitive search loop), do NOT initialize the fallback string (`search_query_lower`) conditionally to an empty string (`String::new()`). `n.to_lowercase().contains("")` evaluates to true, breaking the filter by matching all items with non-ASCII text.
**Action:** Always eagerly initialize the lowercase fallback string outside the loop (`let search_query_lower = search_query.to_lowercase();`) if you still intend to use `.contains(&search_query_lower)` as the fallback branch within the loop for strings that fail the `.is_ascii()` check.

## 2024-05-30 - [Optimize bulk space filter]
**Learning:** To reduce RwLock contention, batch filtering logic utilizing iterators directly inside the lock method avoids overheads of massive `Vec` capacity preallocations and iterative atomic locks.
**Action:** When migrating N loops on locked traits, use internal iterator callbacks inside a scoped read guard instead of pre-collecting into N-size Vectors to fetch them piecemeal.
## 2024-06-03 - [Refactored `filtered_room_list` to store indices]
**Learning:** In Rust UI apps that sync large lists of data (like Matrix rooms), avoid `Vec::clone` on large structs. We previously kept `filtered_room_list` as `Vec<RoomData>`, which caused massive `O(N)` heap allocations containing string clones every single keystroke.
**Action:** Changed `filtered_room_list` to store indices (`Vec<usize>`) referencing `room_list` instead. When rendering or applying filtering, just lookup `&room_list[idx]`. This reduced search/filter update overhead effectively.
## 2024-06-25 - [Optimize bulk space filter traversals]
**Learning:** Checking Space inclusions via upward `is_child_of_recursive` traversal required parsing string `room.id` into `RoomId` for every room in the app state on every keystroke, leading to `O(N * depth)` overhead and high allocation. Space hierarchy children are a mirror of parents.
**Action:** In `cosmic-ext-constellations`, to optimize bulk Space filtering, avoid iterative upward `SpaceHierarchy` tree traversals and string parsing (`RoomId::parse`) per room. Instead, precompute the descendants of the target space via a downward traversal (`get_descendants_strs`) into a `HashSet<&str>`, enabling `O(1)` allocations-free membership checks.
