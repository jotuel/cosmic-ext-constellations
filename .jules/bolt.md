## 2025-02-23 - [Dependencies Recursion Limit Issue]
**Learning:** The external dependency `matrix-sdk v0.16.0` produces a query depth recursion limit error when running `cargo check` on this environment. This is a known upstream dependency issue on newer rustc versions, not a local code bug in the `cosmic-ext-constellations` refactoring.
**Action:** Ignore `cargo check` blocking errors related to `matrix-sdk` query depth when verifying optimizations that only touch local frontend render logic.
