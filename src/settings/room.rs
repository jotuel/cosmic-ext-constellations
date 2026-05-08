use crate::matrix::MatrixEngine;
use cosmic::iced::Alignment;
use cosmic::widget::{Column, Row, button, text, text_input, tooltip, tooltip::Position};
use cosmic::{Action, Element, Task};
use matrix_sdk::ruma::RoomId;
use matrix_sdk::ruma::events::room::MediaSource;
use matrix_sdk::ruma::events::room::history_visibility::HistoryVisibility;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct State {
    pub room_id: Option<Arc<str>>,
    pub name: String,
    pub original_name: String,
    pub topic: String,
    pub original_topic: String,
    pub is_loading: bool,
    pub is_saving: bool,
    pub error: Option<String>,
    pub avatar_url: Option<String>,
    pub avatar_handle: Option<cosmic::iced::widget::image::Handle>,
    pub is_uploading_avatar: bool,
    pub is_loading_avatar: bool,
    pub membership: Option<matrix_sdk::RoomState>,
    pub power_levels: Option<(i64, HashMap<matrix_sdk::ruma::OwnedUserId, i64>)>,
    pub is_loading_power_levels: bool,
    pub updating_power_level_for: Option<String>,
    pub ban_level: i64,
    pub original_ban_level: i64,
    pub invite_level: i64,
    pub original_invite_level: i64,
    pub kick_level: i64,
    pub original_kick_level: i64,
    pub redact_level: i64,
    pub original_redact_level: i64,
    pub events_default_level: i64,
    pub original_events_default_level: i64,
    pub room_name_level: i64,
    pub original_room_name_level: i64,
    pub room_topic_level: i64,
    pub original_room_topic_level: i64,
    pub room_avatar_level: i64,
    pub original_room_avatar_level: i64,
    pub invite_level_str: String,
    pub kick_level_str: String,
    pub ban_level_str: String,
    pub redact_level_str: String,
    pub events_default_level_str: String,
    pub room_name_level_str: String,
    pub room_topic_level_str: String,
    pub room_avatar_level_str: String,
    pub invite_user_id: String,
    pub kick_user_id: String,
    pub ban_user_id: String,
    pub action_reason: String,
    pub current_user_id: Option<String>,
    pub my_power_level: i64,
    pub member_filter: String,
    pub notification_mode: Option<matrix_sdk::notification_settings::RoomNotificationMode>,
    pub is_loading_notifications: bool,
    pub join_rule: Option<matrix_sdk::ruma::events::room::join_rules::JoinRule>,
    pub history_visibility: Option<HistoryVisibility>,
    pub restricted_space_id: String,
    pub pinned_events: Vec<matrix_sdk::ruma::OwnedEventId>,
    pub pinned_event_id_input: String,
    pub is_encrypted: bool,
    pub canonical_alias: String,
    pub original_canonical_alias: String,
    pub alt_aliases: Vec<String>,
    pub original_alt_aliases: Vec<String>,
    pub new_alt_alias_input: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    LoadRoom(std::sync::Arc<str>),
    RoomLoaded(Result<RoomInfo, String>),
    NameChanged(String),
    TopicChanged(String),
    SaveRoom,
    RoomSaved(Result<(), String>),
    DismissError,
    AvatarMediaFetched(Result<Vec<u8>, String>),
    SelectAvatar,
    AvatarFileSelected(Option<std::path::PathBuf>),
    AvatarUploaded(Result<(), String>),
    LeaveRoom,
    RoomLeft(Result<(), String>),
    ForgetRoom,
    RoomForgotten(Result<(), String>),
    LoadPowerLevels,
    PowerLevelsLoaded(Result<PowerLevelInfo, String>),
    UpdatePowerLevel(String, i64),
    PowerLevelUpdated(String, Result<(), String>),
    BanLevelChanged(String),
    InviteLevelChanged(String),
    KickLevelChanged(String),
    RedactLevelChanged(String),
    EventsDefaultLevelChanged(String),
    RoomNameLevelChanged(String),
    RoomTopicLevelChanged(String),
    RoomAvatarLevelChanged(String),
    InviteUser,
    UserInvited(Result<(), String>),
    KickUser(String),
    UserKicked(String, Result<(), String>),
    BanUser(String),
    UserBanned(String, Result<(), String>),
    InviteUserIdChanged(String),
    ActionReasonChanged(String),
    MemberFilterChanged(String),
    NotificationModeChanged(matrix_sdk::notification_settings::RoomNotificationMode),
    NotificationModeSet(Result<(), String>),
    JoinRuleChanged(matrix_sdk::ruma::events::room::join_rules::JoinRule),
    HistoryVisibilityChanged(HistoryVisibility),
    RestrictedSpaceIdChanged(String),
    PinnedEventIdChanged(String),
    PinEvent,
    UnpinEvent(matrix_sdk::ruma::OwnedEventId),
    EnableEncryption,
    EncryptionEnabled(Result<(), String>),
    CanonicalAliasChanged(String),
    AltAliasAdded,
    AltAliasRemoved(String),
    NewAltAliasInputChanged(String),
}

#[derive(Debug, Clone)]
pub struct PowerLevelInfo {
    pub default_level: i64,
    pub users: HashMap<matrix_sdk::ruma::OwnedUserId, i64>,
    pub my_level: i64,
}

#[derive(Debug, Clone)]
pub struct RoomInfo {
    pub name: String,
    pub topic: String,
    pub avatar_url: Option<String>,
    pub membership: matrix_sdk::RoomState,
    pub ban_level: i64,
    pub invite_level: i64,
    pub kick_level: i64,
    pub redact_level: i64,
    pub events_default_level: i64,
    pub room_name_level: i64,
    pub room_topic_level: i64,
    pub room_avatar_level: i64,
    pub current_user_id: Option<String>,
    pub notification_mode: Option<matrix_sdk::notification_settings::RoomNotificationMode>,
    pub join_rule: Option<matrix_sdk::ruma::events::room::join_rules::JoinRule>,
    pub history_visibility: Option<HistoryVisibility>,
    pub pinned_events: Vec<matrix_sdk::ruma::OwnedEventId>,
    pub is_encrypted: bool,
    pub canonical_alias: Option<String>,
    pub alt_aliases: Vec<String>,
}

impl State {
    pub fn update(
        &mut self,
        message: Message,
        matrix: &Option<MatrixEngine>,
    ) -> Task<Action<crate::Message>> {
        match message {
            Message::LoadRoom(room_id) => {
                if let Some(matrix) = matrix {
                    self.room_id = Some(room_id.clone());
                    self.is_loading = true;
                    self.error = None;

                    let engine = matrix.clone();
                    Task::perform(
                        async move {
                            let room_id_parsed =
                                RoomId::parse(&room_id).map_err(|e| e.to_string())?;
                            let client = engine.client().await;
                            let room = client
                                .get_room(&room_id_parsed)
                                .ok_or_else(|| "Room not found".to_string())?;

                            let pl = room.power_levels().await.map_err(|e| e.to_string())?;
                            let current_user_id = client.user_id().map(|id| id.to_string());
                            let notification_settings = client.notification_settings().await;
                            let notification_mode = notification_settings
                                .get_user_defined_room_notification_mode(&room_id_parsed)
                                .await;

                            let join_rule = room
                                .get_state_event_static::<matrix_sdk::ruma::events::room::join_rules::RoomJoinRulesEventContent>()
                                .await
                                .ok()
                                .flatten()
                                .and_then(|e| e.deserialize().ok())
                                .and_then(|e| match e {
                                    matrix_sdk_base::deserialized_responses::SyncOrStrippedState::Sync(
                                        matrix_sdk::ruma::events::SyncStateEvent::Original(ev),
                                    ) => Some(ev.content.join_rule),
                                    matrix_sdk_base::deserialized_responses::SyncOrStrippedState::Stripped(
                                        ev,
                                    ) => Some(ev.content.join_rule),
                                    _ => None,
                                });

                            let history_visibility = room
                                .get_state_event_static::<matrix_sdk::ruma::events::room::history_visibility::RoomHistoryVisibilityEventContent>()
                                .await
                                .ok()
                                .flatten()
                                .and_then(|e| e.deserialize().ok())
                                .and_then(|e| match e {
                                    matrix_sdk_base::deserialized_responses::SyncOrStrippedState::Sync(
                                        matrix_sdk::ruma::events::SyncStateEvent::Original(ev),
                                    ) => Some(ev.content.history_visibility),
                                    matrix_sdk_base::deserialized_responses::SyncOrStrippedState::Stripped(
                                        ev,
                                    ) => Some(ev.content.history_visibility),
                                    _ => None,
                                });

                            let pinned_events = room
                                .get_state_event_static::<matrix_sdk::ruma::events::room::pinned_events::RoomPinnedEventsEventContent>()
                                .await
                                .ok()
                                .flatten()
                                .and_then(|e| e.deserialize().ok())
                                .and_then(|e| match e {
                                    matrix_sdk_base::deserialized_responses::SyncOrStrippedState::Sync(
                                        matrix_sdk::ruma::events::SyncStateEvent::Original(ev),
                                    ) => Some(ev.content.pinned),
                                    matrix_sdk_base::deserialized_responses::SyncOrStrippedState::Stripped(
                                        ev,
                                    ) => ev.content.pinned,
                                    _ => None,
                                })
                                .unwrap_or_default();

                            let is_encrypted = room.encryption_settings().is_some();
                            let (canonical_alias, alt_aliases) = room
                                .get_state_event_static::<matrix_sdk::ruma::events::room::canonical_alias::RoomCanonicalAliasEventContent>()
                                .await
                                .ok()
                                .flatten()
                                .and_then(|e| e.deserialize().ok())
                                .and_then(|e| match e {
                                    matrix_sdk_base::deserialized_responses::SyncOrStrippedState::Sync(
                                        matrix_sdk::ruma::events::SyncStateEvent::Original(ev),
                                    ) => Some((
                                        ev.content.alias.map(|a| a.to_string()),
                                        ev.content.alt_aliases.into_iter().map(|a| a.to_string()).collect(),
                                    )),
                                    matrix_sdk_base::deserialized_responses::SyncOrStrippedState::Stripped(
                                        ev,
                                    ) => Some((
                                        ev.content.alias.map(|a| a.to_string()),
                                        ev.content.alt_aliases.into_iter().map(|a| a.to_string()).collect(),
                                    )),
                                    _ => None,
                                })
                                .unwrap_or((None, Vec::new()));

                            Ok(RoomInfo {
                                name: room.name().unwrap_or_default(),
                                topic: room.topic().unwrap_or_default(),
                                avatar_url: room.avatar_url().map(|u| u.to_string()),
                                membership: room.state(),
                                ban_level: pl.ban.into(),
                                invite_level: pl.invite.into(),
                                kick_level: pl.kick.into(),
                                redact_level: pl.redact.into(),
                                events_default_level: pl.events_default.into(),
                                room_name_level: pl.events.get(&matrix_sdk::ruma::events::TimelineEventType::RoomName).map(|l| (*l).into()).unwrap_or(pl.state_default.into()),
                                room_topic_level: pl.events.get(&matrix_sdk::ruma::events::TimelineEventType::RoomTopic).map(|l| (*l).into()).unwrap_or(pl.state_default.into()),
                                room_avatar_level: pl.events.get(&matrix_sdk::ruma::events::TimelineEventType::RoomAvatar).map(|l| (*l).into()).unwrap_or(pl.state_default.into()),
                                current_user_id,
                                notification_mode,
                                join_rule,
                                history_visibility,
                                pinned_events,
                                is_encrypted,
                                canonical_alias,
                                alt_aliases,
                            })
                        },
                        |res| Action::from(crate::Message::RoomSettings(Message::RoomLoaded(res))),
                    )
                } else {
                    Task::none()
                }
            }
            Message::RoomLoaded(res) => {
                self.is_loading = false;
                match res {
                    Ok(info) => {
                        self.name = info.name.clone();
                        self.original_name = info.name;
                        self.topic = info.topic.clone();
                        self.original_topic = info.topic;
                        self.avatar_url = info.avatar_url;
                        self.membership = Some(info.membership);
                        self.kick_level = info.kick_level;
                        self.original_kick_level = info.kick_level;
                        self.kick_level_str = info.kick_level.to_string();
                        self.redact_level = info.redact_level;
                        self.original_redact_level = info.redact_level;
                        self.redact_level_str = info.redact_level.to_string();
                        self.ban_level = info.ban_level;
                        self.original_ban_level = info.ban_level;
                        self.ban_level_str = info.ban_level.to_string();
                        self.invite_level = info.invite_level;
                        self.original_invite_level = info.invite_level;
                        self.invite_level_str = info.invite_level.to_string();
                        self.events_default_level = info.events_default_level;
                        self.original_events_default_level = info.events_default_level;
                        self.events_default_level_str = info.events_default_level.to_string();
                        self.room_name_level = info.room_name_level;
                        self.original_room_name_level = info.room_name_level;
                        self.room_name_level_str = info.room_name_level.to_string();
                        self.room_topic_level = info.room_topic_level;
                        self.original_room_topic_level = info.room_topic_level;
                        self.room_topic_level_str = info.room_topic_level.to_string();
                        self.room_avatar_level = info.room_avatar_level;
                        self.original_room_avatar_level = info.room_avatar_level;
                        self.room_avatar_level_str = info.room_avatar_level.to_string();
                        self.current_user_id = info.current_user_id;
                        self.notification_mode = info.notification_mode;
                        self.join_rule = info.join_rule;
                        self.history_visibility = info.history_visibility;
                        self.join_rule = info.join_rule.clone();
                        self.restricted_space_id = match &info.join_rule {
                            Some(matrix_sdk::ruma::events::room::join_rules::JoinRule::Restricted(r)) => {
                                r.allow.iter().find_map(|a| match a {
                                    matrix_sdk::ruma::events::room::join_rules::AllowRule::RoomMembership(m) => Some(m.room_id.to_string()),
                                    _ => None,
                                }).unwrap_or_default()
                            }
                            _ => String::new(),
                        };
                        self.pinned_events = info.pinned_events;
                        self.pinned_event_id_input = String::new();
                        self.is_encrypted = info.is_encrypted;
                        self.canonical_alias = info.canonical_alias.clone().unwrap_or_default();
                        self.original_canonical_alias = info.canonical_alias.unwrap_or_default();
                        self.alt_aliases = info.alt_aliases.clone();
                        self.original_alt_aliases = info.alt_aliases;
                        self.new_alt_alias_input = String::new();
                        self.error = None;

                        let mut tasks = Vec::new();

                        if let Some(url) = &self.avatar_url
                            && let Some(matrix) = matrix
                        {
                            let engine = matrix.clone();
                            let mxc = url.clone();
                            self.is_loading_avatar = true;
                            tasks.push(Task::perform(
                                async move {
                                    let mxc_uri = <&matrix_sdk::ruma::MxcUri>::from(mxc.as_str());
                                    let source = MediaSource::Plain(mxc_uri.to_owned());
                                    engine.fetch_media(source).await.map_err(|e| e.to_string())
                                },
                                |res| {
                                    Action::from(crate::Message::RoomSettings(
                                        Message::AvatarMediaFetched(res),
                                    ))
                                },
                            ));
                        }

                        tasks.push(Task::done(Action::from(crate::Message::RoomSettings(
                            Message::LoadPowerLevels,
                        ))));
                        return Task::batch(tasks);
                    }
                    Err(e) => {
                        self.error = Some(e);
                    }
                }
                Task::none()
            }
            Message::HistoryVisibilityChanged(history_visibility) => {
                if let Some(matrix) = matrix
                    && let Some(room_id) = &self.room_id
                {
                    let engine = matrix.clone();
                    let room_id_clone = room_id.clone();
                    let room_id_clone_reload = room_id.clone();
                    return Task::perform(
                        async move {
                            engine
                                .set_room_history_visibility(&room_id_clone, history_visibility)
                                .await
                                .map_err(|e| e.to_string())
                        },
                        move |res| {
                            Action::from(crate::Message::RoomSettings(match res {
                                Ok(_) => {
                                    // Reload room data to reflect changes
                                    Message::LoadRoom(room_id_clone_reload.clone())
                                }
                                Err(e) => Message::RoomSaved(Err(e)),
                            }))
                        },
                    );
              }
              Task::none()
            }
            Message::EventsDefaultLevelChanged(l) => {
                self.events_default_level_str = l.clone();
                if let Ok(l) = l.parse() {
                    self.events_default_level = l;
                }
                Task::none()
            }
            Message::RoomNameLevelChanged(l) => {
                self.room_name_level_str = l.clone();
                if let Ok(l) = l.parse() {
                    self.room_name_level = l;
                }
                Task::none()
            }
            Message::RoomTopicLevelChanged(l) => {
                self.room_topic_level_str = l.clone();
                if let Ok(l) = l.parse() {
                    self.room_topic_level = l;
                }
                Task::none()
            }
            Message::RoomAvatarLevelChanged(l) => {
                self.room_avatar_level_str = l.clone();
                if let Ok(l) = l.parse() {
                    self.room_avatar_level = l;
                }
                Task::none()
            }
            Message::LoadPowerLevels => {
                if let Some(matrix) = matrix
                    && let Some(room_id) = &self.room_id
                {
                    self.is_loading_power_levels = true;
                    let engine = matrix.clone();
                    let room_id_clone = room_id.clone();
                    return Task::perform(
                        async move {
                            let (default, users) = engine
                                .get_room_power_levels(&room_id_clone)
                                .await
                                .map_err(|e| e.to_string())?;
                            let client = engine.client().await;
                            let user_id = client.user_id().ok_or("No user ID")?;
                            let room = client
                                .get_room(
                                    &RoomId::parse(&room_id_clone).map_err(|e| e.to_string())?,
                                )
                                .ok_or("Room not found")?;
                            let my_level = match room.get_user_power_level(user_id).await {
                                    Ok(matrix_sdk::ruma::events::room::power_levels::UserPowerLevel::Int(l)) => l.into(),
                                    Ok(matrix_sdk::ruma::events::room::power_levels::UserPowerLevel::Infinite) => 100, // Room creators are basically 100+
                                    Ok(_) => 100, // Handle future versions gracefully
                                    Err(_) => default,
                                };
                            Ok(PowerLevelInfo {
                                default_level: default,
                                users,
                                my_level,
                            })
                        },
                        |res| {
                            Action::from(crate::Message::RoomSettings(Message::PowerLevelsLoaded(
                                res,
                            )))
                        },
                    );
                }
                Task::none()
            }
            Message::PowerLevelsLoaded(res) => {
                self.is_loading_power_levels = false;
                match res {
                    Ok(info) => {
                        self.power_levels = Some((info.default_level, info.users));
                        self.my_power_level = info.my_level;
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to load power levels: {}", e));
                    }
                }
                Task::none()
            }
            Message::InviteUser => {
                if let Some(matrix) = matrix
                    && let Some(room_id) = &self.room_id
                {
                    let engine = matrix.clone();
                    let room_id_clone = room_id.clone();
                    let user_id_clone = self.invite_user_id.clone();
                    return Task::perform(
                        async move {
                            engine
                                .invite_user(&room_id_clone, &user_id_clone)
                                .await
                                .map_err(|e| e.to_string())
                        },
                        |res| Action::from(crate::Message::RoomSettings(Message::UserInvited(res))),
                    );
                }
                Task::none()
            }
            Message::UserInvited(res) => {
                match res {
                    Ok(_) => {
                        self.invite_user_id = String::new();
                        self.error = None;
                        return Task::done(Action::from(crate::Message::RoomSettings(
                            Message::LoadPowerLevels,
                        )));
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to invite user: {}", e));
                    }
                }
                Task::none()
            }
            Message::KickUser(user_id) => {
                if let Some(matrix) = matrix
                    && let Some(room_id) = &self.room_id
                {
                    let engine = matrix.clone();
                    let room_id_clone = room_id.clone();
                    let user_id_clone = user_id.clone();
                    let user_id_for_task = user_id.clone();
                    let reason = if self.action_reason.is_empty() {
                        None
                    } else {
                        Some(self.action_reason.clone())
                    };
                    return Task::perform(
                        async move {
                            engine
                                .kick_user(&room_id_clone, &user_id_for_task, reason)
                                .await
                                .map_err(|e| e.to_string())
                        },
                        move |res| {
                            Action::from(crate::Message::RoomSettings(Message::UserKicked(
                                user_id_clone,
                                res,
                            )))
                        },
                    );
                }
                Task::none()
            }
            Message::JoinRuleChanged(join_rule) => {
                self.join_rule = Some(join_rule.clone());
                if let Some(matrix) = matrix
                    && let Some(room_id) = &self.room_id
                {
                    let engine = matrix.clone();
                    let room_id_clone = room_id.clone();
                    let room_id_clone_reload = room_id.clone();
                    return Task::perform(
                        async move {
                            engine
                                .set_room_join_rule(&room_id_clone, join_rule)
                                .await
                                .map_err(|e| e.to_string())
                        },
                        move |res| {
                            Action::from(crate::Message::RoomSettings(match res {
                                Ok(_) => {
                                    // Reload room data to reflect changes
                                    Message::LoadRoom(room_id_clone_reload.clone())
                                }
                                Err(e) => Message::RoomSaved(Err(e)),
                            }))
                        },
                    );
                }
                Task::none()
            }
            Message::UserKicked(user_id, res) => {
                match res {
                    Ok(_) => {
                        self.action_reason = String::new();
                        self.error = None;
                        return Task::done(Action::from(crate::Message::RoomSettings(
                            Message::LoadPowerLevels,
                        )));
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to kick {}: {}", user_id, e));
                    }
                }
                Task::none()
            }
            Message::BanUser(user_id) => {
                if let Some(matrix) = matrix
                    && let Some(room_id) = &self.room_id
                {
                    let engine = matrix.clone();
                    let room_id_clone = room_id.clone();
                    let user_id_clone = user_id.clone();
                    let user_id_for_task = user_id.clone();
                    let reason = if self.action_reason.is_empty() {
                        None
                    } else {
                        Some(self.action_reason.clone())
                    };
                    return Task::perform(
                        async move {
                            engine
                                .ban_user(&room_id_clone, &user_id_for_task, reason)
                                .await
                                .map_err(|e| e.to_string())
                        },
                        move |res| {
                            Action::from(crate::Message::RoomSettings(Message::UserBanned(
                                user_id_clone,
                                res,
                            )))
                        },
                    );
                }
                Task::none()
            }
            Message::UserBanned(user_id, res) => {
                match res {
                    Ok(_) => {
                        self.action_reason = String::new();
                        self.error = None;
                        return Task::done(Action::from(crate::Message::RoomSettings(
                            Message::LoadPowerLevels,
                        )));
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to ban {}: {}", user_id, e));
                    }
                }
                Task::none()
            }
            Message::InviteUserIdChanged(id) => {
                self.invite_user_id = id;
                Task::none()
            }
            Message::ActionReasonChanged(r) => {
                self.action_reason = r;
                Task::none()
            }
            Message::MemberFilterChanged(f) => {
                self.member_filter = f;
                Task::none()
            }
            Message::UpdatePowerLevel(user_id, level) => {
                if let Some(matrix) = matrix
                    && let Some(room_id) = &self.room_id
                {
                    self.updating_power_level_for = Some(user_id.clone());
                    let engine = matrix.clone();
                    let room_id_clone = room_id.clone();
                    let user_id_clone = user_id.clone();
                    let user_id_for_task = user_id.clone();
                    return Task::perform(
                        async move {
                            engine
                                .update_user_power_level(&room_id_clone, &user_id_for_task, level)
                                .await
                                .map_err(|e| e.to_string())
                        },
                        move |res| {
                            Action::from(crate::Message::RoomSettings(Message::PowerLevelUpdated(
                                user_id_clone,
                                res,
                            )))
                        },
                    );
                }
                Task::none()
            }
            Message::PowerLevelUpdated(user_id, res) => {
                self.updating_power_level_for = None;
                match res {
                    Ok(_) => {
                        self.invite_user_id = String::new();
                        return Task::done(Action::from(crate::Message::RoomSettings(
                            Message::LoadPowerLevels,
                        )));
                    }
                    Err(e) => {
                        self.error = Some(format!(
                            "Failed to update power level for {}: {}",
                            user_id, e
                        ));
                    }
                }
                Task::none()
            }
            Message::BanLevelChanged(l) => {
                self.ban_level_str = l.clone();
                if let Ok(l) = l.parse() {
                    self.ban_level = l;
                }
                Task::none()
            }
            Message::InviteLevelChanged(l) => {
                self.invite_level_str = l.clone();
                if let Ok(l) = l.parse() {
                    self.invite_level = l;
                }
                Task::none()
            }
            Message::KickLevelChanged(l) => {
                self.kick_level_str = l.clone();
                if let Ok(l) = l.parse() {
                    self.kick_level = l;
                }
                Task::none()
            }
            Message::RedactLevelChanged(l) => {
                self.redact_level_str = l.clone();
                if let Ok(l) = l.parse() {
                    self.redact_level = l;
                }
                Task::none()
            }
            Message::AvatarMediaFetched(res) => {
                self.is_loading_avatar = false;
                match res {
                    Ok(data) => {
                        self.avatar_handle =
                            Some(cosmic::iced::widget::image::Handle::from_bytes(data));
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to fetch avatar: {}", e));
                    }
                }
                Task::none()
            }
            Message::NameChanged(name) => {
                self.name = name;
                Task::none()
            }
            Message::TopicChanged(topic) => {
                self.topic = topic;
                Task::none()
            }
            Message::SaveRoom => {
                if let Some(matrix) = matrix {
                    if let Some(room_id) = &self.room_id {
                        self.is_saving = true;
                        self.error = None;

                        let engine = matrix.clone();
                        let new_name = self.name.clone();
                        let new_topic = self.topic.clone();
                        let room_id_clone = room_id.clone();
                        let original_name = self.original_name.clone();
                        let original_topic = self.original_topic.clone();
                        let original_ban = self.original_ban_level;
                        let original_invite = self.original_invite_level;
                        let original_kick = self.original_kick_level;
                        let original_redact = self.original_redact_level;
                        let original_events_default = self.original_events_default_level;
                        let original_room_name = self.original_room_name_level;
                        let original_room_topic = self.original_room_topic_level;
                        let original_room_avatar = self.original_room_avatar_level;

                        let new_ban = self.ban_level;
                        let new_invite = self.invite_level;
                        let new_kick = self.kick_level;
                        let new_redact = self.redact_level;
                        let original_canonical = self.original_canonical_alias.clone();
                        let new_canonical = if self.canonical_alias.is_empty() {
                            None
                        } else {
                            Some(self.canonical_alias.clone())
                        };
                        let original_alt = self.original_alt_aliases.clone();
                        let new_alt = self.alt_aliases.clone();
                        let new_events_default = self.events_default_level;
                        let new_room_name = self.room_name_level;
                        let new_room_topic = self.room_topic_level;
                        let new_room_avatar = self.room_avatar_level;

                        Task::perform(
                            async move {
                                if new_name != original_name {
                                    engine
                                        .set_room_name(&room_id_clone, new_name)
                                        .await
                                        .map_err(|e| e.to_string())?;
                                }
                                if new_topic != original_topic {
                                    engine
                                        .set_room_topic(&room_id_clone, new_topic)
                                        .await
                                        .map_err(|e| e.to_string())?;
                                }
                                if new_ban != original_ban
                                    || new_invite != original_invite
                                    || new_kick != original_kick
                                    || new_redact != original_redact
                                    || new_events_default != original_events_default
                                    || new_room_name != original_room_name
                                    || new_room_topic != original_room_topic
                                    || new_room_avatar != original_room_avatar
                                {
                                    engine
                                        .update_room_power_level_settings(
                                            &room_id_clone,
                                            if new_ban != original_ban {
                                                Some(new_ban)
                                            } else {
                                                None
                                            },
                                            if new_invite != original_invite {
                                                Some(new_invite)
                                            } else {
                                                None
                                            },
                                            if new_kick != original_kick {
                                                Some(new_kick)
                                            } else {
                                                None
                                            },
                                            if new_redact != original_redact {
                                                Some(new_redact)
                                            } else {
                                                None
                                            },
                                            if new_events_default != original_events_default {
                                                Some(new_events_default)
                                            } else {
                                                None
                                            },
                                            if new_room_name != original_room_name {
                                                Some(new_room_name)
                                            } else {
                                                None
                                            },
                                            if new_room_topic != original_room_topic {
                                                Some(new_room_topic)
                                            } else {
                                                None
                                            },
                                            if new_room_avatar != original_room_avatar {
                                                Some(new_room_avatar)
                                            } else {
                                                None
                                            },
                                        )
                                        .await
                                        .map_err(|e| e.to_string())?;
                                }

                                if new_canonical.as_deref() != Some(&original_canonical)
                                    || new_alt != original_alt
                                {
                                    engine
                                        .update_room_aliases(&room_id_clone, new_canonical, new_alt)
                                        .await
                                        .map_err(|e| e.to_string())?;
                                }

                                Ok(())
                            },
                            |res| {
                                Action::from(crate::Message::RoomSettings(Message::RoomSaved(res)))
                            },
                        )
                    } else {
                        Task::none()
                    }
                } else {
                    Task::none()
                }
            }
            Message::RoomSaved(res) => {
                self.is_saving = false;
                match res {
                    Ok(_) => {
                        self.original_name = self.name.clone();
                        self.original_topic = self.topic.clone();
                        self.original_ban_level = self.ban_level;
                        self.original_invite_level = self.invite_level;
                        self.original_kick_level = self.kick_level;
                        self.original_redact_level = self.redact_level;
                        self.original_canonical_alias = self.canonical_alias.clone();
                        self.original_alt_aliases = self.alt_aliases.clone();
                        self.original_events_default_level = self.events_default_level;
                        self.original_room_name_level = self.room_name_level;
                        self.original_room_topic_level = self.room_topic_level;
                        self.original_room_avatar_level = self.room_avatar_level;
                        self.error = None;
                    }
                    Err(e) => {
                        self.error = Some(e);
                    }
                }
                Task::none()
            }
            Message::SelectAvatar => Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .add_filter("Images", &["png", "jpg", "jpeg", "webp", "gif"])
                        .set_title("Select Room Avatar")
                        .pick_file()
                        .await
                        .map(|handle| handle.path().to_owned())
                },
                |res| {
                    Action::from(crate::Message::RoomSettings(Message::AvatarFileSelected(
                        res,
                    )))
                },
            ),
            Message::AvatarFileSelected(path_opt) => {
                if let Some(path) = path_opt
                    && let Some(matrix) = matrix
                {
                    self.is_uploading_avatar = true;
                    let engine = matrix.clone();
                    let room_id = self.room_id.clone().unwrap_or_default();

                    return Task::perform(
                        async move {
                            let data = std::fs::read(&path).map_err(|e| e.to_string())?;
                            let mime = mime_guess::from_path(&path)
                                .first_raw()
                                .unwrap_or("image/jpeg");
                            engine
                                .upload_room_avatar(&room_id, data, mime)
                                .await
                                .map_err(|e| e.to_string())
                        },
                        |res| {
                            Action::from(crate::Message::RoomSettings(Message::AvatarUploaded(res)))
                        },
                    );
                }
                Task::none()
            }
            Message::AvatarUploaded(res) => {
                self.is_uploading_avatar = false;
                match res {
                    Ok(_) => {
                        // Reload room data to get new avatar URL
                        if let Some(room_id) = &self.room_id {
                            return self.update(Message::LoadRoom(room_id.clone()), matrix);
                        }
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to upload avatar: {}", e));
                    }
                }
                Task::none()
            }
            Message::LeaveRoom => {
                if let Some(matrix) = matrix
                    && let Some(room_id) = &self.room_id
                {
                    self.is_saving = true;
                    let engine = matrix.clone();
                    let room_id_clone = room_id.clone();
                    return Task::perform(
                        async move {
                            engine
                                .leave_room(&room_id_clone)
                                .await
                                .map_err(|e| e.to_string())
                        },
                        |res| Action::from(crate::Message::RoomSettings(Message::RoomLeft(res))),
                    );
                }
                Task::none()
            }
            Message::RoomLeft(res) => {
                self.is_saving = false;
                match res {
                    Ok(_) => {
                        // Reload to update membership state UI
                        if let Some(room_id) = &self.room_id {
                            return self.update(Message::LoadRoom(room_id.clone()), matrix);
                        }
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to leave room: {}", e));
                    }
                }
                Task::none()
            }
            Message::ForgetRoom => {
                if let Some(matrix) = matrix
                    && let Some(room_id) = &self.room_id
                {
                    self.is_saving = true;
                    let engine = matrix.clone();
                    let room_id_clone = room_id.clone();
                    return Task::perform(
                        async move {
                            engine
                                .forget_room(&room_id_clone)
                                .await
                                .map_err(|e| e.to_string())
                        },
                        |res| {
                            Action::from(crate::Message::RoomSettings(Message::RoomForgotten(res)))
                        },
                    );
                }
                Task::none()
            }
            Message::RoomForgotten(res) => {
                self.is_saving = false;
                match res {
                    Ok(_) => {
                        // Close settings panel as the room is gone
                        return Task::done(Action::from(crate::Message::CloseSettings));
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to forget room: {}", e));
                    }
                }
                Task::none()
            }
            Message::NotificationModeChanged(mode) => {
                if let Some(matrix) = matrix
                    && let Some(room_id) = &self.room_id
                {
                    self.is_loading_notifications = true;
                    self.notification_mode = Some(mode);
                    let engine = matrix.clone();
                    let room_id_clone = room_id.clone();
                    return Task::perform(
                        async move {
                            let client = engine.client().await;
                            let ns = client.notification_settings().await;
                            let rid = RoomId::parse(&room_id_clone).map_err(|e| e.to_string())?;
                            ns.set_room_notification_mode(&rid, mode)
                                .await
                                .map_err(|e| e.to_string())
                        },
                        |res| {
                            Action::from(crate::Message::RoomSettings(
                                Message::NotificationModeSet(res),
                            ))
                        },
                    );
                }
                Task::none()
            }
            Message::NotificationModeSet(res) => {
                self.is_loading_notifications = false;
                if let Err(e) = res {
                    self.error = Some(e);
                }
                Task::none()
            }
            Message::DismissError => {
                self.error = None;
                Task::none()
            }
            Message::RestrictedSpaceIdChanged(id) => {
                self.restricted_space_id = id;
                Task::none()
            }
            Message::PinnedEventIdChanged(id) => {
                self.pinned_event_id_input = id;
                Task::none()
            }
            Message::PinEvent => {
                if let Some(matrix) = matrix
                    && let Some(room_id) = &self.room_id
                {
                    let event_id_res =
                        matrix_sdk::ruma::EventId::parse(&self.pinned_event_id_input);
                    match event_id_res {
                        Ok(event_id) => {
                            if !self.pinned_events.contains(&event_id) {
                                let mut new_pinned = self.pinned_events.clone();
                                new_pinned.push(event_id);
                                let engine = matrix.clone();
                                let room_id_clone = room_id.clone();
                                let room_id_clone_reload = room_id.clone();
                                return Task::perform(
                                    async move {
                                        engine
                                            .set_pinned_events(&room_id_clone, new_pinned)
                                            .await
                                            .map_err(|e| e.to_string())
                                    },
                                    move |res| {
                                        Action::from(crate::Message::RoomSettings(match res {
                                            Ok(_) => {
                                                Message::LoadRoom(room_id_clone_reload.clone())
                                            }
                                            Err(e) => Message::RoomSaved(Err(e)),
                                        }))
                                    },
                                );
                            }
                        }
                        Err(e) => {
                            self.error = Some(format!("Invalid Event ID: {}", e));
                        }
                    }
                }
                Task::none()
            }
            Message::UnpinEvent(event_id) => {
                if let Some(matrix) = matrix
                    && let Some(room_id) = &self.room_id
                {
                    let mut new_pinned = self.pinned_events.clone();
                    new_pinned.retain(|id| id != &event_id);
                    let engine = matrix.clone();
                    let room_id_clone = room_id.clone();
                    let room_id_clone_reload = room_id.clone();
                    return Task::perform(
                        async move {
                            engine
                                .set_pinned_events(&room_id_clone, new_pinned)
                                .await
                                .map_err(|e| e.to_string())
                        },
                        move |res| {
                            Action::from(crate::Message::RoomSettings(match res {
                                Ok(_) => Message::LoadRoom(room_id_clone_reload.clone()),
                                Err(e) => Message::RoomSaved(Err(e)),
                            }))
                        },
                    );
                }
                Task::none()
            }
            Message::EnableEncryption => {
                if let Some(matrix) = matrix
                    && let Some(room_id) = &self.room_id
                {
                    let engine = matrix.clone();
                    let room_id_clone = room_id.clone();
                    return Task::perform(
                        async move {
                            engine
                                .enable_encryption(&room_id_clone)
                                .await
                                .map_err(|e| e.to_string())
                        },
                        |res| {
                            Action::from(crate::Message::RoomSettings(Message::EncryptionEnabled(
                                res,
                            )))
                        },
                    );
                }
                Task::none()
            }
            Message::EncryptionEnabled(res) => {
                match res {
                    Ok(_) => {
                        if let Some(room_id) = &self.room_id {
                            return self.update(Message::LoadRoom(room_id.clone()), matrix);
                        }
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to enable encryption: {}", e));
                    }
                }
                Task::none()
            }
            Message::CanonicalAliasChanged(alias) => {
                self.canonical_alias = alias;
                Task::none()
            }
            Message::AltAliasAdded => {
                let alias = self.new_alt_alias_input.trim().to_string();
                if !alias.is_empty() && !self.alt_aliases.contains(&alias) {
                    self.alt_aliases.push(alias);
                }
                self.new_alt_alias_input = String::new();
                Task::none()
            }
            Message::AltAliasRemoved(alias) => {
                self.alt_aliases.retain(|a| a != &alias);
                Task::none()
            }
            Message::NewAltAliasInputChanged(input) => {
                self.new_alt_alias_input = input;
                Task::none()
            }
        }
    }

    fn view_notifications(&self) -> Element<'_, Message> {
        use matrix_sdk::notification_settings::RoomNotificationMode;
        let mut col = Column::new().spacing(10);
        col = col.push(text::title3("Notifications"));

        let mut row = Row::new().spacing(10);

        for mode in [
            RoomNotificationMode::AllMessages,
            RoomNotificationMode::MentionsAndKeywordsOnly,
            RoomNotificationMode::Mute,
        ] {
            let label = match mode {
                RoomNotificationMode::AllMessages => "All Messages",
                RoomNotificationMode::MentionsAndKeywordsOnly => "Mentions Only",
                RoomNotificationMode::Mute => "Muted",
            };

            let mut btn = if self.notification_mode == Some(mode) {
                button::suggested(label)
            } else {
                button::text(label)
            };

            if self.notification_mode != Some(mode) && !self.is_loading_notifications {
                btn = btn.on_press(Message::NotificationModeChanged(mode));
            }

            row = row.push(btn);
        }

        col.push(row).into()
    }

    fn view_error(&self) -> Option<Element<'_, Message>> {
        self.error.as_ref().map(|error| {
            Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(text::body(error))
                .push(button::text("Dismiss").on_press(Message::DismissError))
                .into()
        })
    }

    fn view_security(&self) -> Element<'_, Message> {
        let mut col = Column::new().spacing(10);
        col = col.push(text::title3("Security"));

        let mut row = Row::new().spacing(10).align_y(Alignment::Center);
        row = row.push(text::body("End-to-End Encryption").width(200));

        if self.is_encrypted {
            row = row.push(button::suggested("Enabled"));
            col = col.push(row);
        } else {
            row = row.push(button::destructive("Enable Encryption").on_press(Message::EnableEncryption));
            col = col.push(row);
            col = col.push(
                text::body("⚠️ This is a one-way action and cannot be undone.")
                    .size(12),
            );
        }

        col.into()
    }

    fn view_profile(&self) -> Element<'_, Message> {
        let mut col = Column::new().spacing(20);
        col = col.push(text::title3("Room Profile"));

        // Avatar Section
        let mut avatar_row = Row::new().spacing(20).align_y(Alignment::Center);
        if let Some(handle) = &self.avatar_handle {
            avatar_row = avatar_row.push(
                cosmic::widget::image(handle.clone())
                    .width(cosmic::iced::Length::Fixed(64.0))
                    .height(cosmic::iced::Length::Fixed(64.0)),
            );
        } else if self.is_loading_avatar {
            avatar_row = avatar_row.push(text::body("Loading avatar..."));
        } else {
            avatar_row = avatar_row.push(
                cosmic::widget::container(text::body("No Avatar"))
                    .width(cosmic::iced::Length::Fixed(64.0))
                    .height(cosmic::iced::Length::Fixed(64.0))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            );
        }

        let mut upload_btn = button::text(if self.is_uploading_avatar {
            "Uploading..."
        } else {
            "Change Avatar"
        });
        if !self.is_uploading_avatar {
            upload_btn = upload_btn.on_press(Message::SelectAvatar);
        }
        avatar_row = avatar_row.push(upload_btn);
        col = col.push(avatar_row);

        // Room Name
        col = col.push(
            Column::new()
                .spacing(5)
                .push(text::body("Room Name").size(12))
                .push(text_input::text_input("Name", &self.name).on_input(Message::NameChanged)),
        );

        // Room Topic
        col = col.push(
            Column::new()
                .spacing(5)
                .push(text::body("Room Topic").size(12))
                .push(text_input::text_input("Topic", &self.topic).on_input(Message::TopicChanged)),
        );

        // Room ID
        if let Some(id) = &self.room_id {
            col = col.push(
                Column::new()
                    .spacing(5)
                    .push(text::body("Room ID").size(12))
                    .push(
                        text_input::text_input("", id.as_ref())
                            // Read-only by not providing on_input
                    ),
            );
        }

        col.into()
    }

    fn view_aliases(&self) -> Element<'_, Message> {
        let mut col = Column::new().spacing(10);
        col = col.push(text::title3("Room Aliases"));

        // Canonical Alias
        col = col.push(
            Column::new()
                .spacing(5)
                .push(text::body("Canonical Alias").size(12))
                .push(
                    text_input::text_input("#alias:example.com", &self.canonical_alias)
                        .on_input(Message::CanonicalAliasChanged),
                ),
        );

        // Alternative Aliases
        col = col.push(text::body("Alternative Aliases").size(12));
        for alias in &self.alt_aliases {
            let row = Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(text::body(alias).size(14))
                .push(cosmic::widget::space().width(cosmic::iced::Length::Fill))
                .push(
                    button::destructive("Remove")
                        .on_press(Message::AltAliasRemoved(alias.clone())),
                );
            col = col.push(row);
        }

        // Add Alternative Alias
        let mut add_alias_input = Row::new().spacing(10).align_y(Alignment::Center).push(
            text_input::text_input("#new-alias:example.com", &self.new_alt_alias_input)
                .on_input(Message::NewAltAliasInputChanged)
                .on_submit(|_| Message::AltAliasAdded),
        );

        let is_empty = self.new_alt_alias_input.trim().is_empty();
        let mut add_btn = button::text("Add");
        if !is_empty {
            add_btn = add_btn.on_press(Message::AltAliasAdded);
        }

        let add_widget: Element<'_, Message> = if is_empty {
            tooltip(
                add_btn,
                text::body("Enter an alias to add"),
                Position::Top,
            )
            .into()
        } else {
            add_btn.into()
        };

        add_alias_input = add_alias_input.push(add_widget);
        col = col.push(add_alias_input);

        col.into()
    }

    fn view_permissions(&self) -> Element<'_, Message> {
        use matrix_sdk::ruma::events::room::join_rules::{AllowRule, JoinRule, Restricted};

        let mut perm_col = Column::new().spacing(10);
        perm_col = perm_col.push(text::title3("Permissions"));

        let mut join_rule_row = Row::new().spacing(10).align_y(Alignment::Center);
        join_rule_row = join_rule_row.push(text::body("Join Rule").width(100));

        for rule in [JoinRule::Public, JoinRule::Invite, JoinRule::Knock] {
            let label = match rule {
                JoinRule::Public => "Public",
                JoinRule::Invite => "Invite Only",
                JoinRule::Knock => "Knock",
                _ => unreachable!(),
            };

            let is_selected = self.join_rule.as_ref() == Some(&rule);
            let mut btn = if is_selected {
                button::suggested(label)
            } else {
                button::text(label)
            };

            if !is_selected {
                btn = btn.on_press(Message::JoinRuleChanged(rule));
            }
            join_rule_row = join_rule_row.push(btn);
        }

        let is_restricted = matches!(self.join_rule, Some(JoinRule::Restricted(_)));

        let parsed_restricted_space_id = RoomId::parse(&self.restricted_space_id).ok();

        let mut restricted_btn = if is_restricted {
            button::suggested("Restricted")
        } else {
            button::text("Restricted")
        };

        if !is_restricted {
            if let Some(space_id) = &parsed_restricted_space_id {
                let restricted = Restricted::new(vec![AllowRule::room_membership(space_id.clone())]);
                restricted_btn =
                    restricted_btn.on_press(Message::JoinRuleChanged(JoinRule::Restricted(restricted)));
            }
        }

        join_rule_row = join_rule_row.push(restricted_btn);

        perm_col = perm_col.push(join_rule_row);

        let mut history_visibility_row = Row::new().spacing(10).align_y(Alignment::Center);
        history_visibility_row =
            history_visibility_row.push(text::body("History Visibility").width(100));

        for visibility in [
            HistoryVisibility::Shared,
            HistoryVisibility::Invited,
            HistoryVisibility::Joined,
        ] {
            let label = match visibility {
                HistoryVisibility::Shared => "Shared",
                HistoryVisibility::Invited => "Invited",
                HistoryVisibility::Joined => "Joined",
                _ => unreachable!(),
            };

            let is_selected = self.history_visibility.as_ref() == Some(&visibility);
            let mut btn = if is_selected {
                button::suggested(label)
            } else {
                button::text(label)
            };

            if !is_selected {
                btn = btn.on_press(Message::HistoryVisibilityChanged(visibility));
            }
            history_visibility_row = history_visibility_row.push(btn);
        }

        perm_col = perm_col.push(history_visibility_row);
        if is_restricted || !self.restricted_space_id.is_empty() {
            let mut restricted_row = Row::new().spacing(10).align_y(Alignment::Center);
            restricted_row = restricted_row.push(text::body("Space ID").width(100));
            restricted_row = restricted_row.push(
                text_input::text_input("!space_id:example.com", &self.restricted_space_id)
                    .on_input(Message::RestrictedSpaceIdChanged),
            );

            if let Some(space_id) = parsed_restricted_space_id {
                let current_restricted_match =
                    if let Some(JoinRule::Restricted(r)) = &self.join_rule {
                        r.allow.iter().any(|a| match a {
                            AllowRule::RoomMembership(m) => m.room_id == space_id,
                            _ => false,
                        })
                    } else {
                        false
                    };

                if !current_restricted_match {
                    let restricted = Restricted::new(vec![AllowRule::room_membership(space_id)]);
                    restricted_row = restricted_row.push(
                        button::text("Apply").on_press(Message::JoinRuleChanged(
                            JoinRule::Restricted(restricted),
                        )),
                    );
                }
            }

            perm_col = perm_col.push(restricted_row);
        }

        perm_col = perm_col.push(
            Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(text::body("Invite level").width(100))
                .push(
                    text_input::text_input("50", &self.invite_level_str)
                        .on_input(Message::InviteLevelChanged),
                ),
        );
        perm_col = perm_col.push(
            Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(text::body("Kick level").width(100))
                .push(
                    text_input::text_input("50", &self.kick_level_str)
                        .on_input(Message::KickLevelChanged),
                ),
        );
        perm_col = perm_col.push(
            Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(text::body("Ban level").width(100))
                .push(
                    text_input::text_input("50", &self.ban_level_str)
                        .on_input(Message::BanLevelChanged),
                ),
        );
        perm_col = perm_col.push(
            Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(text::body("Redact level").width(100))
                .push(
                    text_input::text_input("50", &self.redact_level_str)
                        .on_input(Message::RedactLevelChanged),
                ),
        );
        perm_col = perm_col.push(
            Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(text::body("Send messages").width(100))
                .push(
                    text_input::text_input("0", &self.events_default_level_str)
                        .on_input(Message::EventsDefaultLevelChanged),
                ),
        );
        perm_col = perm_col.push(
            Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(text::body("Change name").width(100))
                .push(
                    text_input::text_input("50", &self.room_name_level_str)
                        .on_input(Message::RoomNameLevelChanged),
                ),
        );
        perm_col = perm_col.push(
            Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(text::body("Change topic").width(100))
                .push(
                    text_input::text_input("50", &self.room_topic_level_str)
                        .on_input(Message::RoomTopicLevelChanged),
                ),
        );
        perm_col = perm_col.push(
            Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(text::body("Change avatar").width(100))
                .push(
                    text_input::text_input("50", &self.room_avatar_level_str)
                        .on_input(Message::RoomAvatarLevelChanged),
                ),
        );
        perm_col.into()
    }

    fn view_save_button(&self) -> Option<Element<'_, Message>> {
        let mut save_btn = button::text(if self.is_saving {
            "Saving..."
        } else {
            "Save Changes"
        });

        let has_changes = self.name != self.original_name
            || self.topic != self.original_topic
            || self.ban_level != self.original_ban_level
            || self.invite_level != self.original_invite_level
            || self.kick_level != self.original_kick_level
            || self.redact_level != self.original_redact_level
            || self.canonical_alias != self.original_canonical_alias
            || self.alt_aliases != self.original_alt_aliases;
            || self.events_default_level != self.original_events_default_level
            || self.room_name_level != self.original_room_name_level
            || self.room_topic_level != self.original_room_topic_level
            || self.room_avatar_level != self.original_room_avatar_level;

        if has_changes && !self.is_saving {
            save_btn = save_btn.on_press(Message::SaveRoom);
        }

        let widget: Element<'_, Message> = if !has_changes {
            tooltip(save_btn, text::body("Make changes to save"), Position::Top).into()
        } else {
            save_btn.into()
        };

        Some(widget)
    }

    fn view_manage_members(&self) -> Option<Element<'_, Message>> {
        if let Some((default_level, users)) = &self.power_levels {
            let mut pl_col = Column::new().spacing(10);
            pl_col = pl_col.push(text::title3("Manage Members"));

            // Member Filter
            pl_col = pl_col.push(
                text_input::text_input("Filter members...", &self.member_filter)
                    .on_input(Message::MemberFilterChanged),
            );

            pl_col = pl_col.push(text::body(format!("Default level: {}", default_level)).size(12));

            // Reason for actions (Kick/Ban)
            pl_col = pl_col.push(
                Column::new()
                    .spacing(5)
                    .push(text::body("Reason for action").size(12))
                    .push(
                        text_input::text_input("Reason...", &self.action_reason)
                            .on_input(Message::ActionReasonChanged),
                    ),
            );

            let filter = self.member_filter.to_lowercase();

            let filter_is_ascii = self.member_filter.is_ascii();

            for (user_id, level) in users {
                let user_id_str = user_id.as_str();
                if !filter.is_empty() {
                    let matches =
                        crate::contains_ignore_ascii_case(user_id_str, &filter, filter_is_ascii);

                    if !matches {
                        continue;
                    }
                }

                let is_updating = self.updating_power_level_for.as_deref() == Some(user_id_str);
                let is_me = self.current_user_id.as_deref() == Some(user_id_str);

                let user_row = Row::new()
                    .spacing(10)
                    .align_y(Alignment::Center)
                    .push(text::body(user_id_str).size(14))
                    .push(text::body(level.to_string()).size(14))
                    .push(cosmic::widget::space().width(cosmic::iced::Length::Fill));

                let mut level_row = Row::new().spacing(5);
                for l in [0, 50, 100] {
                    let mut btn = button::text(match l {
                        0 => "Default",
                        50 => "Mod",
                        100 => "Admin",
                        _ => "??",
                    });
                    if !is_updating && *level != l {
                        btn = btn.on_press(Message::UpdatePowerLevel(user_id_str.to_string(), l));
                    }
                    level_row = level_row.push(btn);
                }

                pl_col = pl_col.push(user_row).push(level_row);

                if !is_me {
                    let mut action_row = Row::new().spacing(5);
                    if self.my_power_level >= self.kick_level {
                        action_row = action_row.push(
                            button::destructive("Kick")
                                .on_press(Message::KickUser(user_id_str.to_string())),
                        );
                    }
                    if self.my_power_level >= self.ban_level {
                        action_row = action_row.push(
                            button::destructive("Ban")
                                .on_press(Message::BanUser(user_id_str.to_string())),
                        );
                    }
                    pl_col = pl_col.push(action_row);
                }
            }
            Some(pl_col.into())
        } else {
            None
        }
    }

    fn view_invite_promote(&self) -> Element<'_, Message> {
        let mut add_pl_col = Column::new().spacing(5);
        add_pl_col = add_pl_col.push(text::title3("Invite & Promote"));
        add_pl_col = add_pl_col.push(text::body("User ID").size(12));
        add_pl_col = add_pl_col.push(
            text_input::text_input("@user:example.com", &self.invite_user_id)
                .on_input(Message::InviteUserIdChanged),
        );

        let is_empty = self.invite_user_id.trim().is_empty();

        let mut promote_row = Row::new().spacing(10);

        if self.my_power_level >= self.invite_level {
            let mut invite_btn = button::text("Invite");
            if !is_empty {
                invite_btn = invite_btn.on_press(Message::InviteUser);
            }

            let invite_widget: Element<'_, Message> = if is_empty {
                tooltip(
                    invite_btn,
                    text::body("Enter a User ID to invite/promote"),
                    Position::Top,
                )
                .into()
            } else {
                invite_btn.into()
            };
            promote_row = promote_row.push(invite_widget);
        }

        let mut mod_btn = button::text("Mod");
        let mut admin_btn = button::text("Admin");

        if !is_empty {
            mod_btn = mod_btn.on_press(Message::UpdatePowerLevel(self.invite_user_id.clone(), 50));
            admin_btn =
                admin_btn.on_press(Message::UpdatePowerLevel(self.invite_user_id.clone(), 100));
        }

        let mod_widget: Element<'_, Message> = if is_empty {
            tooltip(
                mod_btn,
                text::body("Enter a User ID to invite/promote"),
                Position::Top,
            )
            .into()
        } else {
            mod_btn.into()
        };

        let admin_widget: Element<'_, Message> = if is_empty {
            tooltip(
                admin_btn,
                text::body("Enter a User ID to invite/promote"),
                Position::Top,
            )
            .into()
        } else {
            admin_btn.into()
        };

        promote_row = promote_row.push(mod_widget).push(admin_widget);

        add_pl_col = add_pl_col.push(promote_row);
        add_pl_col.into()
    }

    fn view_pinned_events(&self) -> Element<'_, Message> {
        let mut col = Column::new().spacing(10);
        col = col.push(text::title3("Pinned Messages"));

        for event_id in &self.pinned_events {
            let row = Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(text::body(event_id.as_str()).size(14))
                .push(cosmic::widget::space().width(cosmic::iced::Length::Fill))
                .push(button::destructive("Unpin").on_press(Message::UnpinEvent(event_id.clone())));
            col = col.push(row);
        }

        let mut pin_input = Row::new().spacing(10).align_y(Alignment::Center).push(
            text_input::text_input("Event ID ($...)", &self.pinned_event_id_input)
                .on_input(Message::PinnedEventIdChanged),
        );

        let is_empty = self.pinned_event_id_input.trim().is_empty();
        let mut pin_btn = button::text("Pin");
        if !is_empty {
            pin_btn = pin_btn.on_press(Message::PinEvent);
        }

        let pin_widget: Element<'_, Message> = if is_empty {
            tooltip(
                pin_btn,
                text::body("Enter an Event ID to pin"),
                Position::Top,
            )
            .into()
        } else {
            pin_btn.into()
        };

        pin_input = pin_input.push(pin_widget);

        col = col.push(pin_input);
        col.into()
    }

    fn view_membership_actions(&self) -> Option<Element<'_, Message>> {
        if let Some(membership) = &self.membership {
            use matrix_sdk::RoomState;
            let mut actions_col = Column::new().spacing(10);
            actions_col = actions_col.push(text::title3("Actions"));

            match membership {
                RoomState::Joined => {
                    actions_col = actions_col
                        .push(button::destructive("Leave Room").on_press(Message::LeaveRoom));
                }
                RoomState::Left | RoomState::Invited => {
                    actions_col = actions_col
                        .push(button::destructive("Forget Room").on_press(Message::ForgetRoom));
                }
                _ => {}
            }
            Some(actions_col.into())
        } else {
            None
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        if self.is_loading {
            return Column::new()
                .spacing(20)
                .push(text::body("Loading room data..."))
                .into();
        }

        let mut col = Column::new().spacing(20);

        if let Some(error_view) = self.view_error() {
            col = col.push(error_view);
        }

        col = col.push(self.view_profile());
        col = col.push(self.view_security());
        col = col.push(self.view_aliases());
        col = col.push(self.view_notifications());
        col = col.push(self.view_permissions());
        col = col.push(self.view_pinned_events());

        if let Some(save_btn) = self.view_save_button() {
            col = col.push(save_btn);
        }

        if let Some(members_view) = self.view_manage_members() {
            col = col.push(members_view);
        } else if self.is_loading_power_levels {
            col = col.push(text::body("Loading members..."));
        }

        col = col.push(self.view_invite_promote());

        if let Some(actions_view) = self.view_membership_actions() {
            col = col.push(actions_view);
        }

        col.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_changed() {
        let mut state = State::default();
        let _ = state.update(Message::NameChanged("New Room Name".to_string()), &None);
        assert_eq!(state.name, "New Room Name");
    }

    #[test]
    fn test_topic_changed() {
        let mut state = State::default();
        let _ = state.update(Message::TopicChanged("New Topic".to_string()), &None);
        assert_eq!(state.topic, "New Topic");
    }

    #[test]
    fn test_load_room_no_matrix() {
        let mut state = State::default();
        let room_id: Arc<str> = Arc::from("!some_room:example.com");
        let _ = state.update(Message::LoadRoom(room_id.clone()), &None);

        // Without matrix engine, it shouldn't try to load
        assert_eq!(state.is_loading, false);
        assert_eq!(state.room_id, None);
    }

    #[test]
    fn test_dismiss_error() {
        let mut state = State::default();
        state.error = Some("An error occurred".to_string());

        let _ = state.update(Message::DismissError, &None);
        assert_eq!(state.error, None);
    }

    #[test]
    fn test_invite_user_id_changed() {
        let mut state = State::default();
        let _ = state.update(
            Message::InviteUserIdChanged("@user:example.com".to_string()),
            &None,
        );
        assert_eq!(state.invite_user_id, "@user:example.com");
    }

    #[test]
    fn test_action_reason_changed() {
        let mut state = State::default();
        let _ = state.update(Message::ActionReasonChanged("Spam".to_string()), &None);
        assert_eq!(state.action_reason, "Spam");
    }

    #[test]
    fn test_member_filter_changed() {
        let mut state = State::default();
        let _ = state.update(Message::MemberFilterChanged("John".to_string()), &None);
        assert_eq!(state.member_filter, "John");
    }

    #[test]
    fn test_join_rule_changed() {
        use matrix_sdk::ruma::events::room::join_rules::JoinRule;
        let mut state = State::default();
        state.room_id = Some(Arc::from("!room:example.com"));
        // This won't actually call the engine since we pass None, but we can check if it returns a Task
        let _task = state.update(Message::JoinRuleChanged(JoinRule::Public), &None);
        // The task should be none since matrix engine is None
        // We can't easily inspect Task, but we can verify it compiles and runs.
    }

    #[test]
    fn test_pinned_event_id_changed() {
        let mut state = State::default();
        let _ = state.update(
            Message::PinnedEventIdChanged("$event:example.com".to_string()),
            &None,
        );
        assert_eq!(state.pinned_event_id_input, "$event:example.com");
    }

    #[test]
    fn test_history_visibility_changed() {
        use matrix_sdk::ruma::events::room::history_visibility::HistoryVisibility;
        let mut state = State::default();
        state.room_id = Some(Arc::from("!room:example.com"));
        // This won't actually call the engine since we pass None
        let _task = state.update(Message::HistoryVisibilityChanged(HistoryVisibility::Shared), &None);
    }
  
    #[test]
    fn test_restricted_space_id_changed() {
        let mut state = State::default();
        let _ = state.update(
            Message::RestrictedSpaceIdChanged("!space:example.com".to_string()),
            &None,
        );
        assert_eq!(state.restricted_space_id, "!space:example.com");
    }

    #[test]
    fn test_join_rule_changed_knock() {
        use matrix_sdk::ruma::events::room::join_rules::JoinRule;
        let mut state = State::default();
        state.room_id = Some(Arc::from("!room:example.com"));
        let _ = state.update(Message::JoinRuleChanged(JoinRule::Knock), &None);
        assert_eq!(state.join_rule, Some(JoinRule::Knock));
    }
  
    #[test]
    fn test_aliases_changed() {
        let mut state = State::default();

        // Test canonical alias change
        let _ = state.update(Message::CanonicalAliasChanged("#new:example.com".to_string()), &None);
        assert_eq!(state.canonical_alias, "#new:example.com");

        // Test alt alias input
        let _ = state.update(Message::NewAltAliasInputChanged("#alt1:example.com".to_string()), &None);
        assert_eq!(state.new_alt_alias_input, "#alt1:example.com");

        // Test alt alias addition
        let _ = state.update(Message::AltAliasAdded, &None);
        assert_eq!(state.alt_aliases, vec!["#alt1:example.com".to_string()]);
        assert_eq!(state.new_alt_alias_input, "");

        // Test alt alias removal
        let _ = state.update(Message::AltAliasRemoved("#alt1:example.com".to_string()), &None);
        assert!(state.alt_aliases.is_empty());
    }
}
