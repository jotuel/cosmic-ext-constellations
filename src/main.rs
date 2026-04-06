#![recursion_limit = "512"]

mod matrix;
mod ipc;

use cosmic::iced::{Alignment, Subscription};
use cosmic::widget::{button, column, row, scrollable, text, container, text_input};
use cosmic::{Application, Element, Task, Core, Action};
use anyhow::Result;
use matrix_sdk::ruma::OwnedRoomId;
use matrix_sdk::ruma::events::room::message::MessageType;
use matrix_sdk::ruma::events::room::MediaSource;
use matrix_sdk_ui::sync_service::State as SyncServiceState;
use std::path::PathBuf;
use std::sync::Arc;
use eyeball_im::Vector;
use url::Url;

struct Constellations {
    core: Core,
    matrix: Option<matrix::MatrixEngine>,
    sync_status: matrix::SyncStatus,
    room_list: Vec<matrix::RoomData>,
    selected_room: Option<String>,
    timeline_items: Vector<Arc<matrix::TimelineItem>>,
    composer_text: String,
    composer_is_preview: bool,
    user_id: Option<String>,
    media_cache: std::collections::HashMap<String, cosmic::iced::widget::image::Handle>,
    creating_room: bool,
    new_room_name: String,
    error: Option<String>,
    login_homeserver: String,
    login_username: String,
    login_password: String,
    is_logging_in: bool,
    is_oidc_logging_in: bool,
    is_initializing: bool,
    selected_space: Option<OwnedRoomId>,
}

#[derive(Debug, Clone)]
enum Message {
    Matrix(matrix::MatrixEvent),
    RoomSelected(String),
    EngineReady(Result<matrix::MatrixEngine, matrix::SyncError>),
    ComposerChanged(String),
    TogglePreview,
    SendMessage,
    MessageSent(Result<(), String>),
    LoadMore,
    LoadMoreFinished(Result<(), String>),
    UserReady(Option<String>, Result<(), matrix::SyncError>),
    FetchMedia(MediaSource),
    MediaFetched(String, Result<Vec<u8>, String>),
    CreateRoom(String),
    RoomCreated(Result<String, String>),
    NewRoomNameChanged(String),
    ToggleCreateRoom,
    DismissError,
    LoginHomeserverChanged(String),
    LoginUsernameChanged(String),
    LoginPasswordChanged(String),
    SubmitLogin,
    LoginFinished(Result<String, matrix::SyncError>),
    SelectSpace(Option<OwnedRoomId>),
    NoOp,
    SubmitOidcLogin,
    OidcLoginStarted(Result<Url, String>),
    OidcCallback(Url),
}

#[derive(Clone, Debug)]
struct MatrixEngineWrapper(matrix::MatrixEngine);

impl std::hash::Hash for MatrixEngineWrapper {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        "matrix-sync".hash(state);
    }
}

impl PartialEq for MatrixEngineWrapper {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for MatrixEngineWrapper {}

impl Constellations {
    fn view_timeline(&self) -> Element<'_, Message> {
        let mut timeline = column().spacing(10).width(cosmic::iced::Length::Fill);

        if self.selected_room.is_some() {
            timeline = timeline.push(
                container(
                    button::text("Load More")
                        .on_press(Message::LoadMore)
                )
                .width(cosmic::iced::Length::Fill)
                .align_x(Alignment::Center)
                .padding(10)
            );
        }

        for item in &self.timeline_items {
            if let Some(event) = item.as_event() {
                if let Some(message) = event.content().as_message() {
                    let sender = event.sender().to_string();
                    let (sender_name, avatar_url) = match event.sender_profile() {
                        matrix_sdk_ui::timeline::TimelineDetails::Ready(profile) => (
                            profile.display_name.as_deref().unwrap_or(&sender),
                            profile.avatar_url.clone(),
                        ),
                        _ => (&sender as &str, None),
                    };
                    let ts_millis = u64::from(event.timestamp().0);
                    let datetime = chrono::DateTime::from_timestamp_millis(ts_millis as i64).unwrap_or_default();
                    let timestamp = datetime.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string();

                    let mut reaction_row = row().spacing(5);
                    if let Some(reactions) = event.content().reactions() {
                        for (reaction, details) in reactions.iter() {
                            let count = details.len();
                            reaction_row = reaction_row.push(
                                container(text::body(format!("{} {}", reaction, count)).size(10))
                                    .padding(2)
                            );
                        }
                    }

                    let is_me = self.user_id.as_ref() == Some(&sender);

                    let mut sender_info = row().spacing(5).align_y(Alignment::Center);

                    if let Some(mxc_uri) = &avatar_url {
                        let mxc_url = mxc_uri.to_string();
                        if let Some(handle) = self.media_cache.get(&mxc_url) {
                            sender_info = sender_info.push(
                                cosmic::widget::image(handle.clone())
                                    .width(20)
                                    .height(20)
                            );
                        } else {
                            sender_info = sender_info.push(
                                container(text::body("👤").size(12))
                                    .padding(2)
                            );
                        }
                    } else {
                        sender_info = sender_info.push(
                            container(text::body("👤").size(12))
                                .padding(2)
                        );
                    }

                    sender_info = sender_info.push(text::body(sender_name.to_string()).size(10));
                    sender_info = sender_info.push(text::body(timestamp).size(10));

                    let mut bubble_col = column()
                        .spacing(2)
                        .push(sender_info);

                    match message.msgtype() {
                        MessageType::Image(image) => {
                            let mxc_url = match &image.source {
                                MediaSource::Plain(uri) => uri.to_string(),
                                MediaSource::Encrypted(file) => file.url.to_string(),
                            };
                            bubble_col = bubble_col.push(text::body(format!("📷 Image: {}", image.body)));
                            if let Some(handle) = self.media_cache.get(&mxc_url) {
                                bubble_col = bubble_col.push(
                                    cosmic::widget::image(handle.clone())
                                        .width(300)
                                );
                            } else {
                                bubble_col = bubble_col.push(
                                    button::text("Download Image")
                                        .on_press(Message::FetchMedia(image.source.clone()))
                                );
                            }
                        }
                        MessageType::File(file) => {
                            let mxc_url = match &file.source {
                                MediaSource::Plain(uri) => uri.to_string(),
                                MediaSource::Encrypted(file) => file.url.to_string(),
                            };
                            bubble_col = bubble_col.push(text::body(format!("📁 File: {}", file.body)));
                            if self.media_cache.contains_key(&mxc_url) {
                                bubble_col = bubble_col.push(text::body("[Downloaded]"));
                            } else {
                                bubble_col = bubble_col.push(
                                    button::text("Download File")
                                        .on_press(Message::FetchMedia(file.source.clone()))
                                );
                            }
                        }
                        _ => {
                            bubble_col = bubble_col.push(text::body(message.body().to_string()));
                        }
                    }

                    if event.content().reactions().is_some_and(|r| !r.is_empty()) {
                        bubble_col = bubble_col.push(reaction_row);
                    }

                    let bubble = container(bubble_col)
                        .padding(10)
                        .max_width(600);

                    let bubble_wrapper = container(bubble)
                        .width(cosmic::iced::Length::Fill)
                        .align_x(if is_me { Alignment::End } else { Alignment::Start });

                    timeline = timeline.push(bubble_wrapper);
                }
            } else if let Some(virt) = item.as_virtual() {
                if let matrix::VirtualTimelineItem::DateDivider(_date) = virt {
                    timeline = timeline.push(
                        container(text::body("--- Day Divider ---").size(12))
                            .width(cosmic::iced::Length::Fill)
                            .align_x(Alignment::Center)
                            .padding(10)
                    );
                }
            }
        }

        scrollable(timeline).height(cosmic::iced::Length::Fill).into()
    }

    fn view_preview(&self) -> Element<'_, Message> {
        let mut preview_col = column().spacing(10);

        for cached_row in &self.composer_preview_cache {
            let mut current_row = row().spacing(0).align_y(Alignment::Center);

            for item in cached_row {
                match item {
                    PreviewItem::Text(t) => {
                        current_row = current_row.push(text::body(t.clone()));
                    }
                    PreviewItem::Code(c) => {
                        current_row = current_row.push(
                            container(text::body(c.clone()))
                                .padding(2)
                        );
                    }
                    PreviewItem::Space => {
                        current_row = current_row.push(text::body(" "));
                    }
                }
            }

            preview_col = preview_col.push(current_row);
        }

        container(scrollable(preview_col).height(100))
            .padding(10)
            .into()
    }

    fn view_login(&self) -> Element<'_, Message> {
        let mut content = column()
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
            content = content.push(
                text::body(error)
            );
        }

        let homeserver_input = text_input("Homeserver", &self.login_homeserver);
        let username_input = text_input("Username", &self.login_username);
        let password_input = text_input("Password", &self.login_password).password();

        let (homeserver_input, username_input, password_input) = if self.is_logging_in || self.is_oidc_logging_in {
            (homeserver_input, username_input, password_input)
        } else {
            (
                homeserver_input.on_input(Message::LoginHomeserverChanged),
                username_input.on_input(Message::LoginUsernameChanged),
                password_input.on_input(Message::LoginPasswordChanged).on_submit(|_| Message::SubmitLogin),
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
            if !self.login_homeserver.is_empty() && !self.login_username.is_empty() && !self.login_password.is_empty() && !self.is_oidc_logging_in {
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

    fn update_title(&mut self) -> Task<Action<Message>> {
        let selected_room_name = self.selected_room.as_ref().and_then(|id| {
            self.room_list.iter().find(|r| &r.id == id).and_then(|r| r.name.as_deref())
        });

        let title = selected_room_name.unwrap_or("Constellations - Matrix Client");
        self.core.set_header_title(title.to_string());
        Task::none()
    }

    fn view_space_switcher(&self) -> Element<'_, Message> {
        let mut content = column().spacing(10).align_x(Alignment::Center);

        // Global icon (All Rooms)
        let is_global_selected = self.selected_space.is_none();
        let global_container = container(text::body("🌐").size(24))
            .padding(8)
            .align_x(Alignment::Center);

        let global_btn = if is_global_selected {
            button::custom(global_container)
        } else {
            button::custom(global_container)
                .on_press(Message::SelectSpace(None))
        };

        content = content.push(global_btn);

        for space in self.room_list.iter().filter(|r| r.is_space) {
            let space_id_str = space.id.clone();
            let space_id = match matrix_sdk::ruma::RoomId::parse(space_id_str) {
                Ok(id) => id.to_owned(),
                Err(_) => continue,
            };
            let is_selected = self.selected_space.as_ref() == Some(&space_id);

            let avatar: Element<'_, Message> = if let Some(url) = &space.avatar_url {
                if let Some(handle) = self.media_cache.get(url) {
                    cosmic::widget::image(handle.clone()).width(32).height(32).into()
                } else {
                    text::body(space.name.as_deref().unwrap_or("S").chars().next().unwrap_or('S').to_string()).size(24).into()
                }
            } else {
                text::body(space.name.as_deref().unwrap_or("S").chars().next().unwrap_or('S').to_string()).size(24).into()
            };

            let space_container = container(avatar)
                .padding(8)
                .align_x(Alignment::Center);

            let btn = if is_selected {
                button::custom(space_container)
            } else {
                button::custom(space_container)
                    .on_press(Message::SelectSpace(Some(space_id)))
            };

            content = content.push(btn);
        }

        container(scrollable(content))
            .width(60)
            .padding(5)
            .into()
    }
}

impl Application for Constellations {
    type Executor = cosmic::executor::Default;
    type Message = Message;
    type Flags = Option<String>;
    const APP_ID: &'static str = "fi.joonastuomi.CosmicExtConstellations";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, Task<Action<Self::Message>>) {
        let data_dir = dirs::data_dir()
            .map(|d| d.join("fi.joonastuomi.CosmicExtConstellations"));

        let mut tasks = Vec::new();
        tasks.push(Task::perform(async move {
            matrix::MatrixEngine::new(data_dir.expect("Error: No data dir")).await.map_err(matrix::SyncError::from)
        }, |res| Action::from(Message::EngineReady(res))));

        if let Some(uri) = flags {
            if let Ok(url) = Url::parse(&uri) {
                tasks.push(Task::done(Action::from(Message::OidcCallback(url))));
            }
        }

        let mut app = Constellations {
            core: core.clone(),
            matrix: None,
            sync_status: matrix::SyncStatus::Disconnected,
            room_list: Vec::new(),
            selected_room: None,
            timeline_items: Vector::new(),
            composer_text: String::new(),
            composer_is_preview: false,
            user_id: None,
            media_cache: std::collections::HashMap::new(),
            creating_room: false,
            new_room_name: String::new(),
            error: None,
            login_homeserver: "https://matrix.org".to_string(),
            login_username: String::new(),
            login_password: String::new(),
            is_logging_in: false,
            is_oidc_logging_in: false,
            is_initializing: true,
            selected_space: None,
        };

        let title_task = app.update_title();
        tasks.push(title_task);

        (app, Task::batch(tasks))
    }

    fn update(&mut self, message: Message) -> Task<Action<Self::Message>> {
        match message {
            Message::EngineReady(res) => {
                match res {
                    Ok(engine) => {
                        self.matrix = Some(engine.clone());
                        return Task::perform(async move {
                            let _ = engine.restore_session().await;
                            let user_id = engine.client().await.user_id().map(|u| u.to_string());
                            let sync_res = engine.start_sync().await;
                            (user_id, sync_res)
                        }, |(user_id, sync_res)| Action::from(Message::UserReady(user_id, sync_res)));
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to initialize Matrix engine: {}", e));
                        self.is_initializing = false;
                    }
                }
            }
            Message::UserReady(user_id, sync_res) => {
                self.user_id = user_id;
                self.is_initializing = false;
                let _ = self.update_title();
                match sync_res {
                    Ok(_) => {}
                    Err(matrix::SyncError::MissingSlidingSyncSupport) => {
                        self.sync_status = matrix::SyncStatus::MissingSlidingSyncSupport;
                    }
                    Err(e) => {
                        self.sync_status = matrix::SyncStatus::Error(e.to_string());
                    }
                }
            }
            Message::Matrix(event) => {
                match event {
                    matrix::MatrixEvent::SyncStatusChanged(status) => {
                        self.sync_status = status;
                    }
                    matrix::MatrixEvent::RoomDiff(diff) => {
                        match diff {
                            eyeball_im::VectorDiff::Insert { index, value } => {
                                if index <= self.room_list.len() {
                                    self.room_list.insert(index, value);
                                } else {
                                    self.room_list.push(value);
                                }
                            }
                            eyeball_im::VectorDiff::Remove { index } => {
                                if index < self.room_list.len() {
                                    self.room_list.remove(index);
                                }
                            }
                            eyeball_im::VectorDiff::Set { index, value } => {
                                if index < self.room_list.len() {
                                    self.room_list[index] = value;
                                }
                            }
                            eyeball_im::VectorDiff::Reset { values } => {
                                self.room_list = values.into_iter().collect();
                            }
                            eyeball_im::VectorDiff::PushBack { value } => {
                                self.room_list.push(value);
                            }
                            eyeball_im::VectorDiff::PushFront { value } => {
                                self.room_list.insert(0, value);
                            }
                            eyeball_im::VectorDiff::PopBack => {
                                self.room_list.pop();
                            }
                            eyeball_im::VectorDiff::PopFront => {
                                if !self.room_list.is_empty() {
                                    self.room_list.remove(0);
                                }
                            }
                            eyeball_im::VectorDiff::Clear => {
                                self.room_list.clear();
                            }
                            eyeball_im::VectorDiff::Append { values } => {
                                self.room_list.extend(values);
                            }
                            eyeball_im::VectorDiff::Truncate { length } => {
                                self.room_list.truncate(length);
                            }
                        }
                        let _ = self.update_title();
                    }
                    matrix::MatrixEvent::TimelineDiff(diff) => {
                        let mut tasks = Vec::new();
                        let mut check_item = |item: &std::sync::Arc<matrix::TimelineItem>| {
                            if let Some(event) = item.as_event() {
                                if let matrix_sdk_ui::timeline::TimelineDetails::Ready(profile) = event.sender_profile() {
                                    if let Some(avatar_url) = &profile.avatar_url {
                                        let url_str = avatar_url.to_string();
                                        if !self.media_cache.contains_key(&url_str) {
                                            if let Some(matrix) = &self.matrix {
                                                let matrix_clone = matrix.clone();
                                                let mxc_url = url_str.clone();
                                                let source = matrix_sdk::ruma::events::room::MediaSource::Plain(avatar_url.clone());
                                                tasks.push(cosmic::iced::Task::perform(async move {
                                                    matrix_clone.fetch_media(source).await.map_err(|e| e.to_string())
                                                }, move |res| Message::MediaFetched(mxc_url.clone(), res).into()));
                                            }
                                        }
                                    }
                                }
                            }
                        };

                        match &diff {
                            eyeball_im::VectorDiff::Insert { value, .. } => check_item(value),
                            eyeball_im::VectorDiff::Set { value, .. } => check_item(value),
                            eyeball_im::VectorDiff::PushBack { value } => check_item(value),
                            eyeball_im::VectorDiff::PushFront { value } => check_item(value),
                            eyeball_im::VectorDiff::Append { values } => values.iter().for_each(&mut check_item),
                            eyeball_im::VectorDiff::Reset { values } => values.iter().for_each(&mut check_item),
                            _ => {}
                        }

                        match diff {
                            eyeball_im::VectorDiff::Insert { index, value } => {
                                if index <= self.timeline_items.len() {
                                    self.timeline_items.insert(index, value);
                                } else {
                                    self.timeline_items.push_back(value);
                                }
                            }
                            eyeball_im::VectorDiff::Remove { index } => {
                                if index < self.timeline_items.len() {
                                    self.timeline_items.remove(index);
                                }
                            }
                            eyeball_im::VectorDiff::Set { index, value } => {
                                if index < self.timeline_items.len() {
                                    self.timeline_items.set(index, value);
                                }
                            }
                            eyeball_im::VectorDiff::Reset { values } => {
                                self.timeline_items = values;
                            }
                            eyeball_im::VectorDiff::PushBack { value } => {
                                self.timeline_items.push_back(value);
                            }
                            eyeball_im::VectorDiff::PushFront { value } => {
                                self.timeline_items.push_front(value);
                            }
                            eyeball_im::VectorDiff::PopBack => {
                                self.timeline_items.pop_back();
                            }
                            eyeball_im::VectorDiff::PopFront => {
                                self.timeline_items.pop_front();
                            }
                            eyeball_im::VectorDiff::Clear => {
                                self.timeline_items.clear();
                            }
                            eyeball_im::VectorDiff::Append { values } => {
                                self.timeline_items.extend(values);
                            }
                            eyeball_im::VectorDiff::Truncate { length } => {
                                self.timeline_items.truncate(length);
                            }
                        }

                        if !tasks.is_empty() {
                            return cosmic::iced::Task::batch(tasks);
                        }
                    }
                    matrix::MatrixEvent::TimelineReset => {
                        self.timeline_items.clear();
                    }
                    matrix::MatrixEvent::ReactionAdded { .. } => {
                        // For now, we don't do anything specific as reactions are handled via TimelineDiff
                    }
                }
            }
            Message::LoadMore => {
                if let (Some(matrix), Some(room_id)) = (&self.matrix, &self.selected_room) {
                    let matrix = matrix.clone();
                    let room_id = room_id.clone();
                    return Task::perform(async move {
                        matrix.paginate_backwards(&room_id, 20).await
                            .map_err(|e| e.to_string())
                    }, |res| Action::from(Message::LoadMoreFinished(res)));
                }
            }
            Message::LoadMoreFinished(res) => {
                if let Err(e) = res {
                    self.error = Some(format!("Failed to load more messages: {}", e));
                }
            }
            Message::RoomSelected(room_id) => {
                self.selected_room = Some(room_id);
                self.timeline_items.clear();
                return self.update_title();
            }
            Message::ComposerChanged(text) => {
                let parser = pulldown_cmark::Parser::new(&text);
                let mut rows = Vec::new();
                let mut current_row = Vec::new();
                for event in parser {
                    match event {
                        pulldown_cmark::Event::Start(tag) => if let pulldown_cmark::Tag::Heading { .. } = tag {
                            if !current_row.is_empty() {
                                rows.push(current_row.clone());
                                current_row.clear();
                            }
                        },
                        pulldown_cmark::Event::End(tag) => match tag {
                            pulldown_cmark::TagEnd::Paragraph | pulldown_cmark::TagEnd::Heading(_) => {
                                if !current_row.is_empty() {
                                    rows.push(current_row.clone());
                                    current_row.clear();
                                }
                            }
                            _ => {}
                        },
                        pulldown_cmark::Event::Text(t) => {
                            current_row.push(PreviewItem::Text(t.to_string()));
                        }
                        pulldown_cmark::Event::Code(c) => {
                            current_row.push(PreviewItem::Code(c.to_string()));
                        }
                        pulldown_cmark::Event::SoftBreak | pulldown_cmark::Event::HardBreak => {
                            current_row.push(PreviewItem::Space);
                        }
                        _ => {}
                    }
                }
                if !current_row.is_empty() {
                    rows.push(current_row);
                }
                self.composer_preview_cache = rows;
                self.composer_text = text;
            }
            Message::TogglePreview => {
                self.composer_is_preview = !self.composer_is_preview;
            }
            Message::SendMessage => {
                if let (Some(matrix), Some(room_id)) = (&self.matrix, &self.selected_room) {
                    let body = self.composer_text.clone();
                    if body.is_empty() {
                        return Task::none();
                    }

                    let html_body = matrix::markdown_to_html(&body);

                    let matrix = matrix.clone();
                    let room_id = room_id.clone();

                    return Task::perform(async move {
                        matrix.send_message(&room_id, body, Some(html_body)).await
                            .map_err(|e| e.to_string())
                    }, |res| Action::from(Message::MessageSent(res)));
                }
            }
            Message::MessageSent(res) => {
                match res {
                    Ok(_) => {
                        self.composer_text.clear();
                        self.composer_is_preview = false;
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to send message: {}", e));
                    }
                }
            }
            Message::FetchMedia(source) => {
                if let Some(matrix) = &self.matrix {
                    let matrix = matrix.clone();
                    let mxc_url = match &source {
                        MediaSource::Plain(uri) => uri.to_string(),
                        MediaSource::Encrypted(file) => file.url.to_string(),
                    };
                    return Task::perform(async move {
                        matrix.fetch_media(source).await
                            .map_err(|e| e.to_string())
                    }, move |res| Action::from(Message::MediaFetched(mxc_url, res)));
                }
            }
            Message::MediaFetched(mxc_url, res) => {
                match res {
                    Ok(data) => {
                        self.media_cache.insert(mxc_url, cosmic::iced::widget::image::Handle::from_bytes(data));
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to fetch media: {}", e));
                    }
                }
            }
            Message::DismissError => {
                self.error = None;
            }
            Message::ToggleCreateRoom => {
                self.creating_room = !self.creating_room;
                self.new_room_name.clear();
            }
            Message::NewRoomNameChanged(name) => {
                self.new_room_name = name;
            }
            Message::CreateRoom(name) => {
                if let Some(matrix) = &self.matrix {
                    let matrix = matrix.clone();
                    return Task::perform(async move {
                        matrix.create_room(&name).await
                            .map(|id| id.to_string())
                            .map_err(|e| e.to_string())
                    }, |res| Action::from(Message::RoomCreated(res)));
                }
            }
            Message::RoomCreated(res) => {
                match res {
                    Ok(room_id) => {
                        self.creating_room = false;
                        self.new_room_name.clear();
                        self.selected_room = Some(room_id);
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to create room: {}", e));
                    }
                }
            }
            Message::LoginHomeserverChanged(homeserver) => {
                self.login_homeserver = homeserver;
            }
            Message::LoginUsernameChanged(username) => {
                self.login_username = username;
            }
            Message::LoginPasswordChanged(password) => {
                self.login_password = password;
            }
            Message::SubmitLogin => {
                if let Some(matrix) = &self.matrix {
                    self.is_logging_in = true;
                    self.error = None;
                    self.sync_status = matrix::SyncStatus::Disconnected;
                    let matrix = matrix.clone();
                    let homeserver = self.login_homeserver.clone();
                    let username = self.login_username.clone();
                    let password = self.login_password.clone();
                    self.login_password.clear();

                    return Task::perform(async move {
                        matrix.login(&homeserver, &username, &password).await?;
                        let user_id = matrix.client().await.user_id()
                            .map(|u| u.to_string())
                            .ok_or_else(|| anyhow::anyhow!("Failed to get user ID after login"))?;
                        matrix.start_sync().await?;
                        Ok(user_id)
                    }, |res: Result<String, anyhow::Error>| Action::from(Message::LoginFinished(res.map_err(matrix::SyncError::from))));
                }
            }
            Message::LoginFinished(res) => {
                self.is_logging_in = false;
                self.is_oidc_logging_in = false;
                match res {
                    Ok(user_id) => {
                        self.user_id = Some(user_id);
                    }
                    Err(matrix::SyncError::MissingSlidingSyncSupport) => {
                        self.sync_status = matrix::SyncStatus::MissingSlidingSyncSupport;
                    }
                    Err(e) => {
                        self.error = Some(format!("Login failed: {}", e));
                    }
                }
            }
            Message::SelectSpace(space_id) => {
                self.selected_space = space_id.clone();
                if let Some(matrix) = &self.matrix {
                    let matrix = matrix.clone();
                    return Task::perform(async move {
                        let _ = matrix.update_room_list_filter(space_id).await;
                    }, |_| Action::from(Message::NoOp));
                }
            }
            Message::NoOp => {}
            Message::SubmitOidcLogin => {
                if let Some(matrix) = &self.matrix {
                    self.is_oidc_logging_in = true;
                    self.error = None;
                    let matrix = matrix.clone();
                    let homeserver = self.login_homeserver.clone();
                    return Task::perform(async move {
                        matrix.login_oidc(&homeserver).await.map_err(|e| e.to_string())
                    }, |res| Action::from(Message::OidcLoginStarted(res)));
                }
            }
            Message::OidcLoginStarted(res) => {
                match res {
                    Ok(url) => {
                        tracing::info!("Opening URL: {}", redact_url(&url));
                        let _ = open::that(url.as_str());
                        return Task::none();
                    }
                    Err(e) => {
                        self.is_oidc_logging_in = false;
                        self.error = Some(format!("OIDC login failed to start: {}", e));
                    }
                }
            }
            Message::OidcCallback(url) => {
                if let Some(matrix) = &self.matrix {
                    self.is_oidc_logging_in = true;
                    self.error = None;
                    let matrix = matrix.clone();
                    return Task::perform(async move {
                        matrix.complete_oidc_login(url).await?;
                        let user_id = matrix.client().await.user_id()
                            .map(|u| u.to_string())
                            .ok_or_else(|| anyhow::anyhow!("Failed to get user ID after OIDC login"))?;
                        matrix.start_sync().await?;
                        Ok(user_id)
                    }, |res: Result<String, anyhow::Error>| Action::from(Message::LoginFinished(res.map_err(matrix::SyncError::from))));
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        if self.is_initializing {
            return container(text::title1("Initializing..."))
                .width(cosmic::iced::Length::Fill)
                .height(cosmic::iced::Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .into();
        }

        if self.user_id.is_none() {
            return self.view_login();
        }

        let status_text = match &self.sync_status {
            matrix::SyncStatus::Disconnected => "Disconnected".to_string(),
            matrix::SyncStatus::Syncing => "Syncing...".to_string(),
            matrix::SyncStatus::Connected => "Connected".to_string(),
            matrix::SyncStatus::Error(e) => format!("⚠️ Sync Error: {}", e),
            matrix::SyncStatus::MissingSlidingSyncSupport => "Error: Your homeserver does not support Sliding Sync (MSC4186), which is required by Constellations.".to_string(),
        };

        let mut room_list = column().spacing(5);

        let create_room_ui = if self.creating_room {
            column().spacing(5)
                .push(text_input("Room Name", &self.new_room_name)
                    .on_input(Message::NewRoomNameChanged)
                    .on_submit(|_| Message::CreateRoom(self.new_room_name.clone())))
                .push(row().spacing(5)
                    .push(button::text("Create").on_press(Message::CreateRoom(self.new_room_name.clone())))
                    .push(button::text("Cancel").on_press(Message::ToggleCreateRoom)))
        } else {
            column().push(button::text("+ Create Room").on_press(Message::ToggleCreateRoom))
        };

        room_list = room_list.push(container(create_room_ui).padding(5));

        let filtered_rooms = if let Some(selected_space) = &self.selected_space {
            let mut rooms = Vec::new();
            if let Some(matrix) = &self.matrix {
                for room in self.room_list.iter().filter(|r| !r.is_space) {
                    if let Ok(room_id) = matrix_sdk::ruma::RoomId::parse(&room.id) {
                        if matrix.is_in_space_sync(&room_id, selected_space) {
                            rooms.push(room);
                        }
                    }
                }
            }
            rooms
        } else {
            self.room_list.iter().filter(|r| !r.is_space).collect()
        };

        for room in filtered_rooms {
            let name = room.name.as_deref().unwrap_or("Unknown Room");
            let room_id = room.id.clone();

            let mut room_content = column().spacing(2);

            let mut header = row().spacing(10).align_y(Alignment::Center);
            header = header.push(text::body("#"));
            header = header.push(text::body(name));

            if room.unread_count > 0 {
                header = header.push(text::body(format!("({})", room.unread_count)).size(12));
            }

            room_content = room_content.push(header);

            if let Some(last_msg) = &room.last_message {
                let mut snippet = last_msg.clone();
                if snippet.len() > 30 {
                    snippet.truncate(26);
                    snippet.push_str("...");
                }
                room_content = room_content.push(text::body(snippet).size(12));
            }

            let btn = button::custom(container(room_content).padding(5).width(cosmic::iced::Length::Fill))
                .on_press(Message::RoomSelected(room_id));

            room_list = room_list.push(btn);
        }

        let sidebar = container(scrollable(room_list))
            .width(250)
            .padding(10);

        let mut content = column()
            .spacing(20)
            .padding(20)
            .width(cosmic::iced::Length::Fill);

        if matches!(self.sync_status, matrix::SyncStatus::Error(_) | matrix::SyncStatus::MissingSlidingSyncSupport) {
            content = content.push(text::body(status_text).size(14));
        } else {
            content = content.push(text::body(format!("Status: {}", status_text)));
        }

        if self.selected_room.is_some() {
            content = content.push(self.view_timeline());

            let composer = if self.composer_is_preview {
                self.view_preview()
            } else {
                container(
                    text_input("Type a message...", &self.composer_text)
                        .on_input(Message::ComposerChanged)
                        .on_submit(|_| Message::SendMessage)
                )
                .padding(10)
                .into()
            };

            let controls = row()
                .spacing(10)
                .push(button::text(if self.composer_is_preview { "Edit" } else { "Preview" })
                    .on_press(Message::TogglePreview))
                .push(button::text("Send")
                    .on_press(Message::SendMessage));

            content = content.push(column().spacing(10).push(composer).push(controls));
        } else {
            content = content.align_x(Alignment::Center);
        }

        if let Some(error) = &self.error {
            let error_bar = container(
                row()
                    .spacing(10)
                    .align_y(Alignment::Center)
                    .push(text::body(error))
                    .push(button::text("Dismiss").on_press(Message::DismissError))
            )
            .padding(10);
            content = content.push(error_bar);
        }

        row()
            .push(self.view_space_switcher())
            .push(sidebar)
            .push(content)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let ipc_sub = Subscription::run_with(
            (),
            |_| {
                let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
                tokio::spawn(async move {
                    match ipc::start_server(tx).await {
                        Ok(_conn) => {
                            tracing::info!("IPC server started and waiting");
                        }
                        Err(e) => {
                            tracing::error!("Failed to start IPC server: {}", e);
                            return;
                        }
                    }
                    std::future::pending::<()>().await;
                });
                cosmic::iced::futures::stream::unfold(rx, |mut rx| async move {
                    loop {
                        if let Some(uri) = rx.recv().await {
                            if let Ok(url) = Url::parse(&uri) {
                                return Some((Message::OidcCallback(url), rx));
                            }
                        } else {
                            return None;
                        }
                    }
                })
            }
        );

        let matrix = match &self.matrix {
            Some(m) => m,
            None => return ipc_sub,
        };

        let sync_sub = Subscription::run_with(
            MatrixEngineWrapper(matrix.clone()),
            |wrapper| {
                let engine = wrapper.0.clone();
                let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

                let tx_status = tx.clone();
                let engine_status = engine.clone();
                tokio::spawn(async move {
                    let sync_service = loop {
                        if let Some(s) = engine_status.sync_service().await {
                            break s;
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    };

                    let mut status_stream = sync_service.state();
                    while let Some(status) = status_stream.next().await {
                        let sync_status = match status {
                            SyncServiceState::Idle => matrix::SyncStatus::Connected,
                            SyncServiceState::Running => matrix::SyncStatus::Syncing,
                            SyncServiceState::Terminated => matrix::SyncStatus::Disconnected,
                            SyncServiceState::Offline => matrix::SyncStatus::Disconnected,
                            SyncServiceState::Error(_) => {
                                matrix::SyncStatus::Error("Sync error encountered. This may be due to missing server support for Sliding Sync (MSC4186) or network issues.".to_string())
                            }
                        };
                        let _ = tx_status.send(Message::Matrix(matrix::MatrixEvent::SyncStatusChanged(sync_status)));
                    }
                });

                let tx_rooms = tx.clone();
                let engine_rooms = engine.clone();
                tokio::spawn(async move {
                    let room_list_service = loop {
                        if let Some(rls) = engine_rooms.room_list_service().await {
                            break rls;
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    };
                    let rooms = match room_list_service.all_rooms().await {
                        Ok(rooms) => rooms,
                        Err(_) => return,
                    };
                    let (stream, controller) = rooms.entries_with_dynamic_adapters(20);
                    let controller = Arc::new(controller);
                    engine_rooms.set_room_list_controller(controller.clone()).await;

                    use matrix_sdk_ui::room_list_service::filters;
                    controller.set_filter(Box::new(filters::new_filter_all(vec![])));

                    use cosmic::iced::futures::StreamExt;
                    let mut stream = Box::pin(stream);
                    while let Some(diffs) = stream.next().await {
                        for diff in diffs {
                            let room_diff = match diff {
                                eyeball_im::VectorDiff::Insert { index, value } => {
                                    get_room_data(&engine_rooms, &value).await
                                        .map(|data| eyeball_im::VectorDiff::Insert { index, value: data })
                                }
                                eyeball_im::VectorDiff::Remove { index } => {
                                    Some(eyeball_im::VectorDiff::Remove { index })
                                }
                                eyeball_im::VectorDiff::Set { index, value } => {
                                    get_room_data(&engine_rooms, &value).await
                                        .map(|data| eyeball_im::VectorDiff::Set { index, value: data })
                                }
                                eyeball_im::VectorDiff::Reset { values } => {
                                    let futures: Vec<_> = values.iter().map(|v| get_room_data(&engine_rooms, v)).collect();
                                    let new_values: Vec<_> = cosmic::iced::futures::future::join_all(futures)
                                        .await
                                        .into_iter()
                                        .flatten()
                                        .collect();
                                    Some(eyeball_im::VectorDiff::Reset { values: new_values.into() })
                                }
                                eyeball_im::VectorDiff::Append { values } => {
                                    let futures: Vec<_> = values.iter().map(|v| get_room_data(&engine_rooms, v)).collect();
                                    let new_values: Vec<_> = cosmic::iced::futures::future::join_all(futures)
                                        .await
                                        .into_iter()
                                        .flatten()
                                        .collect();
                                    Some(eyeball_im::VectorDiff::Append { values: new_values.into() })
                                }
                                eyeball_im::VectorDiff::Truncate { length } => {
                                    Some(eyeball_im::VectorDiff::Truncate { length })
                                }
                                eyeball_im::VectorDiff::PushBack { value } => {
                                    get_room_data(&engine_rooms, &value).await
                                        .map(|data| eyeball_im::VectorDiff::PushBack { value: data })
                                }
                                eyeball_im::VectorDiff::PushFront { value } => {
                                    get_room_data(&engine_rooms, &value).await
                                        .map(|data| eyeball_im::VectorDiff::PushFront { value: data })
                                }
                                eyeball_im::VectorDiff::PopBack => {
                                    Some(eyeball_im::VectorDiff::PopBack)
                                }
                                eyeball_im::VectorDiff::PopFront => {
                                    Some(eyeball_im::VectorDiff::PopFront)
                                }
                                eyeball_im::VectorDiff::Clear => {
                                    Some(eyeball_im::VectorDiff::Clear)
                                }
                            };

                            if let Some(diff) = room_diff {
                                let _ = tx_rooms.send(Message::Matrix(matrix::MatrixEvent::RoomDiff(diff)));
                            }
                        }
                    }
                });

                cosmic::iced::futures::stream::unfold(rx, |mut rx| async move {
                    rx.recv().await.map(|msg| (msg, rx))
                })
            },
        );

        if let Some(room_id) = self.selected_room.clone() {
            let timeline_sub = Subscription::run_with(
                (MatrixEngineWrapper(matrix.clone()), room_id.clone()),
                |(wrapper, room_id)| {
                    let engine = wrapper.0.clone();
                    let room_id = room_id.clone();
                    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

                    tokio::spawn(async move {
                        let timeline = match engine.timeline(&room_id).await {
                            Ok(t) => t,
                            Err(_) => return,
                        };

                        let (items, mut stream) = timeline.subscribe().await;
                        let _ = tx.send(Message::Matrix(matrix::MatrixEvent::TimelineReset));

                        for (index, item) in items.into_iter().enumerate() {
                            let _ = tx.send(Message::Matrix(matrix::MatrixEvent::TimelineDiff(
                                eyeball_im::VectorDiff::Insert { index, value: item }
                            )));
                        }

                        use cosmic::iced::futures::StreamExt;
                        while let Some(diff) = stream.next().await {
                            for d in diff {
                                let _ = tx.send(Message::Matrix(matrix::MatrixEvent::TimelineDiff(d)));
                            }
                        }
                    });

                    cosmic::iced::futures::stream::unfold(rx, |mut rx| async move {
                        rx.recv().await.map(|msg| (msg, rx))
                    })
                }
            );
            Subscription::batch([ipc_sub, sync_sub, timeline_sub])
        } else {
            Subscription::batch([ipc_sub, sync_sub])
        }
    }
}

async fn get_room_data(engine: &matrix::MatrixEngine, room: &matrix_sdk::Room) -> Option<matrix::RoomData> {
    let client = engine.client().await;
    let room_id = room.room_id();
    let room = client.get_room(room_id)?;

    engine.fetch_room_data(&room).await.ok()
}

fn redact_url(url: &Url) -> String {
    let mut redacted = url.clone();
    let pairs: Vec<(String, String)> = redacted.query_pairs().map(|(k, v)| (k.into_owned(), v.into_owned())).collect();
    redacted.set_query(None);
    for (k, mut v) in pairs {
        if k == "code" || k == "state" {
            v = "[REDACTED]".to_string();
        }
        redacted.query_pairs_mut().append_pair(&k, &v);
    }
    redacted.to_string()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("matrix_sdk=debug,matrix_sdk_ui=debug,cosmic_ext_constellations=debug")
        .with_writer(std::io::stderr)
        .init();
    let args: Vec<String> = std::env::args().collect();
    let uri = args.get(1).filter(|u| u.starts_with("fi.joonastuomi.CosmicExtConstellations://")).cloned();

    let rt = tokio::runtime::Runtime::new()?;
    let is_running = rt.block_on(async {
        let connection = match zbus::Connection::session().await {
            Ok(conn) => conn,
            Err(_) => return false,
        };
        let dbus = match zbus::fdo::DBusProxy::new(&connection).await {
            Ok(proxy) => proxy,
            Err(_) => return false,
        };
        dbus.name_has_owner(ipc::DBUS_NAME.try_into().unwrap()).await.unwrap_or(false)
    });

    if is_running {
        if let Some(uri) = uri {
            rt.block_on(async {
                if let Err(e) = ipc::call_handle_callback(uri).await {
                    tracing::error!("Failed to send URI to existing instance: {}", e);
                }
            });
        }
        tracing::info!("Another instance is already running, exiting.");
        return Ok(());
    }

    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();

    cosmic::app::run::<Constellations>(cosmic::app::Settings::default(), uri)?;
    Ok(())
}
