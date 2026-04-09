## 2024-04-08 - Adding Helpful Empty States
**Learning:** I learned that an empty main content area when no room is selected leaves users confused about what to do next. Empty states are an important part of the UX to help guide users on how to use the interface.
**Action:** Implemented a friendly and informative empty state in the main content area when no room is selected to improve the overall UX.
## 2023-10-27 - Tooltips for Icon-Only Buttons
**Learning:** Icon-only buttons (like Space Switcher avatars/initials) lack context. Adding tooltips makes the app significantly more accessible and intuitive without cluttering the UI.
**Action:** When creating a list or grid of generic or custom icons, wrap them in `cosmic::widget::tooltip` so users know what they represent.
