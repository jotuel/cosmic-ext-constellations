---
name: testing
description: Unit testing guidelines, state machine verification, and troubleshooting isolated compilation packages.
---

# COSMIC/Rust Testing Best Practices

This skill outlines guidelines and setup strategies for writing unit tests and verifying state machine updates in libcosmic/iced Rust applications.

## 1. State Machine Verification
* **Regression-Free State Updates**: When adding tests to state machines, focus on verifying that each `update()` variant transitions the state properly and produces the expected side-effect `Task`/`Action` commands.
* **Update Verification Loop**: For every message update flow, assert both the final state variables and the returned command structure.

## 2. Dependencies & Build Isolation
* **Managing Compilation Failures in Testing**: When parent project dependencies encounter severe compiler or recursion limits (e.g. `matrix-sdk` crate depth boundaries), unit tests can fail to compile.
  - *Action:* Setup isolated sandbox test packages (or dummy cargo modules) to establish sound testing baselines before integrating with complex upstream crates. This isolates the testing environment from unrelated compilation issues.
