## 2024-05-24 - Async Executor Blocking via Synchronous File I/O
**Vulnerability:** Synchronous `std::fs::read` was used inside an async `Task::perform` block when reading avatar images.
**Learning:** Calling blocking synchronous APIs (like disk I/O) within async blocks can block the executor thread, leading to performance degradation or Denial of Service (DoS) in high-load scenarios.
**Prevention:** Always use asynchronous equivalents (`tokio::fs::read`) when performing I/O operations inside `async` blocks or tasks to ensure the executor can yield and continue processing other futures.
