# ***Bugs***

[x] Regression session not persisting between runs

[ ] After Logout can't Login again

# ***Messaging***

[ ] Rich Media Support: Sending images etc.

[ ] Adding Emoji reactions

[ ] Threaded conversations

[ ] Automatic sliding sync / pagination

[ ] Option to ignore user

# ***Notifications***

[ ] Plug in cosmic-notifications

[ ] Investigate OSD notifications

# ***Settings***
## **Application**

[ ] Settings for Notifications

[ ] Set up Config

[ ] Global Notification Rules: The NotificationSettings API in the SDK allows you to
 configure whether the user is notified for specific events (e.g., mentions, direct
 messages, room messages, encrypted messages).
## **Room**

### *1. General Profile & Appearance*

[ ] Room Name & Topic: Read and update via room.name() / room.topic() and their respective state setters (room.set_name(), room.set_room_topic()).

[ ] Room Avatar: Fetch the avatar using room.avatar_url() and upload a new one by sending an m.room.avatar state event.

### *2. Notification Settings*

[ ] Push Rules & Muting: The SDK has a dedicated client.notification_settings() API. You can allow users to set the room to:
    - All Messages: Notify for every message.
    - Mentions & Keywords Only: Only notify if the user is pinged.
    - Mute: Completely suppress notifications for the room.

### *3. Membership & Access Control*

[ ] Leave / Forget: Add buttons for room.leave() and room.forget() (once left).

[ ] Manage Members: List participants (room.members()) and provide actions to:
    - Invite (room.invite_user_by_id())
    - Kick (room.kick_user())
    - Ban / Unban (room.ban_user(), room.unban_user())

[ ] Join Rules: Modify the m.room.join_rules state event to toggle whether the room is Invite Only, Public, Knock (request to join), or Restricted (requires membership in a
specific Space).

### *4. Security & Encryption*

[ ] Enable E2E Encryption: If the room is unencrypted, you can add a button to call room.enable_encryption(). (Note: Make sure to warn the user this is a one-way action and
cannot be undone).

[ ] History Visibility: Change the m.room.history_visibility state event to determine if new members can see past messages (Shared, Invited, or Joined).

### *5. Role & Permission Management*

[ ] Power Levels: You can fetch the m.room.power_levels event and build a UI to promote/demote users (e.g., Default (0), Moderator (50), Admin (100)).

[ ] Action Permissions: Use the power levels payload to let Admins restrict who can change the room name, invite new people, or send certain types of messages.

### *6. Advanced / Debug*

[ ] Room ID & Aliases: Display the internal Matrix !room_id:server.com and manage its canonical aliases (#alias:server.com).

[ ] Pinned Messages: Interface with the m.room.pinned_events state event to manage important messages.
## **Space**
## **User**
### *1. Profile Management (client.account())*

[x] Display Name: Read and update the user's global display name (get_display_name, set_display_name).

[x] Avatar: Upload and change the user's global profile picture (get_avatar_url, upload_avatar, set_avatar_url).

### *2. Account & Security (client.account() / client.devices())*

[x] Session / Device Management: List all active logins, rename them, or log them out remotely (client.devices(), client.rename_device(), client.delete_devices()).

[x] Change Password: Allow the user to update their account password (change_password()).

[ ] Emails and Phone Numbers (3PIDs): View, add, and remove linked emails or phone numbers (get_3pids(), add_3pid(), delete_3pid()).

[ ] Deactivate Account: Allow users to permanently delete their matrix account (deactivate()).

### *3. Notifications & Push Rules (client.notification_settings())*

[ ] Keywords: Manage custom keywords the user wants to be notified for.

### *4. Privacy & Preferences (client.account())*

[ ] Ignored Users: View and manage the list of blocked/ignored users (ignore_user(), unignore_user(), is_user_ignored()).

[ ] Media & URL Previews: Toggle global policies for displaying media previews and
 invite avatars (set_media_previews_display_policy(),
 set_invite_avatars_display_policy()).

### *5. Cross-Signing & Encryption (client.encryption())*

[ ] Key Verification: Check cross-signing status, bootstrap cross-signing, or display the user's master/self-signing keys.

[x] Session Verification: Trigger or accept interactive verification requests for new devices.
