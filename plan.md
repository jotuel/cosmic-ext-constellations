1. **Identify the UX improvements**:
   - The device management page uses standard text buttons with emojis instead of proper libcosmic icon buttons for actions (renaming and deleting devices). This is specifically called out in Palette's guidelines: "buttons with icons look better than text with symbols etc."
   - Rename `button::text("✏️")` to `button::icon(cosmic::widget::icon::Named::new("document-edit-symbolic"))` (wrapped in tooltip).
   - Change the delete button to use an icon button, wrapped in a tooltip.
   - The user avatar placeholder in the timeline `text::body("👤")` should use a libcosmic icon: `cosmic::widget::icon::from_name("avatar-default-symbolic")`.

Let's do this via `replace_with_git_merge_diff`.
