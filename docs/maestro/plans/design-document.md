# Design Document: claw - Phase 1 & Foundation

## 1. Objective
Establish the project foundation for 'claw', a COSMIC-native Matrix client, and implement Phase 1: Matrix Sync & Auth.

## 2. Key Files & Context
- `SPEC.md`: The primary design specification for the application architecture.
- `.agents/skills/COSMIC-DEVELOPMENT.md`: Specialized guidelines for COSMIC development.
- `Cargo.toml`: Project configuration and dependencies.
- `src/main.rs`: Application entry point and libcosmic lifecycle.
- `src/matrix/mod.rs`: Matrix SDK integration layer.

## 3. Scope & Boundary
- **In Scope**:
  - Rust project initialization (`cargo init`).
  - Core dependencies setup (`libcosmic`, `matrix-sdk-ui`, `matrix-sdk-sqlite`, `tokio`).
  - `matrix-sdk` client initialization with SQLite persistence.
  - Phase 1 Authentication flow (Username/Password or OIDC logic placeholders).
  - libcosmic application shell setup with basic `update()` loop.
  - Unit tests for client initialization.
- **Out of Scope**:
  - Room List (Phase 2), Timeline (Phase 3), and Composer (Phase 4).
  - Persistent credential storage via `oo7` (deferred to implementation detail of Auth).

## 4. Proposed Solution
The application will follow a decoupled "Engine and Shell" model.
- **Engine**: A background `tokio` task managing the `matrix-sdk` client and sync service.
- **Shell**: A `libcosmic` application that communicates with the Engine via an `iced::Subscription` or message passing.
- **Auth**: A dedicated module to handle Matrix login and session management, initially targeting standard Password/OIDC.

## 5. Trade-offs & Alternatives
- **Alternative**: Implementing all 4 phases at once. **Rejected** due to complexity and the need for a stable foundation first.
- **Alternative**: Mocked UI first. **Rejected** because Matrix SDK integration is the core technical challenge for this project.

## 6. Verification
- `cargo test`: Verify client initialization and mock auth flows.
- `cargo run`: Launch the libcosmic shell with a basic status indicator for Matrix sync.

Approve this design before I move to implementation planning?

1. Approve design
2. Revise design