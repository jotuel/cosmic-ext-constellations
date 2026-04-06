## System Architecture
The application follows a decoupled "Engine and Shell" model to bridge the asynchronous nature of the Matrix protocol with the reactive UI of COSMIC.

### Core Components
- **The SDK Engine**: Handles matrix-sdk-ui services (Sliding Sync, Timeline).
- **The Persistence Layer**: SQLite for local state and E2EE key storage.
- **The Async Bridge**: A background task that converts SDK Stream events into COSMIC Message variants.
- **The UI Shell**: libcosmic widgets organized into a Navigation Sidepane (Room List) and a Detail View (Timeline + Composer).
- **The IPC Interface**: A D-Bus service for handling OIDC callbacks and single-instance enforcement.

## Technical Stack
| Category | Crate / Tool | Purpose |
| --- | --- | --- |
| Matrix Logic | matrix-sdk-ui | High-level RoomListService and Timeline APIs. |
| State Store | matrix-sdk-sqlite | Local database for messages and encryption keys. |
| UI Toolkit | libcosmic | COSMIC-native widgets and application lifecycle. |
| Editor | cosmic-edit | Multi-line text buffer for the message composer. |
| Markdown | pulldown-cmark | Parsing raw input for the preview and HTML generation. |
| Keyring | oo7 | Secure storage of the user's access token and store passphrase via Secret Service. |
| IPC | zbus | D-Bus communication for OIDC callbacks. |

## Implementation 
### Phases 
1. **The Matrix Sync & Auth**
   - Initialize the Client with a local SQLite store.
   - Implement login flow (Password or OIDC).
   - Store the session token and SQLite store passphrase using oo7.
   - Start the SyncService (Sliding Sync) in a background tokio task with automatic recovery.

2. **The Room List (Sidebar)**
   - Subscribe to the RoomListService.
   - Map RoomListDiff events into your application's update() function.
   - Render a cosmic::widget::Nav list where each entry shows the room name and last message snippet. 

3. **The Timeline (Chat View)**
   - Upon selecting a room, initialize a Timeline object from the SDK.
   - Use iced::Subscription::from_stream to pipe timeline changes into the UI.
   - Implement a virtualized VerticalList to render message bubbles.

4. **The Markdown Composer**
   - State Management: Track input_text: String and is_preview: bool.
   - Toggle Logic: Use a Segmented Button to switch between cosmic-edit and the PreviewArea.
   - The PreviewArea: A function that takes the raw string, parses it with pulldown-cmark, and returns a Column of text widgets.
   - The Send Action: Generate HTML via pulldown-cmark and send as formatted_body.

### OIDC Authentication
Claw supports modern OIDC-based authentication. The flow is as follows:
1. The user enters their homeserver.
2. Claw requests an OIDC login URL from the homeserver.
3. Claw opens the URL in the user's default browser.
4. After successful login, the browser redirects to `com.system76.Claw://callback?code=...&state=...`.
5. The `cosmic-ext-claw` binary is invoked with the URI.
6. If an instance is already running, the URI is sent via D-Bus to the existing instance's `handle_callback` method.
7. The existing instance completes the login flow using the provided code and state.

### Synchronization & Recovery
To ensure a resilient connection, the background synchronization task implements an automatic retry loop with exponential backoff:
- **Retry Mechanism**: If the `SyncService` stops, the engine automatically attempts to restart it.
- **Exponential Backoff**: Retries use a doubling delay starting at 2 seconds, capped at 60 seconds.
- **Backoff Reset**: The retry delay is reset if the service maintains a stable connection for at least 30 seconds.

### Security Considerations
- **URI Validation**: The IPC interface validates that all incoming callback URIs start with the expected prefix (`com.system76.Claw://callback`) to prevent URI injection attacks.
- **Store Encryption**: The SQLite state store is encrypted using a unique passphrase generated on first run and stored securely in the user's keyring via `oo7`.
- **Credential Storage**: Matrix session tokens (access and refresh tokens) are stored in the system keyring using `oo7`.
- **Log Redaction**: Sensitive OIDC parameters (`code` and `state`) are redacted from application logs to prevent accidental leakage of authentication tokens.
- **E2EE Persistence**: Encryption keys are managed by `matrix-sdk-sqlite` and stored in the encrypted SQLite database.
