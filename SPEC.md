## System Architecture
The application follows a decoupled "Engine and Shell" model to bridge the asynchronous nature of the Matrix protocol with the reactive UI of COSMIC.Core ComponentsThe SDK Engine: Handles matrix-sdk-ui services (Sliding Sync, Timeline).The Persistence Layer: SQLite for local state and E2EE key storage.The Async Bridge: A background task that converts SDK Stream events into COSMIC Message variants.The UI Shell: libcosmic widgets organized into a Navigation Sidepane (Room List) and a Detail View (Timeline + Composer).
## Technical Stack
CategoryCrate / ToolPurposeMatrix Logicmatrix-sdk-uiHigh-level RoomListService and Timeline APIs.State Storematrix-sdk-sqliteLocal database for messages and encryption keys.UI ToolkitlibcosmicCOSMIC-native widgets and application lifecycle.Editorcosmic-editMulti-line text buffer for the message composer.Markdownpulldown-cmarkParsing raw input for the preview and HTML generation.Keyringoo7Secure storage of the user's access token via Secret Service.
## Implementation 
### Phases 
1. The Matrix Sync & AuthInitialize the Client with a local SQLite store.Implement login flow (Password or OIDC) and store the session token using oo7.Start the SyncService (Sliding Sync) in a background tokio task with automatic recovery (see Synchronization & Recovery).
2. The Room List (Sidebar)Subscribe to the RoomListService.Map RoomListDiff events into your application's update() function.Render a cosmic::widget::Nav list where each entry shows the room name and last message snippet. 
3. The Timeline (Chat View)Upon selecting a room, initialize a Timeline object from the SDK.Use iced::Subscription::from_stream to pipe timeline changes (new messages, edits, reactions) into the UI.Implement a virtualized VerticalList to render message bubbles.
4. The Markdown ComposerState Management: Track input_text: String and is_preview: bool.Toggle Logic: Use a Segmented Button to switch between cosmic-edit and the PreviewArea.The PreviewArea: A function that takes the raw string, parses it with pulldown-cmark, and returns a Column of text widgets styled for Bold, Italic, and Code.The Send Action: * Plaintext: Send the raw buffer.Formatted: Generate HTML via pulldown-cmark and send as formatted_body.

### Synchronization & Recovery
To ensure a resilient connection, the background synchronization task implements an automatic retry loop with exponential backoff and integrated error reporting:
- **Retry Mechanism**: If the `SyncService` stops due to a transient error, the engine automatically attempts to restart it.
- **Error Reporting**: Sync failures are captured and propagated to the UI via the `SyncStatus::Error(String)` variant. The UI displays a descriptive error message in the status bar to inform the user of the failure.
- **Diagnostic Details**: Due to current limitations in `matrix-sdk-ui` (v0.7.0) regarding specific error payload exposure on the `State` enum, a descriptive fallback is used: *"Sync error encountered. This may be due to missing server support for Sliding Sync (MSC4186) or network issues."*
- **Exponential Backoff**: Retries use a doubling delay starting at 2 seconds (2s, 4s, 8s, 16s, 32s), capped at a maximum of 60 seconds. The error state persists in the UI until a retry attempt succeeds.
- **Backoff Reset**: The retry delay is reset to the initial 2 seconds if the service maintains a stable connection for at least 30 seconds, at which point the error indicator is cleared from the status bar.

### Data Flow Logic: The "Bridge"
To keep the UI responsive, all Matrix API calls must be handled via Commands.UI $\to$ SDK: The user clicks "Send" $\to$ Message::SendClicked $\to$ Command::perform(client.send(...)).SDK $\to$ UI: New message arrives in sync loop $\to$ Subscription yields Message::NewEvent $\to$ view() updates.
5. Security & Performance ConstraintsE2EE Persistence: Never store cross-signing keys in plain text; let matrix-sdk-sqlite handle the encryption of the store itself.Lazy Loading: Do not fetch full message history on room entry. Use the SDK's back-pagination tokens only when the user scrolls to the top of the timeline.Image Handling: Decrypt media in a separate tokio thread pool to prevent the COSMIC UI thread from locking during high-resolution image processing.
