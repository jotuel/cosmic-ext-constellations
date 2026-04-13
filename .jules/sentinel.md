## 2025-02-18 - Uncaught Panic on Invalid Room ID in Space Settings
**Vulnerability:** A Denial of Service (DoS) vulnerability existed in `src/settings/room.rs` where calling `RoomId::parse(&room_id_clone).unwrap()` could trigger a panic if the provided `room_id_clone` string was not a valid Matrix Room ID format.
**Learning:** This existed because the parsing logic assumed that any `room_id` variable passed to the room settings would always be syntactically valid, neglecting the possibility of malformed data or unexpected inputs during application state transitions, leading to an application crash.
**Prevention:** Always handle standard library and dependency `Result` or `Option` returns gracefully when processing external or internal string identifiers. Use `map_err` or `ok_or` to convert parsing failures into handled application errors rather than using `.unwrap()`.
## 2024-05-18 - Insecure E2E Store Passphrase Generation Fallback
**Vulnerability:** The `get_or_create_store_passphrase` function used a time-based pseudo-random generator as a fallback for the E2E store encryption key when `/dev/urandom` failed, which made the key predictable. It also ignored errors when reading from `/dev/urandom`.
**Learning:** Security-critical cryptography, like a master key/passphrase generation, should never fail-open or fall back to an insecure entropy source like `SystemTime`. If sufficient entropy is unavailable, the application must abort securely.
**Prevention:** Always propagate errors for secure operations (e.g., using `read_exact` and returning `Result`) rather than swallowing errors and employing a weak fallback.
## 2024-05-24 - Async Executor Blocking via Synchronous File I/O
**Vulnerability:** Synchronous `std::fs::read` was used inside an async `Task::perform` block when reading avatar images.
**Learning:** Calling blocking synchronous APIs (like disk I/O) within async blocks can block the executor thread, leading to performance degradation or Denial of Service (DoS) in high-load scenarios.
**Prevention:** Always use asynchronous equivalents (`tokio::fs::read`) when performing I/O operations inside `async` blocks or tasks to ensure the executor can yield and continue processing other futures.
## 2025-02-21 - Non-Portable CSPRNG Source
**Vulnerability:** Reading directly from `/dev/urandom` for secure passphrase generation instead of a portable CSPRNG.
**Learning:** Hardcoding standard paths like `/dev/urandom` introduces bugs on platforms lacking these files (like Windows), and synchronous blocking file I/O operations can degrade asynchronous executor performance leading to DoS risks.
**Prevention:** Always use standard cryptographic crates (like `rand` with the `std_rng` or `os_rng` capabilities) instead of raw file system accesses when generating cryptographic materials.
