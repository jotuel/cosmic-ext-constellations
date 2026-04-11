use cosmic::{
    iced::{
        widget::{scrollable, tooltip},
        Alignment,
    },
    widget::{
        button, container, menu, text, text_input, tooltip::Position, Column, RcElementWrapper, Row,
    },
    Action, Element, Task,
};
use matrix_sdk::ruma::events::room::{message::MessageType, MediaSource};

use crate::{matrix, parse_markdown, Constellations, MenuAct, Message, PreviewEvent};

impl Constellations {
    pub fn view_timeline(&self) -> Element<'_, Message> {
        let mut timeline = Column::new().spacing(10).width(cosmic::iced::Length::Fill);

        if self.selected_room.is_some() {
            timeline = timeline.push(
                container(button::text("Load More").on_press(Message::LoadMore))
                    .width(cosmic::iced::Length::Fill)
                    .align_x(Alignment::Center)
                    .padding(10),
            );
        }

        for item in &self.timeline_items {
            if let Some(event) = item.item.as_event() {
                if let Some(message) = event.content().as_message() {
                    let _sender = &item.sender;
                    let sender_name = &item.sender_name;
                    let avatar_url = &item.avatar_url;
                    let timestamp = &item.timestamp;
                    let is_me = item.is_me;

                    let mut reaction_row = Row::new().spacing(5);
                    if let Some(reactions) = event.content().reactions() {
                        for (reaction, details) in reactions.iter() {
                            let count = details.len();
                            reaction_row = reaction_row.push(
                                container(text::body(format!("{} {}", reaction, count)).size(10))
                                    .padding(2),
                            );
                        }
                    }

                    let mut sender_info = Row::new().spacing(5).align_y(Alignment::Center);

                    if let Some(mxc_url) = avatar_url {
                        if let Some(handle) = self.media_cache.get(mxc_url) {
                            sender_info = sender_info
                                .push(cosmic::widget::image(handle.clone()).width(20).height(20));
                        } else {
                            sender_info = sender_info.push(
                                container(
                                    cosmic::widget::icon::from_name("avatar-default-symbolic")
                                        .size(12),
                                )
                                .padding(2),
                            );
                        }
                    } else {
                        sender_info = sender_info.push(
                            container(
                                cosmic::widget::icon::from_name("avatar-default-symbolic").size(12),
                            )
                            .padding(2),
                        );
                    }

                    // Optimization: Avoid allocating a new String on every render frame by using a reference
                    sender_info = sender_info.push(text::body(sender_name.as_str()).size(10));
                    sender_info = sender_info.push(text::body(timestamp.as_str()).size(10));

                    let mut bubble_col = Column::new()
                        .spacing(if self.app_settings.compact_mode { 0 } else { 2 })
                        .push(sender_info);

                    match message.msgtype() {
                        MessageType::Image(image) => {
                            let mxc_url = match &image.source {
                                MediaSource::Plain(uri) => uri.to_string(),
                                MediaSource::Encrypted(file) => file.url.to_string(),
                            };
                            bubble_col = bubble_col.push(
                                text::body(format!("📷 Image: {}", image.body)).size(
                                    if self.app_settings.compact_mode {
                                        12
                                    } else {
                                        14
                                    },
                                ),
                            );
                            if let Some(handle) = self.media_cache.get(&mxc_url) {
                                bubble_col =
                                    bubble_col.push(cosmic::widget::image(handle.clone()).width(
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
                        MessageType::File(file) => {
                            let mxc_url = match &file.source {
                                MediaSource::Plain(uri) => uri.to_string(),
                                MediaSource::Encrypted(file) => file.url.to_string(),
                            };
                            bubble_col = bubble_col.push(
                                text::body(format!("📁 File: {}", file.body)).size(
                                    if self.app_settings.compact_mode {
                                        12
                                    } else {
                                        14
                                    },
                                ),
                            );
                            if self.media_cache.contains_key(&mxc_url) {
                                bubble_col = bubble_col.push(text::body("[Downloaded]"));
                            } else {
                                bubble_col = bubble_col.push(
                                    button::text("Download File")
                                        .on_press(Message::FetchMedia(file.source.clone())),
                                );
                            }
                        }
                        _ => {
                            if self.app_settings.render_markdown {
                                let events = parse_markdown(message.body());
                                let mut md_col = Column::new()
                                    .spacing(if self.app_settings.compact_mode { 2 } else { 5 });
                                let mut current_row =
                                    Row::new().spacing(0).align_y(Alignment::Center);
                                let mut row_has_content = false;

                                for event in events {
                                    match event {
                                        PreviewEvent::StartHeading => {
                                            if row_has_content {
                                                md_col = md_col.push(current_row);
                                                current_row = Row::new()
                                                    .spacing(0)
                                                    .align_y(Alignment::Center);
                                                row_has_content = false;
                                            }
                                        }
                                        PreviewEvent::EndBlock => {
                                            if row_has_content {
                                                md_col = md_col.push(current_row);
                                                current_row = Row::new()
                                                    .spacing(0)
                                                    .align_y(Alignment::Center);
                                                row_has_content = false;
                                            }
                                        }
                                        PreviewEvent::Text(t) => {
                                            current_row = current_row.push(text::body(t).size(
                                                if self.app_settings.compact_mode {
                                                    12
                                                } else {
                                                    14
                                                },
                                            ));
                                            row_has_content = true;
                                        }
                                        PreviewEvent::Code(c) => {
                                            current_row = current_row.push(
                                                container(text::body(c).size(
                                                    if self.app_settings.compact_mode {
                                                        10
                                                    } else {
                                                        12
                                                    },
                                                ))
                                                .padding(2),
                                            );
                                            row_has_content = true;
                                        }
                                        PreviewEvent::Break => {
                                            if row_has_content {
                                                md_col = md_col.push(current_row);
                                                current_row = Row::new()
                                                    .spacing(0)
                                                    .align_y(Alignment::Center);
                                                row_has_content = false;
                                            }
                                        }
                                    }
                                }
                                if row_has_content {
                                    md_col = md_col.push(current_row);
                                }
                                bubble_col = bubble_col.push(md_col);
                            } else {
                                bubble_col = bubble_col.push(text::body(message.body()).size(
                                    if self.app_settings.compact_mode {
                                        12
                                    } else {
                                        14
                                    },
                                ));
                            }
                        }
                    }

                    if event.content().reactions().is_some_and(|r| !r.is_empty()) {
                        bubble_col = bubble_col.push(reaction_row);
                    }

                    let bubble = container(bubble_col)
                        .padding(if self.app_settings.compact_mode {
                            5
                        } else {
                            10
                        })
                        .max_width(600);

                    let bubble_wrapper = container(bubble)
                        .width(cosmic::iced::Length::Fill)
                        .align_x(if is_me {
                            Alignment::End
                        } else {
                            Alignment::Start
                        });

                    timeline = timeline.push(bubble_wrapper);
                }
            } else if let Some(matrix::VirtualTimelineItem::DateDivider(_date)) =
                item.item.as_virtual()
            {
                timeline = timeline.push(
                    container(text::body("--- Day Divider ---").size(12))
                        .width(cosmic::iced::Length::Fill)
                        .align_x(Alignment::Center)
                        .padding(10),
                );
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
        let mut content = Column::new()
            .spacing(10)
            .padding(20)
            .max_width(400)
            .align_x(Alignment::Center)
            .push(text::title1("Login to Matrix"));

        let status_error = match &self.sync_status {
            matrix::SyncStatus::Error(e) => Some(format!("⚠️ Sync Error: {}", e)),
            matrix::SyncStatus::MissingSlidingSyncSupport => Some("Error: Your homeserver does not support Sliding Sync (MSC4186), which is required by Constellations.".to_string()),
            _ => None,
        };

        if let Some(error) = status_error.or_else(|| self.error.clone()) {
            content = content.push(text::body(error));
        }

        let homeserver_input = text_input("Homeserver", &self.login_homeserver);
        let username_input = text_input("Username", &self.login_username);
        let password_input = text_input("Password", &self.login_password).password();

        let (homeserver_input, username_input, password_input) =
            if self.is_logging_in || self.is_oidc_logging_in {
                (homeserver_input, username_input, password_input)
            } else {
                (
                    homeserver_input.on_input(Message::LoginHomeserverChanged),
                    username_input.on_input(Message::LoginUsernameChanged),
                    password_input
                        .on_input(Message::LoginPasswordChanged)
                        .on_submit(|_| Message::SubmitLogin),
                )
            };

        content = content
            .push(homeserver_input)
            .push(username_input)
            .push(password_input);

        let login_button = if self.is_logging_in {
            button::text("Logging in...")
        } else {
            let mut btn = button::text("Login");
            if !self.login_homeserver.is_empty()
                && !self.login_username.is_empty()
                && !self.login_password.is_empty()
                && !self.is_oidc_logging_in
            {
                btn = btn.on_press(Message::SubmitLogin);
            }
            btn
        };

        let oidc_button = if self.is_oidc_logging_in {
            button::text("Waiting for browser...")
        } else {
            let mut btn = button::text("Login with OIDC");
            if !self.login_homeserver.is_empty() && !self.is_logging_in {
                btn = btn.on_press(Message::SubmitOidcLogin);
            }
            btn
        };

        content = content.push(login_button).push(oidc_button);

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

    pub fn view_space_switcher(&self) -> Element<'_, Message> {
        let mut content = Column::new().spacing(10).align_x(Alignment::Center);

        // Global icon (All Rooms)
        let is_global_selected = self.selected_space.is_none();
        let global_container = container(text::body("🌐").size(24))
            .padding(8)
            .align_x(Alignment::Center);

        let global_btn = if is_global_selected {
            button::custom(global_container)
        } else {
            button::custom(global_container).on_press(Message::SelectSpace(None))
        };

        let global_tooltip = tooltip(global_btn, text::body("All Rooms"), Position::Right);

        content = content.push(global_tooltip);

        for space in self.room_list.iter().filter(|r| r.is_space) {
            let space_id_str = space.id.clone();
            let space_id = match matrix_sdk::ruma::RoomId::parse(space_id_str) {
                Ok(id) => id.to_owned(),
                Err(_) => continue,
            };
            let is_selected = self.selected_space.as_ref() == Some(&space_id);

            let avatar: Element<'_, Message> = if let Some(url) = &space.avatar_url {
                if let Some(handle) = self.media_cache.get(url) {
                    cosmic::widget::image(handle.clone())
                        .width(32)
                        .height(32)
                        .into()
                } else {
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
                    .size(24)
                    .into()
                }
            } else {
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
                .size(24)
                .into()
            };

            let space_container = container(avatar).padding(8).align_x(Alignment::Center);

            let btn = if is_selected {
                button::custom(space_container)
            } else {
                button::custom(space_container).on_press(Message::SelectSpace(Some(space_id)))
            };

            let space_name = space.name.as_deref().unwrap_or("Unknown Space");
            let space_tooltip = tooltip(btn, text::body(space_name), Position::Right);

            content = content.push(space_tooltip);
        }

        let scrollable_spaces = scrollable(content).height(cosmic::iced::Length::Fill);

        let user_initial = self
            .user_id
            .as_deref()
            .and_then(|u| u.chars().nth(1))
            .unwrap_or('U')
            .to_ascii_uppercase()
            .to_string();
        let avatar = container(text::body(user_initial).size(24))
            .padding(8)
            .align_x(Alignment::Center);

        let user_btn = button::custom(avatar);
        let key_binds = std::collections::HashMap::new();

        let menu_tree = menu::Tree::with_children(
            RcElementWrapper::new(Element::from(user_btn)),
            menu::items(
                &key_binds,
                vec![
                    menu::Item::Button("App Settings", None, MenuAct::AppSettings),
                    menu::Item::Button("User Settings", None, MenuAct::UserSettings),
                    menu::Item::Button("Logout", None, MenuAct::Logout),
                ],
            ),
        );

        let user_menu = menu::bar(vec![menu_tree])
            .item_height(menu::ItemHeight::Dynamic(40))
            .item_width(menu::ItemWidth::Uniform(120))
            .spacing(4.0);

        let layout = Column::new()
            .push(scrollable_spaces)
            .push(user_menu)
            .align_x(Alignment::Center);

        container(layout).width(60).padding(5).into()
    }
}
