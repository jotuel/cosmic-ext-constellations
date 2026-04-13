## 2024-06-12 - Prevent unneeded string allocations in `libcosmic` text elements

**Learning:** Using `clone()` on strings when pushing to `cosmic::widget::text::body` in loops causes noticeable allocation overhead because text elements accept `impl Into<Cow<'_, str>>`, meaning they can natively accept borrowed slices.
**Action:** Use `.as_deref().unwrap_or(fallback)` to borrow string slices (`&str`) dynamically without allocating, providing significant speedups for dynamic layout construction.
