# Implementation Plan: claw - Phase 1 & Foundation

## 1. Objective
Establish the project foundation for 'claw' and implement Phase 1: Matrix Sync & Auth.

## 2. Key Files & Context
- `SPEC.md`: Technical specification.
- `.agents/skills/COSMIC-DEVELOPMENT.md`: Application design guidelines.
- `Cargo.toml`: Manifest.
- `src/main.rs`: Entrypoint.
- `src/matrix/mod.rs`: Matrix engine.

## 3. Implementation Steps

### Phase 1: Project Initialization & Foundation
- **Tasks**:
  1. Initialize the Rust project (`cargo init`).
  2. Configure `Cargo.toml` with dependencies: `libcosmic`, `matrix-sdk-ui`, `matrix-sdk-sqlite`, `tokio`, `oo7`.
  3. Set up the basic `libcosmic` application shell in `src/main.rs`.
- **Agent**: `coder`
- **Validation**: `cargo check`

### Phase 2: Matrix Engine & Auth Logic
- **Tasks**:
  1. Implement `src/matrix/mod.rs` to initialize the `matrix-sdk` Client with SQLite persistence.
  2. Implement login flow (Password-based for Phase 1) using the SDK.
  3. Implement session storage using `oo7`.
- **Agent**: `coder`
- **Validation**: `cargo test`

### Phase 3: Matrix Sync & Shell Integration
- **Tasks**:
  1. Implement the background `tokio` task to start `SyncService`.
  2. Bridge the `SyncService` events into the `libcosmic` update loop using `iced::Subscription`.
- **Agent**: `coder`
- **Validation**: `cargo build`

## 4. Verification & Testing
- **Unit Tests**: Test client initialization and mock authentication in `src/matrix/tests.rs`.
- **Build**: Ensure the project builds without errors.
- **Run**: Verify that the application launches and attempts to sync with Matrix (simulated or real).

Approve this implementation plan before execution begins?

1. Approve plan
2. Revise plan
3. Abort execution