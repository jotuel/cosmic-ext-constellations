---
name: sentinel
description: Security and correctness checks, panic prevention, safe input parsing, cryptographic guidelines, and async thread blocking warnings.
---

# COSMIC/Rust Security & Stability Checklist

This skill documents security principles, panic-prevention strategies, and async safety guidelines derived from previous vulnerabilities in the codebase.

## 1. Input Validation & Panic Prevention
* **Unwrapped Parsing on External Strings**: Assuming external or user-provided identifiers (such as Room IDs, aliases, or domain strings) are syntactically valid and calling `.unwrap()` on them (e.g. `RoomId::parse(...).unwrap()`) leads to Denial of Service (DoS) crashes if malformed data is processed.
  - *Action:* Always handle parsing methods returning a `Result` or `Option` gracefully. Propagate or map parsing errors to user-facing or log errors instead of panicking.

## 2. Cryptographic Security & Passphrase Generation
* **Predictable Entropy Fallbacks**: Cryptographic key or passphrase generators must never use weak fallbacks (like time-based PRNGs or `SystemTime`) if the system entropy source (e.g., `/dev/urandom`) is temporarily unavailable.
  - *Action:* Cryptographic tasks must fail closed and abort immediately if safe entropy is missing.
* **Non-Portable Entropy Access**: Hardcoding filesystem reads directly to `/dev/urandom` introduces bugs on non-Linux platforms (like Windows) and introduces blocking disk I/O in async contexts.
  - *Action:* Use standard, cross-platform cryptographic crates (e.g. `rand` with the `std_rng` or `os_rng` capabilities) instead of raw file system reads.
* **Direct Cryptographic RNG**: For critical master passphrases, use `rand::rngs::OsRng` directly with secure buffers (e.g. `try_fill_bytes`) to ensure maximum entropy and direct error feedback from the OS.

## 3. Asynchronous Execution Safety
* **Blocking Async Executors via Sync I/O**: Performing blocking synchronous operations (like `std::fs::read` to load avatar images or files) inside asynchronous `async` executor tasks (like `Task::perform` or `tokio::spawn`) blocks the execution thread. This can severely degrade responsiveness or cause deadlocks/DoS.
  - *Action:* Always use non-blocking async equivalents (like `tokio::fs::read`) inside async contexts so the executor can yield and handle other futures concurrent with the disk I/O.
