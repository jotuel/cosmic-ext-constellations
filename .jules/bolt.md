## 2025-02-13 - Avoid String allocations in hot UI loops
**Learning:** In libcosmic/iced `view` rendering loops, unnecessary `String::clone()` calls can significantly degrade performance, especially when filtering collections like `room_list`. `matrix_sdk::ruma` ID parsing methods accept `impl AsRef<str>`, allowing the use of string references directly.
**Action:** Always prefer passing `&String` or `&str` references to parsing and lookup functions instead of cloning owned strings to eliminate redundant heap allocations.
