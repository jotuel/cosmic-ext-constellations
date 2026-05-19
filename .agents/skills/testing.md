## 2026-04-20 - App Settings Testing
**Learning:** Adding tests to state machines requires verifying that each `update()` variant correctly changes state without any regressions, and handling isolated compilation environments properly when the main project faces dependency compiler issues (e.g., matrix_sdk recursion limits).
**Action:** Always create isolated dummy test packages to establish testing baselines if upstream code cannot compile, ensuring your unit tests are sound before integrating.
