# Design Document: Login Implementation

**Status**: Approved
**Date**: 2026-04-04
**Complexity**: Medium

## 1. Problem Statement
The Claw Matrix client lacks a user interface for authentication. While the backend supports password-based login and session restoration, users cannot currently enter credentials if a session is not already present.

## 2. Proposed Solution
Implement a state-based view switcher in the main application. If no authenticated user is detected, the application will present a login form. Upon successful authentication, the app will transition to the main functional view and persist the session.

## 3. Architectural Approach
- **View Switching**: Conditional rendering in `Claw::view()` based on `self.user_id`.
- **State Management**: New fields in `Claw` to track form inputs and authentication status.
- **Backend Integration**: Utilize existing `MatrixEngine::login` which handles `matrix-sdk` authentication and `oo7` keyring persistence.

## 4. Component Design
- **LoginView**: A vertical layout with:
    - Homeserver input (defaulting to `https://matrix.org`)
    - Username input
    - Password input (masked)
    - Login button (with loading indicator)
    - Error display area

## 5. User Flow
1. App Start -> `restore_session()`
2. `user_id` is `None` -> Render `LoginView`
3. User enters credentials -> Click "Login"
4. `SubmitLogin` message -> `MatrixEngine::login()`
5. Success -> `user_id` populated -> Render `MainView` -> `start_sync()`
6. Failure -> Display error in `LoginView`

## 6. Security Considerations
- Use masked password inputs.
- Rely on `oo7` (Secret Service/Keyring) for secure token storage.
