## 2026-04-10 - Missing ARIA Labels for Icon-Only Buttons
**Learning:** Icon-only buttons (like the device rename '✏️' button) lack accessible text for screen readers and tooltips for mouse users, reducing accessibility and clarity.
**Action:** Always wrap icon-only buttons with `cosmic::widget::tooltip` providing a descriptive `text::body` and position (e.g., `Position::Top`) so users can understand their purpose.
## 2024-04-08 - Adding Helpful Empty States
**Learning:** I learned that an empty main content area when no room is selected leaves users confused about what to do next. Empty states are an important part of the UX to help guide users on how to use the interface.
**Action:** Implemented a friendly and informative empty state in the main content area when no room is selected to improve the overall UX.
## 2023-10-27 - Tooltips for Icon-Only Buttons
**Learning:** Icon-only buttons (like Space Switcher avatars/initials) lack context. Adding tooltips makes the app significantly more accessible and intuitive without cluttering the UI.
**Action:** When creating a list or grid of generic or custom icons, wrap them in `cosmic::widget::tooltip` so users know what they represent.
## 2026-04-12 - Semantic Widget Choices & Destructive Buttons
**Learning:** Using generic text buttons (`button::text`) for destructive actions like delete, leave, or ban fails to warn the user visually, leading to potential misclicks or anxiety. Additionally, icon-only buttons ("✏️") lack accessibility context.
**Action:** Always use libcosmic semantic widgets: `button::destructive` for irreversible or dangerous operations, and wrap any `button::icon` lacking descriptive text inside a `cosmic::widget::tooltip` to ensure clarity and accessibility for all users.
## 2024-05-18 - Replacing Emojis with Symbolic Icons
**Learning:** Using raw emojis for UI actions (like '✏️' or '🗑️') can break visual consistency with the system theme and lacks accessible tooltips if not explicitly wrapped.
**Action:** When designing UI elements in `libcosmic`, prefer semantic widgets over generic text buttons: use `button::icon` with standard freedesktop icons (e.g., `Named::new("document-edit-symbolic")`) and ensure they are wrapped in a `cosmic::widget::tooltip` to provide correct visual affordances and accessibility.
## 2026-04-14 - Disabled States for Primary Actions
**Learning:** Users can accidentally trigger empty or invalid actions (like sending an empty message or creating an unnamed room) if primary action buttons remain enabled when input is missing. This leads to phantom submissions and user confusion.
**Action:** Always disable primary action buttons (like 'Send' or 'Create') and block form submission when the required input fields are empty or contain only whitespace to provide immediate visual feedback and prevent invalid states.
## 2024-04-15 - Replace Text Buttons with Semantic Icons
**Learning:** Found that the "All Rooms" switch in the space switcher uses a text button containing a globe emoji (`"🌐"`). `libcosmic` specifies that semantic widgets should be preferred over generic text buttons with emojis. Using `button::icon` wrapped in `cosmic::widget::tooltip` provides better visual affordance, native styling consistency with the COSMIC desktop environment, and improved accessibility.
**Action:** Replaced the `"🌐"` text button with `button::icon(Named::new("applications-internet-symbolic"))` or `"network-workgroup-symbolic"` to align with the `libcosmic` UI guidelines.
## 2026-05-20 - Tooltips for Disabled Actions
**Learning:** While disabling primary actions (like 'Send' or 'Login') prevents invalid states and phantom submissions, it can leave users confused as to *why* the button is unresponsive if the requirements aren't visually obvious.
**Action:** When conditionally disabling primary action buttons, wrap them in a `cosmic::widget::tooltip` explaining the required state (e.g., "Type a message to send" or "Fill in all fields to login").
## 2026-05-22 - Missing Disabled State on Add Child
**Learning:** The "Add Child" button in the space settings did not have a disabled state when the input field was empty, violating our principles for primary actions.
**Action:** Always conditionally disable primary action buttons like "Add Child" and wrap them in a `cosmic::widget::tooltip` explaining the required state (e.g., "Enter a room or space ID to add").
## 2024-05-25 - Disabled States for Settings Actions
**Learning:** Users can encounter errors or be confused when "Change Password", "Invite", "Mod", or "Admin" buttons in the settings stay enabled despite the required input fields being empty or invalid (e.g., mismatched passwords).
**Action:** Disabled the primary action buttons in `src/settings/user.rs` and `src/settings/room.rs` when inputs are empty or invalid, and wrapped them in a `cosmic::widget::tooltip` explaining the missing required state.
## 2026-04-19 - Tooltips for Disabled Save Buttons
**Learning:** Users can easily become confused when 'Save' or 'Save Changes' buttons are disabled without context. The previous UI did not explicitly communicate that unsaved modifications are required to enable these action buttons.
**Action:** When conditionally disabling save buttons based on pending changes, wrap the disabled button in a `cosmic::widget::tooltip` (e.g., explaining "Make changes to save") to proactively guide the user on why the button is inactive.
## 2026-05-30 - Tooltips for Icon-Only Buttons
**Learning:** Icon-only buttons (like `button::icon` or `button::custom` wrapping icons) are frequently used across the application for actions like search, opening menus, creating spaces, and reacting. However, without text labels, they lack context for screen readers and new users.
**Action:** When creating or modifying icon-only buttons in `libcosmic`, always ensure they are wrapped in a `cosmic::widget::tooltip` with a descriptive `text::body` (e.g., "Search", "User Menu", "Add Reaction") and an appropriate `Position` (like `Position::Bottom` or `Position::Top`) to guarantee accessibility and immediate clarity.
