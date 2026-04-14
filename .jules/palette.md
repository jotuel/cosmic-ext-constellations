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
