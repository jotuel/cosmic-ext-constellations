## 2024-05-24 - Efficient String Cloning in Iced UI rendering loops

**Learning:** `iced` rendering loops invoke view functions and map message payloads continuously on every frame, causing performance degradation when deep-copying `String` allocations into message enums via `.clone()`.

**Action:** Wrap string-heavy UI loop fields (e.g. unique identifiers like `device_id`) inside `std::sync::Arc<str>`. This converts O(N) allocation and copy costs per frame into O(1) atomic reference count increments.
