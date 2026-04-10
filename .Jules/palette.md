## 2026-04-10 - Missing ARIA Labels for Icon-Only Buttons
**Learning:** Icon-only buttons (like the device rename '✏️' button) lack accessible text for screen readers and tooltips for mouse users, reducing accessibility and clarity.
**Action:** Always wrap icon-only buttons with `cosmic::widget::tooltip` providing a descriptive `text::body` and position (e.g., `Position::Top`) so users can understand their purpose.
