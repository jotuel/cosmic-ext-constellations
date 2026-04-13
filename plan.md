1.  **Refactor settings UI components to use semantic widgets:** Update various buttons across settings (room, space, user) to utilize libcosmic's semantic and accessible UI components instead of generic text buttons.
    -   In `src/settings/user.rs`, update the rename button ("✏️") to use `button::icon` with the "document-edit-symbolic" icon to provide better visual integration and context.
    -   In `src/settings/user.rs`, `src/settings/room.rs`, and `src/settings/space.rs`, update destructive actions (like "Delete", "Kick", "Ban", "Leave Room", "Forget Room", "Remove") from standard `button::text` to `button::destructive`. This semantic change provides correct visual affordances for dangerous actions and improves the accessibility.
    -   Ensure the rename button is wrapped in a tooltip for accessible context.

2.  **Add a critical UX learning to the Palette journal:** Since the instruction is to act as "Palette" and track *critical* learnings, I will log the learning about semantic button usage (icon buttons lacking text need tooltips, destructive actions need proper affordances) in `.jules/palette.md`.

3.  **Run pre-commit checks:** Complete pre commit steps to make sure proper testing, verifications, reviews and reflections are done.
4.  **Submit the changes:** Create a commit matching Palette's preferred PR format.
