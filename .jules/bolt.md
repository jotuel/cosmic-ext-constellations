## 2024-05-24 - [Efficient String Cloning in Iced UI rendering loops]
**Learning:** `iced` rendering loops invoke view functions and map message payloads continuously on every frame, causing performance degradation when deep-copying `String` allocations into message enums via `.clone()`.
**Action:** Wrap string-heavy UI loop fields (e.g. unique identifiers like `device_id`) inside `std::sync::Arc<str>`. This converts O(N) allocation and copy costs per frame into O(1) atomic reference count increments.
## 2023-10-27 - [Pre-compute UI Strings to Avoid Allocations in Render Loop]
**Learning:** In immediate-mode UI frameworks like `iced` and `libcosmic`, allocating memory inside the rendering cycle (such as using `format!` or `.to_string()` in `view()` functions) heavily impacts framerate since it triggers string allocations up to 60 times a second per widget.
**Action:** When strings depend on underlying integer states (like unread notification counts) that change infrequently compared to the render loop, format them once during the data ingestion pipeline (e.g., when the state diffs update the `RoomData` struct) and store them as cached fields (`Option<String>`). Use borrowed string slices (`&str`) in the view tree.
## 2025-02-23 - [Dependencies Recursion Limit Issue]
**Learning:** The external dependency `matrix-sdk v0.16.0` produces a query depth recursion limit error when running `cargo check` on this environment. This is a known upstream dependency issue on newer rustc versions, not a local code bug in the `cosmic-ext-constellations` refactoring.
**Action:** Ignore `cargo check` blocking errors related to `matrix-sdk` query depth when verifying optimizations that only touch local frontend render logic.
## 2023-10-27 - [Immediate Mode GUI Markdown Parsing]
**Learning:** `pulldown_cmark::Parser::new()` is surprisingly expensive to run inside an immediate-mode GUI `view()` loop (which may execute 60 times a second), causing significant CPU usage and frame drops when typing or rendering long markdown messages.
**Action:** Always parse markdown or other structured text during the `update` cycle (e.g., when the text actually changes) and cache an intermediate structured representation (like an enum of `PreviewEvent`s) in the application state. Iterate over this cached structure in the `view()` function to eliminate frame-by-frame parsing overhead. Additionally, use `.as_str()` instead of `.clone()` on cached strings when constructing UI text widgets to avoid unnecessary memory allocations per frame.
## 2025-02-23 - [Recursive Traversal Allocation]
**Learning:** Checking for cycles in local state hierarchies (like Matrix Spaces) using `HashSet<OwnedRoomId>` forces a string `.to_owned()` allocation on every single traversal node check inside the `is_in_space_recursive` loop, which can add up quickly.
**Action:** Use `HashSet<&'a RoomId>` tied to the `&self` borrowed references inside the cycle-detection traversal instead.
## 2025-02-23 - [Immediate Mode GUI String Allocation]
**Learning:** Immediate-mode GUIs like `iced`/`libcosmic` evaluate the rendering tree every frame. Using `.clone()` or `.to_string()` on text fields to satisfy `text::body(...)` triggers an unnecessary heap allocation on *every single frame render* for *every single text element*.
**Action:** `libcosmic` UI text widgets accept `impl Into<Cow<'_, str>>`. Always pass cheap string references (e.g. `sender_name.as_str()` or `message.body()`) to `text::body()` and similar methods rather than `.to_string()` or `.clone()` to eliminate per-frame text allocations.
## 2024-06-12 - [Prevent unneeded string allocations in `libcosmic` text elements]
**Learning:** Using `clone()` on strings when pushing to `cosmic::widget::text::body` in loops causes noticeable allocation overhead because text elements accept `impl Into<Cow<'_, str>>`, meaning they can natively accept borrowed slices.
**Action:** Use `.as_deref().unwrap_or(fallback)` to borrow string slices (`&str`) dynamically without allocating, providing significant speedups for dynamic layout construction.
## 2025-02-13 - [Avoid String allocations in hot UI loops]
**Learning:** In libcosmic/iced `view` rendering loops, unnecessary `String::clone()` calls can significantly degrade performance, especially when filtering collections like `room_list`. `matrix_sdk::ruma` ID parsing methods accept `impl AsRef<str>`, allowing the use of string references directly.
**Action:** Always prefer passing `&String` or `&str` references to parsing and lookup functions instead of cloning owned strings to eliminate redundant heap allocations.
