# Messaging

[ ] Rich Media Support: Sending images etc.

[ ] Adding Emoji reactions

[ ] Threaded conversations

[ ] Automatic sliding sync / pagination

[ ] Option to ignore user

# Notifications

[ ] Plug in cosmic-notifications

[ ] Investigate OSD notifications

[ ] Settings for Notifications

# Settings
## Application
[ ] Set up Config
## Room
## Space
## User
### 1. Profile Management (client.account())

[x] Display Name: Read and update the user's global display name (get_display_name, set_display_name).

[x] Avatar: Upload and change the user's global profile picture (get_avatar_url, upload_avatar, set_avatar_url).

### 2. Account & Security (client.account() / client.devices())

[ ] Session / Device Management: List all active logins, rename them, or log them out remotely (client.devices(), client.rename_device(), client.delete_devices()).

[x] Change Password: Allow the user to update their account password (change_password()).

[ ] Emails and Phone Numbers (3PIDs): View, add, and remove linked emails or phone numbers (get_3pids(), add_3pid(), delete_3pid()).

[ ] Deactivate Account: Allow users to permanently delete their matrix account (deactivate()).

### 3. Notifications & Push Rules (client.notification_settings())

[ ] Global Notification Rules: The NotificationSettings API in the SDK allows you to
 configure whether the user is notified for specific events (e.g., mentions, direct
 messages, room messages, encrypted messages).

[ ] Keywords: Manage custom keywords the user wants to be notified for.

### 4. Privacy & Preferences (client.account())

[ ] Ignored Users: View and manage the list of blocked/ignored users (ignore_user(), unignore_user(), is_user_ignored()).
 - Media & URL Previews: Toggle global policies for displaying media previews and
 invite avatars (set_media_previews_display_policy(),
 set_invite_avatars_display_policy()).

### 5. Cross-Signing & Encryption (client.encryption())

[ ] Key Verification: Check cross-signing status, bootstrap cross-signing, or display the user's master/self-signing keys.

[x] Session Verification: Trigger or accept interactive verification requests for new devices.
