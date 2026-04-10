## 2024-05-18 - Insecure E2E Store Passphrase Generation Fallback
**Vulnerability:** The `get_or_create_store_passphrase` function used a time-based pseudo-random generator as a fallback for the E2E store encryption key when `/dev/urandom` failed, which made the key predictable. It also ignored errors when reading from `/dev/urandom`.
**Learning:** Security-critical cryptography, like a master key/passphrase generation, should never fail-open or fall back to an insecure entropy source like `SystemTime`. If sufficient entropy is unavailable, the application must abort securely.
**Prevention:** Always propagate errors for secure operations (e.g., using `read_exact` and returning `Result`) rather than swallowing errors and employing a weak fallback.
