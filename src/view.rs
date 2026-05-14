use chrono::{DateTime, DurationRound, TimeDelta};
use cosmic::{
    Action, Element, Task, Theme,
    iced::{
        Alignment,
        widget::{scrollable, tooltip},
    },
    widget::{
        Column, Container, RcElementWrapper, Row, Text, button, container, divider, icon::Named,
        menu, text, text_input, tooltip::Position,
    },
};
use matrix_sdk::ruma::events::room::{MediaSource, message::MessageType};

use crate::{Constellations, MenuAct, Message, PreviewEvent, matrix};

impl Constellations {
    pub fn view_thread(&self) -> Element<'_, Message> {
        let mut timeline = Column::new().spacing(10).width(cosmic::iced::Length::Fill);

        let filter = self.search_query.to_lowercase();
        let filter_is_ascii = self.search_query.is_ascii();
        let is_filtering =
            self.is_search_active && !filter.is_empty() && self.current_settings_panel.is_none();

        if self.selected_room.is_some() {
            timeline = timeline.push(
                Row::new()
                    .align_y(Alignment::Center)
                    .push(button::text(crate::fl!("close-thread")).on_press(Message::CloseThread))
                    .push(cosmic::widget::space().width(cosmic::iced::Length::Fill))
                    .padding(10),
            );
        }

        for item in &self.threaded_timeline_items {
            if let Some(event) = item.item.as_event() {
                if is_filtering {
                    let body = event
                        .content()
                        .as_message()
                        .map(|m| m.body())
                        .unwrap_or_default();
                    if !crate::contains_ignore_ascii_case(body, &filter, filter_is_ascii) {
                        continue;
                    }
                }
                timeline = timeline.push(self.view_item(item));
            }
        }

        scrollable(timeline)
            .height(cosmic::iced::Length::Fill)
            .into()
    }

    pub fn view_timeline(&self) -> Element<'_, Message> {
        let mut timeline = Column::new().spacing(10).width(cosmic::iced::Length::Fill);

        let filter = self.search_query.to_lowercase();
        let filter_is_ascii = self.search_query.is_ascii();
        let is_filtering =
            self.is_search_active && !filter.is_empty() && self.current_settings_panel.is_none();

        if self.selected_room.is_some() {
            let load_btn = if self.is_loading_more {
                button::text(crate::fl!("loading"))
            } else {
                button::text(crate::fl!("load-more")).on_press(Message::LoadMore)
            };

            timeline = timeline.push(
                container(load_btn)
                    .width(cosmic::iced::Length::Fill)
                    .align_x(Alignment::Center)
                    .padding(10),
            );
        }

        for item in &self.timeline_items {
            if let Some(event) = item.item.as_event() {
                if is_filtering {
                    let body = event
                        .content()
                        .as_message()
                        .map(|m| m.body())
                        .unwrap_or_default();
                    if !crate::contains_ignore_ascii_case(body, &filter, filter_is_ascii) {
                        continue;
                    }
                }
                timeline = timeline.push(self.view_item(item));
            } else if let Some(matrix::VirtualTimelineItem::DateDivider(date)) =
                item.item.as_virtual()
            {
                timeline = timeline.push(
                    Row::new()
                        .push(divider::horizontal::default())
                        .push(text::body(
                            DateTime::from_timestamp_secs(date.as_secs().into())
                                .unwrap_or_default()
                                .duration_trunc(TimeDelta::try_days(1).unwrap_or_default())
                                .unwrap_or_default()
                                .to_rfc2822()
                                .trim_end_matches(" 00:00:00 +0000")
                                .to_owned(),
                        ))
                        .push(divider::horizontal::default())
                        .align_y(Alignment::Center),
                )
            }
        }

        scrollable(timeline)
            .height(cosmic::iced::Length::Fill)
            .on_scroll(Message::TimelineScrolled)
            .into()
    }

    fn view_reactions<'a>(
        &'a self,
        event: &'a matrix_sdk_ui::timeline::EventTimelineItem,
    ) -> Row<'a, Message, cosmic::Theme> {
        let mut reaction_row = Row::new().spacing(5).align_y(Alignment::Center);
        let reactions = event.content().reactions();
        let item_id = event.identifier();

        if let Some(reaction) = reactions {
            for key in reaction.keys() {
                let people = reaction.get_key_value(key);
                let count = people.iter().count();

                let is_me_reacted = people.iter().any(|(user_id, _)| {
                    if let Some(me) = &self.user_id {
                        user_id.as_str() == me
                    } else {
                        false
                    }
                });

                let btn_content =
                    container(text::body(format!("{} {}", key, count)).size(10)).padding([2, 4]);

                // We can differentiate style if reacted, but for now we just wrap in button.
                let btn = button::custom(btn_content)
                    .on_press(Message::ToggleReaction(item_id.clone(), key.clone()));

                // If `is_me_reacted` is true, we could style it differently
                if is_me_reacted {
                    // Use standard button to give it some background highlight, or specific style
                    // But custom button with standard background works nicely if we could pass theme
                }

                reaction_row = reaction_row.push(btn);
            }
        }

        // Show picker if active
        if self.active_reaction_picker.as_ref() == Some(&item_id) {
            let emojis = ["👍", "❤️", "😂", "😮", "😢", "🙏", "👎", "🔥", "🎉", "👀"];
            for emoji in emojis {
                let btn = button::custom(container(text::body(emoji).size(12)).padding([2, 4]))
                    .on_press(Message::ToggleReaction(item_id.clone(), emoji.to_string()));
                reaction_row = reaction_row.push(btn);
            }

            // A cancel button to close picker
            let cancel_btn = button::custom(
                container(cosmic::widget::icon::from_name("window-close-symbolic").size(12))
                    .padding([2, 4]),
            )
            .on_press(Message::OpenReactionPicker(None));
            let cancel_tooltip = tooltip(
                cancel_btn,
                text::body(crate::fl!("close-picker")),
                Position::Top,
            );
            reaction_row = reaction_row.push(cancel_tooltip);
        }

        reaction_row
    }

    fn view_sender_info<'a>(
        &'a self,
        avatar_url: Option<&'a str>,
        sender_name: &'a str,
        timestamp: &'a str,
        sender_id: Option<matrix_sdk::ruma::OwnedUserId>,
        is_ignored: bool,
        is_me: bool,
    ) -> Row<'a, Message, cosmic::Theme> {
        let mut sender_info = Row::new().spacing(5).align_y(Alignment::Center);

        if let Some(mxc_url) = avatar_url {
            if let Some(handle) = self.media_cache.get(mxc_url) {
                sender_info =
                    sender_info.push(cosmic::widget::image(handle.clone()).width(20).height(20));
            } else {
                sender_info = sender_info.push(
                    container(cosmic::widget::icon::from_name("avatar-default-symbolic").size(12))
                        .padding(2),
                );
            }
        } else {
            sender_info = sender_info.push(
                container(cosmic::widget::icon::from_name("avatar-default-symbolic").size(12))
                    .padding(2),
            );
        }

        sender_info = sender_info.push(text::body(sender_name).size(10));

        if !is_me && let Some(id) = sender_id {
            if is_ignored {
                sender_info = sender_info.push(
                    button::icon(Named::new("dialog-error-symbolic"))
                        .on_press(Message::UserSettings(
                            crate::settings::user::Message::UnignoreUserById(id),
                        ))
                        .tooltip("Unignore User"),
                );
            } else {
                sender_info = sender_info.push(
                    button::icon(Named::new("dialog-error-symbolic"))
                        .on_press(Message::UserSettings(
                            crate::settings::user::Message::IgnoreUserById(id),
                        ))
                        .tooltip("Ignore User"),
                );
            }
        }

        sender_info = sender_info.push(text::body(timestamp).size(10));

        sender_info
    }

    fn view_message_image<'a>(
        &'a self,
        image: &'a matrix_sdk::ruma::events::room::message::ImageMessageEventContent,
    ) -> Column<'a, Message, cosmic::Theme> {
        let mut bubble_col = Column::new();
        let mxc_url = match &image.source {
            MediaSource::Plain(uri) => uri.to_string(),
            MediaSource::Encrypted(file) => file.url.to_string(),
        };
        bubble_col = bubble_col.push(text::body(format!("📷 Image: {}", image.body)).size(
            if self.app_settings.compact_mode {
                12
            } else {
                14
            },
        ));

        if self.user_settings.media_previews_display_policy {
            if let Some(handle) = self.media_cache.get(&mxc_url) {
                bubble_col = bubble_col.push(cosmic::widget::image(handle.clone()).width(
                    if self.app_settings.compact_mode {
                        150
                    } else {
                        300
                    },
                ));
            } else {
                bubble_col = bubble_col.push(
                    button::text("Download Image")
                        .on_press(Message::FetchMedia(image.source.clone())),
                );
            }
        }
        bubble_col
    }

    fn view_message_file<'a>(
        &'a self,
        file: &'a matrix_sdk::ruma::events::room::message::FileMessageEventContent,
    ) -> Column<'a, Message, cosmic::Theme> {
        let mut bubble_col = Column::new();
        let mxc_url = match &file.source {
            MediaSource::Plain(uri) => uri.to_string(),
            MediaSource::Encrypted(file) => file.url.to_string(),
        };
        bubble_col = bubble_col.push(text::body(format!("📁 File: {}", file.body)).size(
            if self.app_settings.compact_mode {
                12
            } else {
                14
            },
        ));
        if self.media_cache.contains_key(&mxc_url) {
            bubble_col = bubble_col.push(text::body("[Downloaded]"));
        } else {
            bubble_col = bubble_col.push(
                button::text("Download File").on_press(Message::FetchMedia(file.source.clone())),
            );
        }
        bubble_col
    }

    fn view_markdown_text<'a>(&self, t: &'a str) -> Text<'a, Theme> {
        Text::new(t).size(if self.app_settings.compact_mode {
            12
        } else {
            14
        })
    }

    fn view_markdown_code<'a>(&self, c: &'a str) -> Container<'a, Message, Theme> {
        Container::new(text::body(c).size(if self.app_settings.compact_mode {
            10
        } else {
            12
        }))
        .padding(2)
    }

    fn view_markdown<'a>(&'a self, markdown: &'a [PreviewEvent]) -> Column<'a, Message, Theme> {
        let mut md_col: Column<'a, Message, Theme> =
            Column::new().spacing(if self.app_settings.compact_mode { 2 } else { 5 });
        let mut current_row = Row::new().spacing(0).align_y(Alignment::Center);
        let mut row_has_content = false;

        for event in markdown {
            match event {
                PreviewEvent::StartHeading => {
                    if row_has_content {
                        md_col = md_col.push(current_row);
                        current_row = Row::new().spacing(0).align_y(Alignment::Center);
                        row_has_content = false;
                    }
                }
                PreviewEvent::EndBlock => {
                    if row_has_content {
                        md_col = md_col.push(current_row);
                        current_row = Row::new().spacing(0).align_y(Alignment::Center);
                        row_has_content = false;
                    }
                }
                PreviewEvent::Text(t) => {
                    current_row = current_row.push(self.view_markdown_text(t.as_str()));
                    row_has_content = true;
                }
                PreviewEvent::Code(c) => {
                    current_row = current_row.push(self.view_markdown_code(c.as_str()));
                    row_has_content = true;
                }
                PreviewEvent::Break => {
                    if row_has_content {
                        md_col = md_col.push(current_row);
                        current_row = Row::new().spacing(0).align_y(Alignment::Center);
                        row_has_content = false;
                    }
                }
            }
        }
        if row_has_content {
            md_col = md_col.push(current_row);
        }
        md_col
    }

    fn view_message_text<'a>(
        &'a self,
        message: &'a matrix_sdk::ruma::events::room::message::MessageType,
        markdown: &'a [PreviewEvent],
    ) -> Column<'a, Message, Theme> {
        let mut bubble_col: Column<'a, Message, Theme> = Column::new();
        if self.app_settings.render_markdown {
            bubble_col = bubble_col.push(self.view_markdown(markdown));
        } else {
            bubble_col = bubble_col.push(self.view_markdown_text(message.body()));
        }
        bubble_col
    }

    pub fn view_threaded_timeline(&self) -> Element<'_, Message> {
        let mut timeline = Column::new().spacing(10).width(cosmic::iced::Length::Fill);

        let filter = self.search_query.to_lowercase();
        let filter_is_ascii = self.search_query.is_ascii();
        let is_filtering =
            self.is_search_active && !filter.is_empty() && self.current_settings_panel.is_none();

        let header = Row::new()
            .spacing(10)
            .align_y(Alignment::Center)
            .push(text::title3(crate::fl!("thread")))
            .push(cosmic::widget::space().width(cosmic::iced::Length::Fill))
            .push(button::text(crate::fl!("cancel")).on_press(Message::CloseThread));

        timeline = timeline.push(container(header).padding(10));

        // In a real application, you might want to find and show the root message first.
        // For simplicity, we assume it's part of the threaded timeline from the SDK.

        for item in &self.threaded_timeline_items {
            if let Some(event) = item.item.as_event() {
                if is_filtering {
                    let body = event
                        .content()
                        .as_message()
                        .map(|m| m.body())
                        .unwrap_or_default();
                    if !crate::contains_ignore_ascii_case(body, &filter, filter_is_ascii) {
                        continue;
                    }
                }
                timeline = timeline.push(self.view_item(item));
            }
        }

        scrollable(timeline)
            .height(cosmic::iced::Length::Fill)
            .into()
    }

    pub fn view_preview(&self) -> Element<'_, Message> {
        let mut preview_col = Column::new().spacing(10);

        let mut current_row = Row::new().spacing(0).align_y(Alignment::Center);
        let mut row_has_content = false;

        for event in &self.composer_preview_events {
            match event {
                PreviewEvent::StartHeading => {
                    if row_has_content {
                        preview_col = preview_col.push(current_row);
                        current_row = Row::new().spacing(0).align_y(Alignment::Center);
                        row_has_content = false;
                    }
                }
                PreviewEvent::EndBlock => {
                    if row_has_content {
                        preview_col = preview_col.push(current_row);
                        current_row = Row::new().spacing(0).align_y(Alignment::Center);
                        row_has_content = false;
                    }
                }
                PreviewEvent::Text(t) => {
                    let txt = text::body(t.as_str());
                    current_row = current_row.push(txt);
                    row_has_content = true;
                }
                PreviewEvent::Code(c) => {
                    current_row = current_row.push(container(text::body(c.as_str())).padding(2));
                    row_has_content = true;
                }
                PreviewEvent::Break => {
                    current_row = current_row.push(text::body(" "));
                    row_has_content = true;
                }
            }
        }

        if row_has_content {
            preview_col = preview_col.push(current_row);
        }

        container(scrollable(preview_col).height(100))
            .padding(10)
            .into()
    }

    pub fn view_login(&self) -> Element<'_, Message> {
        let title = if self.is_registering_mode {
            crate::fl!("register-title")
        } else {
            crate::fl!("login-title")
        };
        let mut content = Column::new()
            .spacing(10)
            .padding(20)
            .max_width(400)
            .align_x(Alignment::Center)
            .push(text::title1(title));

        let status_error = match &self.sync_status {
            matrix::SyncStatus::Error(e) => Some(format!("⚠️ Sync Error: {}", e)),
            matrix::SyncStatus::MissingSlidingSyncSupport => Some("Error: Your homeserver does not support Sliding Sync (MSC4186), which is required by Constellations.".to_string()),
            _ => None,
        };

        if let Some(error) = status_error.or_else(|| self.error.clone()) {
            content = content.push(text::body(error));
        }

        let homeserver_input = text_input(crate::fl!("homeserver"), &self.login_homeserver);
        let username_input = text_input(crate::fl!("username"), &self.login_username);
        let password_input = text_input(crate::fl!("password"), &self.login_password).password();

        let (homeserver_input, username_input, password_input) =
            if self.is_logging_in || self.is_oidc_logging_in || self.is_registering {
                (homeserver_input, username_input, password_input)
            } else {
                (
                    homeserver_input.on_input(Message::LoginHomeserverChanged),
                    username_input.on_input(Message::LoginUsernameChanged),
                    password_input
                        .on_input(Message::LoginPasswordChanged)
                        .on_submit(|_| {
                            if self.is_registering_mode {
                                Message::SubmitRegister
                            } else {
                                Message::SubmitLogin
                            }
                        }),
                )
            };

        content = content
            .push(homeserver_input)
            .push(username_input)
            .push(password_input);

        let is_missing_fields = self.login_homeserver.trim().is_empty()
            || self.login_username.trim().is_empty()
            || self.login_password.is_empty();

        let main_button: Element<'_, Message> = if self.is_registering_mode {
            if self.is_registering {
                button::text(crate::fl!("creating-account")).into()
            } else {
                let mut btn = button::text(crate::fl!("create-account-button"));
                if !is_missing_fields {
                    btn = btn.on_press(Message::SubmitRegister);
                }
                if is_missing_fields {
                    tooltip(
                        btn,
                        text::body(crate::fl!("fill-all-fields-register")),
                        Position::Top,
                    )
                    .into()
                } else {
                    btn.into()
                }
            }
        } else if self.is_logging_in {
            button::text(crate::fl!("logging-in")).into()
        } else {
            let mut btn = button::text(crate::fl!("login-button"));
            if !is_missing_fields && !self.is_oidc_logging_in {
                btn = btn.on_press(Message::SubmitLogin);
            }
            if is_missing_fields {
                tooltip(
                    btn,
                    text::body(crate::fl!("fill-all-fields-login")),
                    Position::Top,
                )
                .into()
            } else {
                btn.into()
            }
        };

        let oidc_button = if self.is_oidc_logging_in {
            button::text(crate::fl!("waiting-for-browser"))
        } else {
            let mut btn = button::text(crate::fl!("oidc-login-button"));
            if !self.login_homeserver.is_empty() && !self.is_logging_in && !self.is_registering_mode
            {
                btn = btn.on_press(Message::SubmitOidcLogin);
            }
            btn
        };

        let toggle_mode_button = if self.is_registering_mode {
            button::text(crate::fl!("already-have-account"))
        } else {
            button::text(crate::fl!("need-account"))
        };

        let toggle_mode_button =
            if self.is_logging_in || self.is_registering || self.is_oidc_logging_in {
                toggle_mode_button
            } else {
                toggle_mode_button.on_press(Message::ToggleLoginMode)
            };

        content = content.push(main_button);

        if !self.is_registering_mode {
            content = content.push(oidc_button);
        }

        content = content.push(toggle_mode_button);

        container(content)
            .width(cosmic::iced::Length::Fill)
            .height(cosmic::iced::Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into()
    }

    pub fn update_title(&mut self) -> Task<Action<Message>> {
        let selected_room_name = self.selected_room.as_ref().and_then(|id| {
            self.room_list
                .iter()
                .find(|r| &r.id == id)
                .and_then(|r| r.name.as_deref())
        });

        let title = selected_room_name.unwrap_or("Constellations - Matrix Client");
        self.core.set_header_title(title.to_string());
        Task::none()
    }

    fn view_item<'a>(&'a self, item: &'a crate::ConstellationsItem) -> Element<'a, Message> {
        if let Some(event) = item.item.as_event()
            && let Some(message) = event.content().as_message()
        {
            let is_me = item.is_me;

            let reaction_row = self.view_reactions(event);
            let is_ignored = self.user_settings.ignored_users.contains(&item.sender_id);
            let sender_info = self.view_sender_info(
                item.avatar_url.as_deref(),
                item.sender_name.as_str(),
                item.timestamp.as_str(),
                Some(item.sender_id.clone()),
                is_ignored,
                is_me,
            );

            let mut bubble_col = Column::new()
                .spacing(if self.app_settings.compact_mode { 0 } else { 2 })
                .push(sender_info);

            if let Some(in_reply_to) = event.content().in_reply_to() {
                let mut reply_snippet = String::from("Replying...");
                let mut reply_sender = String::new();

                if let matrix_sdk_ui::timeline::TimelineDetails::Ready(replied_ev) =
                    &in_reply_to.event
                {
                    reply_sender = replied_ev.sender.to_string();
                    if let Some(msg) = replied_ev.content.as_message() {
                        reply_snippet = msg.body().to_string();
                    }
                }

                if reply_snippet.len() > 50 {
                    reply_snippet.truncate(47);
                    reply_snippet.push_str("...");
                }

                let reply_indicator = Row::new().spacing(5).push(text::body("⤴").size(10)).push(
                    text::body(if reply_sender.is_empty() {
                        reply_snippet
                    } else {
                        format!("{}: {}", reply_sender, reply_snippet)
                    })
                    .size(10),
                );

                bubble_col = bubble_col.push(container(reply_indicator).padding([0, 0, 5, 10]));
            }

            match message.msgtype() {
                MessageType::Image(image) => {
                    bubble_col = bubble_col.push(self.view_message_image(image));
                }
                MessageType::File(file) => {
                    bubble_col = bubble_col.push(self.view_message_file(file));
                }
                _ => {
                    bubble_col =
                        bubble_col.push(self.view_message_text(message.msgtype(), &item.markdown));
                }
            }

            if let MessageType::Text(_) = message.msgtype() {
                let mut action_row = Row::new().spacing(5).align_y(Alignment::Center);

                // "Add reaction" button
                let btn = button::text(crate::fl!("reaction")).on_press(
                    Message::OpenReactionPicker(Some(event.identifier().clone())),
                );
                let btn_tooltip = tooltip(
                    btn,
                    text::body(crate::fl!("add-reaction")),
                    Position::Bottom,
                );
                action_row = action_row.push(btn_tooltip);

                // Start a thread
                let root_id = event.identifier();
                let start_thread_btn =
                    button::text(crate::fl!("open-thread")).on_press(match root_id {
                        matrix::TimelineEventItemId::EventId(id) => {
                            Message::OpenThread(id.to_owned())
                        }
                        _ => Message::NoOp,
                    });
                let action_tooltip = tooltip(
                    start_thread_btn,
                    text::body(crate::fl!("tooltip-thread")),
                    Position::Bottom,
                );
                action_row = action_row.push(action_tooltip);

                let reply_btn =
                    button::text(crate::fl!("reply")).on_press(Message::StartReply(item.clone()));
                let reply_tooltip = tooltip(
                    reply_btn,
                    text::body(crate::fl!("tooltip-reply")),
                    Position::Bottom,
                );
                action_row = action_row.push(reply_tooltip);

                bubble_col = bubble_col.push(action_row);
            }

            bubble_col = bubble_col.push(reaction_row);

            let bubble = container(bubble_col)
                .padding(if self.app_settings.compact_mode {
                    5
                } else {
                    10
                })
                .max_width(600);

            let bubble_wrapper =
                container(bubble)
                    .width(cosmic::iced::Length::Fill)
                    .align_x(if is_me {
                        Alignment::End
                    } else {
                        Alignment::Start
                    });

            return bubble_wrapper.into();
        }
        cosmic::widget::space().height(0).into()
    }

    pub fn view_space_switcher(&self) -> Element<'_, Message> {
        let mut content = Column::new().spacing(10).align_x(Alignment::Center);

        // Global icon (All Rooms)
        let is_global_selected = self.selected_space.is_none();

        let global_btn = if is_global_selected {
            button::icon(Named::new("system-users"))
        } else {
            button::icon(Named::new("system-users")).on_press(Message::SelectSpace(None))
        };

        let global_tooltip = tooltip(
            global_btn,
            text::body(crate::fl!("all-rooms")),
            Position::Right,
        );

        content = content.push(global_tooltip);

        for space in self.room_list.iter().filter(|r| r.is_space) {
            let space_id_str = space.id.clone();
            // Try to parse just for validity
            if matrix_sdk::ruma::RoomId::parse(&*space_id_str).is_err() {
                continue;
            }
            let is_selected =
                self.selected_space.as_ref().map(|s| s.as_str()) == Some(&*space_id_str);

            let avatar_element: Element<'_, Message> = if let Some(url) = &space.avatar_url {
                if let Some(handle) = self.media_cache.get(url) {
                    cosmic::widget::image(handle.clone())
                        .width(32)
                        .height(32)
                        .into()
                } else {
                    container(
                        text::body(
                            space
                                .name
                                .as_deref()
                                .unwrap_or("S")
                                .chars()
                                .next()
                                .unwrap_or('S')
                                .to_string(),
                        )
                        .size(24),
                    )
                    .width(32)
                    .height(32)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .into()
                }
            } else {
                container(
                    text::body(
                        space
                            .name
                            .as_deref()
                            .unwrap_or("S")
                            .chars()
                            .next()
                            .unwrap_or('S')
                            .to_string(),
                    )
                    .size(24),
                )
                .width(32)
                .height(32)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .into()
            };

            let space_container = container(avatar_element)
                .padding(8)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center);

            let btn = if is_selected {
                button::custom(space_container)
            } else {
                button::custom(space_container).on_press(Message::SelectSpace(Some(space_id_str)))
            };

            let space_name = space.name.as_deref().unwrap_or("Unknown Space");
            let space_tooltip = tooltip(btn, text::body(space_name), Position::Right);

            content = content.push(space_tooltip);
        }

        let scrollable_spaces = scrollable(content).height(cosmic::iced::Length::Fill);

        let mut bottom_content = Column::new().spacing(10).align_x(Alignment::Center);

        let plus_btn = button::icon(Named::new("list-add-symbolic"));
        let plus_tooltip = tooltip(plus_btn, text::body("Create"), Position::Right);
        let key_binds = std::collections::HashMap::new();

        let menu_tree = menu::Tree::with_children(
            RcElementWrapper::new(Element::from(plus_tooltip)),
            menu::items(
                &key_binds,
                vec![
                    menu::Item::Button("Create Room", None, MenuAct::CreateRoom),
                    menu::Item::Button("Create Space", None, MenuAct::CreateSpace),
                ],
            ),
        );

        let create_menu = menu::bar(vec![menu_tree])
            .item_height(menu::ItemHeight::Dynamic(40))
            .item_width(menu::ItemWidth::Uniform(120))
            .spacing(4.0);

        bottom_content = bottom_content.push(create_menu);

        let layout = Column::new()
            .push(scrollable_spaces)
            .push(bottom_content)
            .align_x(Alignment::Center);

        container(layout).width(60).padding(5).into()
    }

    pub fn view_sidebar(&self) -> Element<'_, Message> {
        let mut room_list = Column::new().spacing(5);

        if self.creating_room || self.creating_space {
            let label = if self.creating_room {
                "Room Name"
            } else {
                "Space Name"
            };

            let mut name_input =
                text_input(label, &self.new_room_name).on_input(Message::NewRoomNameChanged);

            let is_empty = self.new_room_name.trim().is_empty();

            let mut create_btn = button::text(crate::fl!("create"));
            if !is_empty {
                if self.creating_room {
                    name_input =
                        name_input.on_submit(|_| Message::CreateRoom(self.new_room_name.clone()));
                    create_btn =
                        create_btn.on_press(Message::CreateRoom(self.new_room_name.clone()));
                } else {
                    name_input =
                        name_input.on_submit(|_| Message::CreateSpace(self.new_room_name.clone()));
                    create_btn =
                        create_btn.on_press(Message::CreateSpace(self.new_room_name.clone()));
                }
            }

            let create_btn_widget: Element<'_, Message> = if is_empty {
                tooltip(
                    create_btn,
                    text::body(format!(
                        "Enter a {} name to create",
                        if self.creating_room { "room" } else { "space" }
                    )),
                    Position::Top,
                )
                .into()
            } else {
                create_btn.into()
            };

            let cancel_msg = if self.creating_room {
                Message::ToggleCreateRoom
            } else {
                Message::ToggleCreateSpace
            };

            let create_ui = Column::new().spacing(5).push(name_input).push(
                Row::new()
                    .spacing(5)
                    .push(create_btn_widget)
                    .push(button::text(crate::fl!("cancel")).on_press(cancel_msg)),
            );

            room_list = room_list.push(container(create_ui).padding(5));
        }

        if let Some(selected_space) = &self.selected_space {
            let space_name = self
                .room_list
                .iter()
                .find(|r| r.id.as_ref() == selected_space.as_str())
                .and_then(|r| r.name.as_deref())
                .unwrap_or("Space");
            let space_header = Row::new()
                .align_y(Alignment::Center)
                .push(text::title3(space_name))
                .push(cosmic::widget::space().width(cosmic::iced::Length::Fill))
                .push(
                    button::icon(Named::new("emblem-system"))
                        .tooltip("Space Settings")
                        .on_press(Message::OpenSettings(crate::SettingsPanel::Space)),
                );
            room_list = room_list.push(container(space_header).padding(5));

            if !self.other_rooms.is_empty() {
                room_list = room_list.push(
                    container(text::title3(crate::fl!("joined-rooms")).size(14))
                        .padding([10, 5, 5, 5]),
                );
            }
        }

        for &room_idx in &self.filtered_room_list {
            let room = &self.room_list[room_idx];
            let name = room.name.as_deref().unwrap_or("Unknown Room");
            let room_id = room.id.clone();

            let mut room_content = Column::new().spacing(2);

            let mut header = Row::new().spacing(10).align_y(Alignment::Center);

            if let Some(avatar_url) = &room.avatar_url {
                if let Some(handle) = self.media_cache.get(avatar_url) {
                    header =
                        header.push(cosmic::widget::image(handle.clone()).width(24).height(24));
                } else {
                    header = header.push(
                        container(text::body("#"))
                            .width(24)
                            .height(24)
                            .align_x(Alignment::Center)
                            .align_y(Alignment::Center),
                    );
                }
            } else {
                header = header.push(
                    container(text::body("#"))
                        .width(24)
                        .height(24)
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                );
            }

            header = header.push(text::body(name));

            if let Some(unread_str) = &room.unread_count_str {
                header = header.push(text::body(unread_str.as_str()).size(12));
            }

            room_content = room_content.push(header);

            if let Some(last_msg) = &room.last_message {
                // Optimization: Avoid allocating a new String on every render frame
                room_content = room_content.push(text::body(last_msg.as_str()).size(12));
            }

            let btn = button::custom(
                container(room_content)
                    .padding(5)
                    .width(cosmic::iced::Length::Fill),
            )
            .on_press(Message::RoomSelected(room_id));

            room_list = room_list.push(btn.width(cosmic::iced::Fill));
        }

        if !self.filtered_other_rooms.is_empty() {
            room_list = room_list.push(
                container(text::title3(crate::fl!("other-rooms")).size(14)).padding([10, 5, 5, 5]),
            );

            for &idx in &self.filtered_other_rooms {
                let room = &self.other_rooms[idx];
                let name = room.name.as_deref().unwrap_or_else(|| {
                    let id = &room.id;
                    id.strip_prefix('!')
                        .and_then(|s| s.split(':').next())
                        .unwrap_or(id)
                });
                let room_id = room.id.clone();

                let mut room_content = Column::new().spacing(2);

                let mut header = Row::new().spacing(10).align_y(Alignment::Center);

                let mut has_avatar = false;
                if self.user_settings.invite_avatars_display_policy
                    && let Some(avatar_url) = &room.avatar_url
                    && let Some(handle) = self.media_cache.get(avatar_url)
                {
                    header = header.push(cosmic::widget::image(handle.clone()));
                    has_avatar = true;
                }

                if !has_avatar {
                    header = header.push(
                        container(text::body("#"))
                            .width(24)
                            .height(24)
                            .align_x(Alignment::Center)
                            .align_y(Alignment::Center),
                    );
                }

                header = header.push(text::body(name));

                if let Some(unread_str) = &room.unread_count_str {
                    header = header.push(text::body(unread_str.as_str()).size(12));
                }

                room_content = room_content.push(header);

                if let Some(last_msg) = &room.last_message {
                    room_content = room_content.push(text::body(last_msg.as_str()).size(12));
                }

                let btn = button::custom(
                    container(room_content)
                        .padding(5)
                        .width(cosmic::iced::Length::Fill),
                );

                let join_btn = button::text("Join").on_press(Message::JoinRoom(room_id.clone()));

                room_list = room_list.push(
                    Row::new()
                        .align_y(Alignment::Center)
                        .push(btn)
                        .push(container(join_btn).padding([0, 5])),
                );
            }
        }

        container(scrollable(room_list))
            .width(250)
            .padding(10)
            .into()
    }

    pub fn view_main_content(&self, status_text: String) -> Element<'_, Message> {
        let mut content = Column::new()
            .spacing(20)
            .padding(20)
            .width(cosmic::iced::Length::Fill);

        if matches!(
            self.sync_status,
            matrix::SyncStatus::Error(_) | matrix::SyncStatus::MissingSlidingSyncSupport
        ) {
            content = content.push(text::body(status_text).size(14));
        }

        if let Some(room_id) = &self.selected_room {
            let room_name = self
                .room_list
                .iter()
                .find(|r| &r.id == room_id)
                .and_then(|r| r.name.as_deref())
                .unwrap_or("Room");
            let is_in_call = self.user_id.as_ref().is_some_and(|uid| {
                if let Ok(user_id) = matrix_sdk::ruma::UserId::parse(uid) {
                    self.call_participants
                        .get(room_id)
                        .is_some_and(|p| p.contains(&user_id))
                } else {
                    false
                }
            });

            let call_participants = self.call_participants.get(room_id);
            let participant_count = call_participants.map_or(0, |p| p.len());

            let mut room_header = Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(text::title3(room_name));

            if participant_count > 0 {
                room_header = room_header.push(
                    container(
                        Row::new()
                            .spacing(5)
                            .align_y(Alignment::Center)
                            .push(cosmic::widget::icon::from_name("camera-video-symbolic").size(16))
                            .push(text::body(participant_count.to_string()).size(12)),
                    )
                    .padding([2, 5]),
                );
            }

            room_header = room_header
                .push(cosmic::widget::space().width(cosmic::iced::Length::Fill))
                .push(if is_in_call {
                    button::icon(Named::new("call-stop"))
                        .on_press(Message::LeaveCall)
                        .tooltip(crate::fl!("call-leave"))
                } else {
                    button::icon(Named::new("camera-web"))
                        .on_press(Message::JoinCall)
                        .tooltip(crate::fl!("call-join"))
                })
                .push(
                    button::icon(Named::new("emblem-system"))
                        .tooltip("Room Settings")
                        .on_press(Message::OpenSettings(crate::SettingsPanel::Room)),
                );
            content = content.push(room_header);

            content = content.push(if self.active_thread_root.is_some() {
                self.view_thread()
            } else {
                self.view_timeline()
            });

            if let Some(replying_to) = &self.replying_to {
                let mut snippet = replying_to
                    .item
                    .as_event()
                    .and_then(|ev| ev.content().as_message())
                    .map(|msg| msg.body().to_string())
                    .unwrap_or_default();
                if snippet.len() > 100 {
                    snippet.truncate(97);
                    snippet.push_str("...");
                }

                let reply_bar = Row::new()
                    .spacing(10)
                    .align_y(Alignment::Center)
                    .push(
                        text::body(crate::fl!(
                            "replying-to",
                            user = replying_to.sender_name.as_str()
                        ))
                        .size(12),
                    )
                    .push(text::body(snippet).size(12))
                    .push(cosmic::widget::space().width(cosmic::iced::Length::Fill))
                    .push(button::text(crate::fl!("cancel")).on_press(Message::CancelReply));
                content = content.push(container(reply_bar).padding(10));
            }

            let composer = if self.composer_is_preview {
                self.view_preview()
            } else {
                container(
                    text_input(crate::fl!("type-message"), &self.composer_text)
                        .on_input(Message::ComposerChanged)
                        .on_submit(|_| Message::SendMessage),
                )
                .padding(10)
                .into()
            };

            let mut attachments_view = Column::new().spacing(5);
            if !self.composer_attachments.is_empty() {
                attachments_view = attachments_view.push(text::body("Attachments:").size(12));
                for (i, path) in self.composer_attachments.iter().enumerate() {
                    let filename = path.file_name().unwrap_or_default().to_string_lossy();
                    let attachment_row = Row::new()
                        .spacing(10)
                        .align_y(Alignment::Center)
                        .push(text::body(filename).size(12))
                        .push(button::destructive("Remove").on_press(Message::RemoveAttachment(i)));
                    attachments_view = attachments_view.push(attachment_row);
                }
            }

            let is_empty =
                self.composer_text.trim().is_empty() && self.composer_attachments.is_empty();

            let mut send_btn = button::text(if self.active_thread_root.is_some() {
                crate::fl!("reply")
            } else {
                crate::fl!("send")
            });
            if !is_empty {
                send_btn = send_btn
                    .on_press(Message::SendMessage)
                    .tooltip(crate::fl!("tooltip-send"));
            }

            let send_btn_widget: Element<'_, Message> = if is_empty {
                tooltip(
                    send_btn,
                    text::body(crate::fl!("type-message-or-attach")),
                    Position::Top,
                )
                .into()
            } else {
                send_btn.into()
            };

            let controls = Row::new()
                .spacing(10)
                .push(
                    button::text(crate::fl!("attach"))
                        .on_press(Message::AddAttachment)
                        .tooltip(crate::fl!("tooltip-attach")),
                )
                .push(if self.composer_is_preview {
                    button::text(crate::fl!("edit"))
                        .on_press(Message::TogglePreview)
                        .tooltip(crate::fl!("tooltip-edit"))
                } else {
                    button::text(crate::fl!("preview"))
                        .on_press(Message::TogglePreview)
                        .tooltip(crate::fl!("tooltip-preview"))
                })
                .push(send_btn_widget);

            content = content.push(
                Column::new()
                    .spacing(10)
                    .push(attachments_view)
                    .push(composer)
                    .push(controls),
            );
        } else {
            let empty_state = container(
                Column::new()
                    .spacing(10)
                    .align_x(Alignment::Center)
                    .push(cosmic::widget::icon::from_name("chat-bubble-symbolic").size(64))
                    .push(text::title1("No room selected"))
                    .push(text::body(
                        "Select a room from the sidebar to start chatting.",
                    )),
            )
            .width(cosmic::iced::Length::Fill)
            .height(cosmic::iced::Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center);

            content = content.push(empty_state);
        }

        if let Some(error) = &self.error {
            let error_bar = container(
                Row::new()
                    .spacing(10)
                    .align_y(Alignment::Center)
                    .push(text::body(error))
                    .push(button::text(crate::fl!("dismiss")).on_press(Message::DismissError)),
            )
            .padding(10);
            content = content.push(error_bar);
        }

        content.into()
    }
}
