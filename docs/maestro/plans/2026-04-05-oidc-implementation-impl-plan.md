---
title: "OIDC Implementation Implementation Plan"
design_ref: "docs/maestro/plans/2026-04-05-oidc-implementation-design.md"
created: "2026-04-05T10:30:00Z"
status: "draft"
total_phases: 7
estimated_files: 6
task_complexity: "complex"
---

# OIDC Implementation Implementation Plan

## Plan Overview

- **Total phases**: 7
- **Agents involved**: coder, security_engineer, tester, technical_writer
- **Estimated effort**: Complex. Involves inter-instance communication, security-critical protocol implementation, and UI integration.

## Dependency Graph

```
Phase 1 (IPC) ----+
                  |
Phase 2 (Engine) -+--> Phase 4 (UI) --> Phase 5 (Security) --> Phase 7 (Docs)
       |          |                      ^
       +--> Phase 3 (Refresh) -----------+
                  |                      |
                  +--> Phase 6 (Testing) -+
```

## Execution Strategy

| Stage | Phases | Execution | Agent Count | Notes |
|-------|--------|-----------|-------------|-------|
| 1     | Phase 1, Phase 2 | Parallel | 2 | Foundation & Core Engine |
| 2     | Phase 3, Phase 4 | Parallel | 2 | Infrastructure & UI |
| 3     | Phase 5, Phase 6 | Parallel | 2 | Quality & Validation |
| 4     | Phase 7 | Sequential | 1 | Documentation |

---

## Phase 1: Custom URI & D-Bus Foundation

### Objective
Register the custom URI scheme and implement a D-Bus listener to receive OIDC callbacks from secondary instances.

### Agent: coder
### Parallel: Yes

### Files to Create

- `src/ipc.rs` — Implements the D-Bus server/client for inter-instance communication.

### Files to Modify

- `src/main.rs` — Initialize the IPC listener and handle incoming callback URIs.
- `Cargo.toml` — Add `zbus` dependency for D-Bus communication.

### Implementation Details
- **URI Scheme**: `com.system76.Claw://callback`.
- **D-Bus Interface**: `com.system76.Claw.Ipc` with a `HandleCallback(uri: String)` method.
- **Single Instance Logic**: On startup, check if a D-Bus service is already active. If yes, send the URI argument to it and exit. If no, start the D-Bus service and proceed as the primary instance.

### Validation
- Run `cargo check`.
- Manually launch the app with a `com.system76.Claw://callback?code=test` argument and verify the primary instance receives it.

### Dependencies
- Blocked by: None
- Blocks: Phase 4

---

## Phase 2: Matrix Engine OIDC Core

### Objective
Implement the core OIDC login logic in `MatrixEngine` using `matrix-sdk`.

### Agent: coder
### Parallel: Yes

### Files to Modify

- `src/matrix/mod.rs` — Implement `login_oidc` and `complete_oidc_login`.

### Implementation Details
- `login_oidc(homeserver: &str)`: Uses `client.matrix_auth().oidc().login_with_redirect(callback_url)` to initiate the flow.
- `complete_oidc_login(callback_url: Url)`: Uses the `OidcLogin` flow returned from initiation to complete the exchange.
- **Error Handling**: Map `matrix-sdk` OIDC errors to the application's internal `Error` type.

### Validation
- Run `cargo check`.
- Unit tests in `src/matrix/mod.rs` (mocking OIDC responses if possible).

### Dependencies
- Blocked by: None
- Blocks: Phase 3, Phase 4, Phase 6

---

## Phase 3: Token Management & Auto-Refresh

### Objective
Extend `SessionData` to include OIDC tokens and configure automatic background token refreshment.

### Agent: coder
### Parallel: Yes

### Files to Modify

- `src/matrix/mod.rs` — Update `SessionData` and implement `set_refresh_handler`.

### Implementation Details
- **SessionData Update**: Add `id_token` (String) and ensure `refresh_token` is correctly populated from the OIDC response.
- **Refresh Handler**: Use `client.matrix_auth().set_refresh_handler` to update the `oo7` keyring whenever a new token is obtained.
- **Serialization**: Ensure `SessionData` is correctly serialized to JSON for storage in `oo7`.

### Validation
- Verify `SessionData` serialization with new fields.
- Integration test for token persistence in a temporary keyring.

### Dependencies
- Blocked by: Phase 2
- Blocks: Phase 5, Phase 6

---

## Phase 4: UI Integration & Login Flow

### Objective
Update the `libcosmic` UI to support OIDC login and handle the callback event.

### Agent: coder
### Parallel: Yes

### Files to Modify

- `src/main.rs` — Add OIDC button, handle `Message::OidcCallback`, and update login state.

### Implementation Details
- **UI Update**: Add a "Login with OIDC" button to the `LoginView`.
- **Message Handling**: Add `Message::OidcCallback(Url)` and `Message::OidcError(String)`.
- **Flow**: User clicks OIDC -> Engine launches browser -> OS redirects -> IPC receives URI -> UI updates to "Logging in..." -> Engine completes flow.

### Validation
- Manual UI test: Verify the OIDC button launches the browser.
- Verify UI updates correctly upon receiving the callback.

### Dependencies
- Blocked by: Phase 1, Phase 2
- Blocks: Phase 5, Phase 6

---

## Phase 5: Security Audit

### Agent: security_engineer
### Parallel: Yes

### Objective
Perform a deep security review of the OIDC implementation.

### Implementation Details
- Review PKCE implementation and `state` parameter validation.
- Audit `oo7` keyring integration for potential leaks.
- Verify CSRF protection in the IPC handler.
- Check for insecure token logging.

### Validation
- Security findings report with no CRITICAL or MAJOR issues remaining.

### Dependencies
- Blocked by: Phase 3, Phase 4
- Blocks: Phase 7

---

## Phase 6: Testing & Validation

### Agent: tester
### Parallel: Yes

### Objective
Implement comprehensive tests for the OIDC flow.

### Files to Create
- `tests/oidc_integration.rs` — Integration tests for the full OIDC lifecycle.

### Files to Modify
- `src/matrix/tests.rs` — Add unit tests for OIDC discovery and token handling.

### Validation
- Run `cargo test`.
- All OIDC-related tests must pass.

### Dependencies

- Blocked by: Phase 3, Phase 4
- Blocks: Phase 5

---

## Phase 7: Documentation & Polish

### Agent: technical_writer
### Parallel: No

### Objective
Update project documentation to include OIDC setup and technical details.

### Files to Modify
- `SPEC.md` — Reflect the new authentication architecture.
- `README.md` (if exists) — Instructions for OIDC login.

### Validation
- Review documentation for accuracy and completeness.

### Dependencies
- Blocked by: Phase 5
- Blocks: None

---

## File Inventory

| # | File | Phase | Purpose |
|---|------|-------|---------|
| 1 | `src/ipc.rs` | 1 | Inter-instance communication for OIDC callback |
| 2 | `src/matrix/mod.rs` | 2, 3 | Core OIDC logic, SessionData, and Refresh handler |
| 3 | `src/main.rs` | 1, 4 | UI integration and IPC listener |
| 4 | `Cargo.toml` | 1 | Dependencies (zbus, etc.) |
| 5 | `tests/oidc_integration.rs` | 6 | Full lifecycle integration tests |
| 6 | `SPEC.md` | 7 | Technical documentation |

## Risk Classification

| Phase | Risk | Rationale |
|-------|------|-----------|
| 1 | MEDIUM | Inter-instance communication can be tricky with D-Bus permissions. |
| 2 | HIGH | Core security protocol implementation; critical for login. |
| 3 | MEDIUM | Background refresh logic must be robust to prevent session loss. |
| 4 | LOW | Standard UI updates. |
| 5 | HIGH | Security review is critical for auth features. |
| 6 | MEDIUM | Mocking OIDC for tests can be complex. |
| 7 | LOW | Documentation only. |

## Execution Profile

```
Execution Profile:
- Total phases: 7
- Parallelizable phases: 6 (in 3 batches)
- Sequential-only phases: 1
- Estimated parallel wall time: ~4-6 turns per batch
- Estimated sequential wall time: ~30-40 turns total

Note: Native subagents currently run without user approval gates.
All tool calls are auto-approved without user confirmation.
```
