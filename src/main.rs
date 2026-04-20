#![recursion_limit = "512"]

mod handlers;
mod ipc;
mod matrix;
pub mod settings;
mod view;

use anyhow::Result;
use cosmic::iced::widget::image;
use cosmic::iced::widget::tooltip;
use cosmic::iced::{Alignment, Subscription};
use cosmic::widget::icon::Named;
use cosmic::widget::menu::action::MenuAction;
use cosmic::widget::{
    Column, RcElementWrapper, Row, button, container, menu, scrollable, text, text_input,
    tooltip::Position,
};
use cosmic::{Action, Application, Core, Element, Task};
use eyeball_im::Vector;
use matrix_sdk::ruma::OwnedRoomId;
use matrix_sdk::ruma::events::room::MediaSource;
use matrix_sdk_ui::sync_service::State as SyncServiceState;
use std::collections::HashMap;
use std::sync::Arc;
use url::Url;

// ⚡ Bolt Optimization:
// We cache the parsed Markdown structure in `PreviewEvent`s to avoid running
// `pulldown_cmark::Parser` on every single render frame inside `view_preview()`.
#[derive(Clone, Debug, PartialEq)]
pub enum PreviewEvent {
    StartHeading,
    EndBlock,
    Text(String),
    Code(String),
    Break,
}

pub fn parse_markdown(text: &str) -> Vec<PreviewEvent> {
    let mut events = Vec::new();
    let parser = pulldown_cmark::Parser::new(text);
    for event in parser {
        match event {
            pulldown_cmark::Event::Start(pulldown_cmark::Tag::Heading { .. }) => {
                events.push(PreviewEvent::StartHeading)
            }
            pulldown_cmark::Event::End(
                pulldown_cmark::TagEnd::Paragraph | pulldown_cmark::TagEnd::Heading(_),
            ) => events.push(PreviewEvent::EndBlock),
            pulldown_cmark::Event::Text(t) => events.push(PreviewEvent::Text(t.to_string())),
            pulldown_cmark::Event::Code(c) => events.push(PreviewEvent::Code(c.to_string())),
            pulldown_cmark::Event::SoftBreak | pulldown_cmark::Event::HardBreak => {
                events.push(PreviewEvent::Break)
            }
            _ => {}
        }
    }
    events
}

struct Constellations {
    core: Core,
    matrix: Option<matrix::MatrixEngine>,
    sync_status: matrix::SyncStatus,
    room_list: Vec<matrix::RoomData>,
    filtered_room_list: Vec<matrix::RoomData>,
    other_rooms: Vec<matrix::RoomData>,
    selected_room: Option<std::sync::Arc<str>>,
    timeline_items: Vector<ConstellationsItem>,
    composer_text: String,
    composer_preview_events: Vec<PreviewEvent>,
    composer_is_preview: bool,
    composer_attachments: Vec<std::path::PathBuf>,
    user_id: Option<String>,
    media_cache: HashMap<String, image::Handle>,
    creating_room: bool,
    creating_space: bool,
    new_room_name: String,
    error: Option<String>,
    login_homeserver: String,
    login_username: String,
    login_password: String,
    is_logging_in: bool,
    is_oidc_logging_in: bool,
    is_registering_mode: bool,
    is_registering: bool,
    is_initializing: bool,
    is_sync_indicator_active: bool,
    search_query: String,
    is_search_active: bool,
    active_reaction_picker: Option<matrix::TimelineEventItemId>,
    joined_room_ids: std::collections::HashSet<std::sync::Arc<str>>,
    selected_space: Option<OwnedRoomId>,
    current_settings_panel: Option<SettingsPanel>,
    user_settings: settings::user::State,
    room_settings: settings::room::State,
    space_settings: settings::space::State,
    app_settings: settings::app::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    Matrix(matrix::MatrixEvent),
    RoomSelected(std::sync::Arc<str>),
    EngineReady(Result<matrix::MatrixEngine, matrix::SyncError>),
    ComposerChanged(String),
    TogglePreview,
    SendMessage,
    MessageSent(Result<(), String>),
    AddAttachment,
    AttachmentsSelected(Vec<std::path::PathBuf>),
    RemoveAttachment(usize),
    AttachmentSent(std::path::PathBuf, Result<(), String>),
    ToggleReaction(matrix::TimelineEventItemId, String),
    ReactionToggled(Result<(), String>),
    OpenReactionPicker(Option<matrix::TimelineEventItemId>),
    LoadMore,
    LoadMoreFinished(Result<(), String>),
    UserReady(Option<String>, Result<(), matrix::SyncError>),
    FetchMedia(MediaSource),
    MediaFetched(String, Result<Vec<u8>, String>),
    MediaFetchedBatch(Vec<(String, Result<Vec<u8>, String>)>),
    CreateRoom(String),
    RoomCreated(Result<String, String>),
    CreateSpace(String),
    SpaceCreated(Result<String, String>),
    NewRoomNameChanged(String),
    ToggleCreateRoom,
    ToggleCreateSpace,
    DismissError,
    LoginHomeserverChanged(String),
    LoginUsernameChanged(String),
    LoginPasswordChanged(String),
    SubmitLogin,
    LoginFinished(Result<String, matrix::SyncError>),
    ToggleLoginMode,
    SubmitRegister,
    RegisterFinished(Result<String, matrix::SyncError>),
    SelectSpace(Option<std::sync::Arc<str>>),
    SpaceChildrenFetched(OwnedRoomId, Result<Vec<matrix::RoomData>, String>),
    NoOp,
    SubmitOidcLogin,
    OidcLoginStarted(Result<Url, String>),
    OidcCallback(Url),
    JoinRoom(std::sync::Arc<str>),
    RoomJoined(Result<OwnedRoomId, String>),
    Logout,
    LogoutFinished,
    OpenSettings(SettingsPanel),
    CloseSettings,
    UserSettings(settings::user::Message),
    RoomSettings(settings::room::Message),
    SpaceSettings(settings::space::Message),
    AppSettings(settings::app::Message),
    AppSettingChanged,
    ToggleSearch,
    SearchQueryChanged(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum SettingsPanel {
    App,
    User,
    Room,
    Space,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum MenuAct {
    AppSettings,
    UserSettings,
    Logout,
    CreateRoom,
    CreateSpace,
}

impl MenuAction for MenuAct {
    type Message = Message;
    fn message(&self) -> Self::Message {
        match self {
            MenuAct::AppSettings => Message::OpenSettings(SettingsPanel::App),
            MenuAct::UserSettings => Message::OpenSettings(SettingsPanel::User),
            MenuAct::Logout => Message::Logout,
            MenuAct::CreateRoom => Message::ToggleCreateRoom,
            MenuAct::CreateSpace => Message::ToggleCreateSpace,
        }
    }
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

trait ApplyVectorDiffExt<T> {
    fn apply_diff(&mut self, diff: eyeball_im::VectorDiff<T>);
}

impl<T: Clone> ApplyVectorDiffExt<T> for Vec<T> {
    fn apply_diff(&mut self, diff: eyeball_im::VectorDiff<T>) {
        match diff {
            eyeball_im::VectorDiff::Insert { index, value } => {
                if index <= self.len() {
                    self.insert(index, value);
                } else {
                    self.push(value);
                }
            }
            eyeball_im::VectorDiff::Remove { index } => {
                if index < self.len() {
                    self.remove(index);
                }
            }
            eyeball_im::VectorDiff::Set { index, value } => {
                if index < self.len() {
                    self[index] = value;
                }
            }
            eyeball_im::VectorDiff::Reset { values } => {
                *self = values.into_iter().collect();
            }
            eyeball_im::VectorDiff::PushBack { value } => {
                self.push(value);
            }
            eyeball_im::VectorDiff::PushFront { value } => {
                self.insert(0, value);
            }
            eyeball_im::VectorDiff::PopBack => {
                self.pop();
            }
            eyeball_im::VectorDiff::PopFront => {
                if !self.is_empty() {
                    self.remove(0);
                }
            }
            eyeball_im::VectorDiff::Clear => {
                self.clear();
            }
            eyeball_im::VectorDiff::Append { values } => {
                self.extend(values);
            }
            eyeball_im::VectorDiff::Truncate { length } => {
                self.truncate(length);
            }
        }
    }
}

#[derive(Clone, Debug)]
struct ConstellationsItem {
    pub item: Arc<matrix::TimelineItem>,
    pub sender_name: String,
    pub avatar_url: Option<String>,
    pub timestamp: String,
    pub is_me: bool,
    pub markdown: Vec<PreviewEvent>,
}

impl ConstellationsItem {
    fn new(item: Arc<matrix::TimelineItem>, user_id: Option<&str>) -> Self {
        let mut sender_name = String::new();
        let mut avatar_url = None;
        let mut timestamp = String::new();
        let mut is_me = false;
        let mut markdown = Vec::new();

        if let Some(event) = item.as_event() {
            if let Some(msg) = event.content().as_message() {
                markdown = crate::parse_markdown(msg.body());
            }
            let (name, url) = match event.sender_profile() {
                matrix_sdk_ui::timeline::TimelineDetails::Ready(profile) => (
                    profile
                        .display_name
                        .as_deref()
                        .unwrap_or(event.sender().as_ref())
                        .to_string(),
                    profile.avatar_url.as_ref().map(|uri| uri.to_string()),
                ),
                _ => (event.sender().to_string(), None),
            };
            sender_name = name;
            avatar_url = url;

            let ts_millis = u64::from(event.timestamp().0);
            let datetime =
                chrono::DateTime::from_timestamp_millis(ts_millis as i64).unwrap_or_default();
            timestamp = datetime
                .with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();

            is_me = user_id == Some(&sender_name);
        }

        Self {
            item,
            sender_name,
            avatar_url,
            timestamp,
            is_me,
            markdown,
        }
    }
}

impl<T: Clone> ApplyVectorDiffExt<T> for eyeball_im::Vector<T> {
    fn apply_diff(&mut self, diff: eyeball_im::VectorDiff<T>) {
        match diff {
            eyeball_im::VectorDiff::Insert { index, value } => {
                if index <= self.len() {
                    self.insert(index, value);
                } else {
                    self.push_back(value);
                }
            }
            eyeball_im::VectorDiff::Remove { index } => {
                if index < self.len() {
                    self.remove(index);
                }
            }
            eyeball_im::VectorDiff::Set { index, value } => {
                if index < self.len() {
                    self.set(index, value);
                }
            }
            eyeball_im::VectorDiff::Reset { values } => {
                *self = values;
            }
            eyeball_im::VectorDiff::PushBack { value } => {
                self.push_back(value);
            }
            eyeball_im::VectorDiff::PushFront { value } => {
                self.push_front(value);
            }
            eyeball_im::VectorDiff::PopBack => {
                self.pop_back();
            }
            eyeball_im::VectorDiff::PopFront => {
                self.pop_front();
            }
            eyeball_im::VectorDiff::Clear => {
                self.clear();
            }
            eyeball_im::VectorDiff::Append { values } => {
                self.extend(values);
            }
            eyeball_im::VectorDiff::Truncate { length } => {
                self.truncate(length);
            }
        }
    }
}

impl Constellations {
    pub fn update_filtered_rooms(&mut self) {
        let search_query = self.search_query.to_lowercase();
        let filter_by_search = |room: &matrix::RoomData| {
            if search_query.is_empty() {
                true
            } else {
                room.name
                    .as_ref()
                    .map(|n| n.to_lowercase().contains(&search_query))
                    .unwrap_or(false)
                    || room.id.to_lowercase().contains(&search_query)
            }
        };

        if let Some(selected_space) = &self.selected_space {
            let mut rooms = Vec::new();
            if let Some(matrix) = &self.matrix {
                for room in self.room_list.iter().filter(|r| !r.is_space) {
                    if let Ok(room_id) = matrix_sdk::ruma::RoomId::parse(&*room.id)
                        && matrix.is_in_space_sync(&room_id, selected_space)
                            && filter_by_search(room)
                        {
                            rooms.push(room.clone());
                        }
                }
            }
            self.filtered_room_list = rooms;

            // Re-filter other_rooms to remove any that we've now joined
            self.other_rooms
                .retain(|r| !self.joined_room_ids.contains(r.id.as_ref()));
        } else {
            self.filtered_room_list = self
                .room_list
                .iter()
                .filter(|r| !r.is_space && filter_by_search(r))
                .cloned()
                .collect();
            self.other_rooms.clear();
        }
    }

    fn ipc_subscription(&self) -> Subscription<Message> {
        Subscription::run_with((), |_| {
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
        })
    }

    fn sync_subscription(&self, matrix: &matrix::MatrixEngine) -> Subscription<Message> {
        Subscription::run_with(MatrixEngineWrapper(matrix.clone()), |wrapper| {
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
                    let _ = tx_status.send(Message::Matrix(
                        matrix::MatrixEvent::SyncStatusChanged(sync_status),
                    ));
                }
            });

            let tx_indicator = tx.clone();
            let engine_indicator = engine.clone();
            tokio::spawn(async move {
                let room_list_service = loop {
                    if let Some(rls) = engine_indicator.room_list_service().await {
                        break rls;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                };

                let mut indicator_stream = Box::pin(room_list_service.sync_indicator(
                    std::time::Duration::from_millis(500),
                    std::time::Duration::from_millis(500),
                ));
                use cosmic::iced::futures::StreamExt;
                while let Some(indicator) = indicator_stream.next().await {
                    let show = indicator == matrix_sdk_ui::room_list_service::SyncIndicator::Show;
                    let _ = tx_indicator.send(Message::Matrix(
                        matrix::MatrixEvent::SyncIndicatorChanged(show),
                    ));
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
                engine_rooms
                    .set_room_list_controller(controller.clone())
                    .await;

                use matrix_sdk_ui::room_list_service::filters;
                controller.set_filter(Box::new(filters::new_filter_all(vec![])));

                use cosmic::iced::futures::StreamExt;
                let mut stream = Box::pin(stream);
                while let Some(diffs) = stream.next().await {
                    for diff in diffs {
                        let room_diff = match diff {
                            eyeball_im::VectorDiff::Insert { index, value } => {
                                get_room_data(&engine_rooms, value.room_id())
                                    .await
                                    .map(|data| eyeball_im::VectorDiff::Insert {
                                        index,
                                        value: data,
                                    })
                            }
                            eyeball_im::VectorDiff::Remove { index } => {
                                Some(eyeball_im::VectorDiff::Remove { index })
                            }
                            eyeball_im::VectorDiff::Set { index, value } => {
                                get_room_data(&engine_rooms, value.room_id())
                                    .await
                                    .map(|data| eyeball_im::VectorDiff::Set { index, value: data })
                            }
                            eyeball_im::VectorDiff::Reset { values } => {
                                let futures: Vec<_> = values
                                    .iter()
                                    .map(|v| get_room_data(&engine_rooms, v.room_id()))
                                    .collect();
                                let new_values: Vec<_> =
                                    cosmic::iced::futures::future::join_all(futures)
                                        .await
                                        .into_iter()
                                        .flatten()
                                        .collect();
                                Some(eyeball_im::VectorDiff::Reset {
                                    values: new_values.into(),
                                })
                            }
                            eyeball_im::VectorDiff::Append { values } => {
                                let futures: Vec<_> = values
                                    .iter()
                                    .map(|v| get_room_data(&engine_rooms, v.room_id()))
                                    .collect();
                                let new_values: Vec<_> =
                                    cosmic::iced::futures::future::join_all(futures)
                                        .await
                                        .into_iter()
                                        .flatten()
                                        .collect();
                                Some(eyeball_im::VectorDiff::Append {
                                    values: new_values.into(),
                                })
                            }
                            eyeball_im::VectorDiff::Truncate { length } => {
                                Some(eyeball_im::VectorDiff::Truncate { length })
                            }
                            eyeball_im::VectorDiff::PushBack { value } => {
                                get_room_data(&engine_rooms, value.room_id())
                                    .await
                                    .map(|data| eyeball_im::VectorDiff::PushBack { value: data })
                            }
                            eyeball_im::VectorDiff::PushFront { value } => {
                                get_room_data(&engine_rooms, value.room_id())
                                    .await
                                    .map(|data| eyeball_im::VectorDiff::PushFront { value: data })
                            }
                            eyeball_im::VectorDiff::PopBack => {
                                Some(eyeball_im::VectorDiff::PopBack)
                            }
                            eyeball_im::VectorDiff::PopFront => {
                                Some(eyeball_im::VectorDiff::PopFront)
                            }
                            eyeball_im::VectorDiff::Clear => Some(eyeball_im::VectorDiff::Clear),
                        };

                        if let Some(diff) = room_diff {
                            let _ =
                                tx_rooms.send(Message::Matrix(matrix::MatrixEvent::RoomDiff(diff)));
                        }
                    }
                }
            });

            cosmic::iced::futures::stream::unfold(rx, |mut rx| async move {
                rx.recv().await.map(|msg| (msg, rx))
            })
        })
    }

    fn timeline_subscription(
        &self,
        matrix: &matrix::MatrixEngine,
        room_id: Arc<str>,
    ) -> Subscription<Message> {
        Subscription::run_with(
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
                            eyeball_im::VectorDiff::Insert { index, value: item },
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
            },
        )
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

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        let mut start = Vec::new();

        if self.is_search_active {
            let row = Row::new()
                .align_y(Alignment::Center)
                .push(
                    button::icon(Named::new("edit-find-symbolic")).on_press(Message::ToggleSearch),
                )
                .push(
                    text_input("Search...", &self.search_query)
                        .on_input(Message::SearchQueryChanged)
                        .width(200.0),
                );
            start.push(row.into());
        } else {
            start.push(
                button::icon(Named::new("edit-find-symbolic"))
                    .on_press(Message::ToggleSearch)
                    .into(),
            );
        }

        start
    }

    fn header_end(&self) -> Vec<Element<'_, Self::Message>> {
        let mut end = Vec::new();

        if self.user_id.is_some() {
            let user_btn = button::icon(Named::new("user-available-symbolic"));
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

            end.push(user_menu.into());
        }

        end
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, Task<Action<Self::Message>>) {
        let data_dir = dirs::data_dir().map(|d| d.join("fi.joonastuomi.Constellations"));

        let mut tasks = Vec::new();
        tasks.push(Task::perform(
            async move {
                let dir = data_dir.ok_or_else(|| {
                    matrix::SyncError::from(anyhow::anyhow!("No standard data directory found"))
                })?;
                matrix::MatrixEngine::new(dir)
                    .await
                    .map_err(matrix::SyncError::from)
            },
            |res| Action::from(Message::EngineReady(res)),
        ));

        if let Some(uri) = flags
            && let Ok(url) = Url::parse(&uri) {
                tasks.push(Task::done(Action::from(Message::OidcCallback(url))));
            }

        let mut app = Constellations {
            core: core.clone(),
            matrix: None,
            sync_status: matrix::SyncStatus::Disconnected,
            room_list: Vec::new(),
            filtered_room_list: Vec::new(),
            other_rooms: Vec::new(),
            selected_room: None,
            timeline_items: Vector::new(),
            composer_text: String::new(),
            composer_preview_events: Vec::new(),
            composer_is_preview: false,
            composer_attachments: Vec::new(),
            user_id: None,
            media_cache: HashMap::new(),
            creating_room: false,
            creating_space: false,
            new_room_name: String::new(),
            error: None,
            login_homeserver: "https://matrix.org".to_string(),
            login_username: String::new(),
            login_password: String::new(),
            is_logging_in: false,
            is_oidc_logging_in: false,
            is_registering_mode: false,
            is_registering: false,
            is_initializing: true,
            is_sync_indicator_active: false,
            search_query: String::new(),
            is_search_active: false,
            active_reaction_picker: None,
            joined_room_ids: std::collections::HashSet::new(),
            selected_space: None,
            current_settings_panel: None,
            user_settings: Default::default(),
            room_settings: Default::default(),
            space_settings: Default::default(),
            app_settings: Default::default(),
        };

        let title_task = app.update_title();
        tasks.push(title_task);

        (app, Task::batch(tasks))
    }

    fn context_drawer(
        &self,
    ) -> Option<cosmic::app::context_drawer::ContextDrawer<'_, Self::Message>> {
        if let Some(panel) = &self.current_settings_panel {
            let title = match panel {
                SettingsPanel::App => "App Settings",
                SettingsPanel::User => "User Settings",
                SettingsPanel::Room => "Room Settings",
                SettingsPanel::Space => "Space Settings",
            };

            let panel_content = match panel {
                SettingsPanel::User => self.user_settings.view().map(Message::UserSettings),
                SettingsPanel::Room => self.room_settings.view().map(Message::RoomSettings),
                SettingsPanel::Space => self.space_settings.view().map(Message::SpaceSettings),
                SettingsPanel::App => self.app_settings.view().map(Message::AppSettings),
            };

            Some(
                cosmic::app::context_drawer::context_drawer(panel_content, Message::CloseSettings)
                    .title(title),
            )
        } else {
            None
        }
    }

    fn update(&mut self, message: Message) -> Task<Action<Self::Message>> {
        match message {
            Message::EngineReady(res) => self.handle_engine_ready(res),
            Message::UserReady(user_id, sync_res) => self.handle_user_ready(user_id, sync_res),

            Message::Matrix(event) => self.handle_matrix_event(event),
            Message::LoadMore => self.handle_load_more(),
            Message::LoadMoreFinished(res) => {
                if let Err(e) = res {
                    self.error = Some(format!("Failed to load more messages: {}", e));
                }
                Task::none()
            }
            Message::RoomSelected(room_id) => {
                self.selected_room = Some(room_id.clone());
                self.timeline_items.clear();
                self.update_title()
            }
            Message::ComposerChanged(text) => {
                self.composer_preview_events = parse_markdown(&text);
                self.composer_text = text;

                if self.app_settings.send_typing_notifications
                    && let Some(matrix) = &self.matrix
                        && let Some(room_id) = &self.selected_room {
                            let matrix = matrix.clone();
                            let room_id = room_id.clone();
                            let typing = !self.composer_text.is_empty();
                            return Task::perform(
                                async move {
                                    let _ = matrix.typing_notice(&room_id, typing).await;
                                },
                                |_| Action::from(Message::NoOp),
                            );
                        }

                Task::none()
            }
            Message::TogglePreview => {
                self.composer_is_preview = !self.composer_is_preview;
                Task::none()
            }
            Message::SendMessage => self.handle_send_message(),
            Message::MessageSent(res) => {
                match res {
                    Ok(_) => {
                        self.composer_text.clear();
                        self.composer_preview_events.clear();
                        self.composer_is_preview = false;
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to send message: {}", e));
                    }
                }
                Task::none()
            }
            Message::AddAttachment => self.handle_add_attachment(),
            Message::AttachmentsSelected(paths) => {
                for path in paths {
                    if !self.composer_attachments.contains(&path) {
                        self.composer_attachments.push(path);
                    }
                }
                Task::none()
            }
            Message::RemoveAttachment(index) => {
                if index < self.composer_attachments.len() {
                    self.composer_attachments.remove(index);
                }
                Task::none()
            }
            Message::AttachmentSent(path, res) => {
                match res {
                    Ok(_) => {
                        // Successfully sent, could remove from ui if we were tracking it per-message
                    }
                    Err(e) => {
                        self.error = Some(format!(
                            "Failed to send attachment {}: {}",
                            path.display(),
                            e
                        ));
                    }
                }
                Task::none()
            }
            Message::OpenReactionPicker(item_id) => {
                self.active_reaction_picker = item_id;
                Task::none()
            }
            Message::ToggleReaction(item_id, key) => {
                self.active_reaction_picker = None;
                if let (Some(matrix), Some(room_id)) = (&self.matrix, &self.selected_room) {
                    let matrix_clone = matrix.clone();
                    let room_id_clone = room_id.clone();
                    return Task::perform(
                        async move {
                            matrix_clone
                                .toggle_reaction(&room_id_clone, &item_id, &key)
                                .await
                                .map_err(|e| e.to_string())
                        },
                        |res| Message::ReactionToggled(res).into(),
                    );
                }
                Task::none()
            }
            Message::ReactionToggled(res) => {
                if let Err(e) = res {
                    self.error = Some(format!("Failed to toggle reaction: {}", e));
                }
                Task::none()
            }
            Message::FetchMedia(source) => self.handle_fetch_media(source),
            Message::MediaFetched(mxc_url, res) => self.handle_media_fetched(mxc_url, res),
            Message::MediaFetchedBatch(batch) => self.handle_media_fetched_batch(batch),
            Message::DismissError => {
                self.error = None;
                Task::none()
            }
            Message::ToggleCreateRoom => {
                self.creating_room = !self.creating_room;
                self.creating_space = false;
                self.new_room_name.clear();
                Task::none()
            }
            Message::ToggleCreateSpace => {
                self.creating_space = !self.creating_space;
                self.creating_room = false;
                self.new_room_name.clear();
                Task::none()
            }
            Message::NewRoomNameChanged(name) => {
                self.new_room_name = name;
                Task::none()
            }
            Message::CreateRoom(name) => self.handle_create_room(name),
            Message::RoomCreated(res) => {
                match res {
                    Ok(room_id) => {
                        self.creating_room = false;
                        self.new_room_name.clear();
                        self.selected_room = Some(room_id.as_str().into());
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to create room: {}", e));
                    }
                }
                Task::none()
            }
            Message::CreateSpace(name) => self.handle_create_space(name),
            Message::SpaceCreated(res) => {
                match res {
                    Ok(space_id) => {
                        self.creating_space = false;
                        self.new_room_name.clear();
                        if let Ok(rid) = space_id.as_str().try_into() {
                            return self.handle_select_space(Some(rid));
                        }
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to create space: {}", e));
                    }
                }
                Task::none()
            }
            Message::LoginHomeserverChanged(homeserver) => {
                self.login_homeserver = homeserver;
                Task::none()
            }
            Message::LoginUsernameChanged(username) => {
                self.login_username = username;
                Task::none()
            }
            Message::LoginPasswordChanged(password) => {
                self.login_password = password;
                Task::none()
            }
            Message::SubmitLogin => self.handle_submit_login(),
            Message::LoginFinished(res) => self.handle_login_finished(res),
            Message::ToggleLoginMode => self.handle_toggle_login_mode(),
            Message::SubmitRegister => self.handle_submit_register(),
            Message::RegisterFinished(res) => self.handle_register_finished(res),
            Message::SelectSpace(space_id) => {
                let parsed_id = space_id.and_then(|id| matrix_sdk::ruma::RoomId::parse(&*id).ok());
                self.handle_select_space(parsed_id)
            }
            Message::SpaceChildrenFetched(space_id, res) => {
                self.handle_space_children_fetched(space_id, res)
            }
            Message::NoOp => Task::none(),
            Message::SubmitOidcLogin => self.handle_submit_oidc_login(),
            Message::OidcLoginStarted(res) => self.handle_oidc_login_started(res),
            Message::OidcCallback(url) => self.handle_oidc_callback(url),
            Message::JoinRoom(room_id) => {
                if let Some(matrix) = &self.matrix {
                    let matrix = matrix.clone();
                    return Task::perform(
                        async move {
                            let rid = matrix_sdk::ruma::RoomId::parse(&*room_id)
                                .map_err(|e| e.to_string())?;
                            matrix
                                .join_room(&rid)
                                .await
                                .map(|_| rid)
                                .map_err(|e| e.to_string())
                        },
                        |res| Message::RoomJoined(res).into(),
                    );
                }
                Task::none()
            }
            Message::RoomJoined(res) => {
                match res {
                    Ok(room_id) => {
                        self.selected_room = Some(room_id.as_str().into());
                        // Refresh both lists
                        self.update_filtered_rooms();
                        if let (Some(matrix), Some(space_id)) = (&self.matrix, &self.selected_space)
                        {
                            let matrix = matrix.clone();
                            let sid = space_id.clone();
                            let sid_clone = sid.clone();
                            return Task::perform(
                                async move {
                                    matrix
                                        .get_space_children(sid_clone.as_str())
                                        .await
                                        .map_err(|e| e.to_string())
                                },
                                move |res| Message::SpaceChildrenFetched(sid, res).into(),
                            );
                        }
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to join room: {}", e));
                    }
                }
                Task::none()
            }
            Message::Logout => self.handle_logout(),
            Message::LogoutFinished => self.handle_logout_finished(),
            Message::OpenSettings(panel) => {
                self.current_settings_panel = Some(panel.clone());
                self.core.set_show_context(true);
                if panel == SettingsPanel::User {
                    return self
                        .user_settings
                        .update(settings::user::Message::LoadProfile, &self.matrix);
                } else if panel == SettingsPanel::Room {
                    if let Some(room_id) = &self.selected_room {
                        return self.room_settings.update(
                            settings::room::Message::LoadRoom(room_id.clone()),
                            &self.matrix,
                        );
                    }
                } else if panel == SettingsPanel::Space
                    && let Some(space_id) = &self.selected_space {
                        return self.space_settings.update(
                            settings::space::Message::LoadSpace(space_id.to_string()),
                            &self.matrix,
                        );
                    }
                Task::none()
            }
            Message::CloseSettings => {
                self.current_settings_panel = None;
                self.core.set_show_context(false);
                Task::none()
            }
            Message::UserSettings(msg) => self.user_settings.update(msg, &self.matrix),
            Message::RoomSettings(msg) => self.room_settings.update(msg, &self.matrix),
            Message::SpaceSettings(msg) => self.space_settings.update(msg, &self.matrix),
            Message::AppSettings(msg) => match msg {
                settings::app::Message::ClearCache => {
                    self.media_cache.clear();
                    Task::none()
                }
                _ => self.app_settings.update(msg),
            },
            Message::AppSettingChanged => Task::none(),
            Message::ToggleSearch => {
                self.is_search_active = !self.is_search_active;
                if !self.is_search_active {
                    self.search_query.clear();
                    self.update_filtered_rooms();
                }
                Task::none()
            }
            Message::SearchQueryChanged(query) => {
                self.search_query = query;
                self.update_filtered_rooms();
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        if self.is_initializing {
            let content = Column::new()
                .push(
                    cosmic::widget::svg(cosmic::widget::svg::Handle::from_memory(include_bytes!(
                        "../res/const.svg"
                    )))
                    .width(cosmic::iced::Length::Fixed(128.0))
                    .height(cosmic::iced::Length::Fixed(128.0)),
                )
                .push(cosmic::widget::progress_bar::indeterminate_circular())
                .spacing(32)
                .align_x(Alignment::Center);

            return container(content)
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

            let mut create_btn = button::text("Create");
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
                    .push(button::text("Cancel").on_press(cancel_msg)),
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
                        .on_press(Message::OpenSettings(SettingsPanel::Space)),
                );
            room_list = room_list.push(container(space_header).padding(5));

            if !self.other_rooms.is_empty() {
                room_list = room_list
                    .push(container(text::title3("Joined Rooms").size(14)).padding([10, 5, 5, 5]));
            }
        }

        for room in &self.filtered_room_list {
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

            room_list = room_list.push(btn);
        }

        if !self.other_rooms.is_empty() {
            room_list = room_list
                .push(container(text::title3("Other Rooms").size(14)).padding([10, 5, 5, 5]));

            for room in &self.other_rooms {
                let name = room.name.as_deref().unwrap_or_else(|| {
                    let id = &room.id;
                    id.strip_prefix('!')
                        .and_then(|s| s.split(':').next())
                        .unwrap_or(id)
                });
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

        let sidebar = container(scrollable(room_list)).width(250).padding(10);

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
            let room_header = Row::new()
                .align_y(Alignment::Center)
                .push(text::title3(room_name))
                .push(cosmic::widget::space().width(cosmic::iced::Length::Fill))
                .push(
                    button::icon(Named::new("emblem-system"))
                        .tooltip("Room Settings")
                        .on_press(Message::OpenSettings(SettingsPanel::Room)),
                );
            content = content.push(room_header);

            content = content.push(self.view_timeline());

            let composer = if self.composer_is_preview {
                self.view_preview()
            } else {
                container(
                    text_input("Type a message...", &self.composer_text)
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
                        .push(button::text("Remove").on_press(Message::RemoveAttachment(i)));
                    attachments_view = attachments_view.push(attachment_row);
                }
            }

            let is_empty =
                self.composer_text.trim().is_empty() && self.composer_attachments.is_empty();

            let mut send_btn = button::text("Send");
            if !is_empty {
                send_btn = send_btn.on_press(Message::SendMessage);
            }

            let send_btn_widget: Element<'_, Message> = if is_empty {
                tooltip(
                    send_btn,
                    text::body("Type a message or attach a file to send"),
                    Position::Top,
                )
                .into()
            } else {
                send_btn.into()
            };

            let controls = Row::new()
                .spacing(10)
                .push(button::text("Attach").on_press(Message::AddAttachment))
                .push(
                    button::text(if self.composer_is_preview {
                        "Edit"
                    } else {
                        "Preview"
                    })
                    .on_press(Message::TogglePreview),
                )
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
                    .push(button::text("Dismiss").on_press(Message::DismissError)),
            )
            .padding(10);
            content = content.push(error_bar);
        }

        let main_view = Row::new()
            .push(self.view_space_switcher())
            .push(sidebar)
            .push(content);

        if self.app_settings.show_sync_indicator && self.is_sync_indicator_active {
            let sync_widget: Element<'_, Message> = match self.sync_status {
                matrix::SyncStatus::Syncing => {
                    container(cosmic::widget::progress_bar::indeterminate_circular().size(24.0))
                        .into()
                }
                matrix::SyncStatus::Connected => {
                    container(cosmic::widget::icon::from_name("network-idle-symbolic").size(24))
                        .into()
                }
                matrix::SyncStatus::Disconnected => {
                    container(cosmic::widget::icon::from_name("network-offline-symbolic").size(24))
                        .into()
                }
                matrix::SyncStatus::Error(_) | matrix::SyncStatus::MissingSlidingSyncSupport => {
                    container(cosmic::widget::icon::from_name("network-error-symbolic").size(24))
                        .into()
                }
            };

            let sync_overlay = container(sync_widget)
                .padding(20)
                .width(cosmic::iced::Length::Fill)
                .height(cosmic::iced::Length::Fill)
                .align_x(Alignment::End)
                .align_y(Alignment::End);

            return cosmic::iced::widget::stack![main_view, sync_overlay].into();
        }

        main_view.into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let ipc_sub = self.ipc_subscription();

        let matrix = match &self.matrix {
            Some(m) => m,
            None => return ipc_sub,
        };

        let sync_sub = self.sync_subscription(matrix);

        if let Some(room_id) = self.selected_room.clone() {
            let timeline_sub = self.timeline_subscription(matrix, room_id);
            Subscription::batch([ipc_sub, sync_sub, timeline_sub])
        } else {
            Subscription::batch([ipc_sub, sync_sub])
        }
    }
}

async fn get_room_data(
    engine: &matrix::MatrixEngine,
    room_id: &matrix_sdk::ruma::RoomId,
) -> Option<matrix::RoomData> {
    let client = engine.client().await;
    let room = client.get_room(room_id)?;

    engine.fetch_room_data(&room).await.ok()
}

fn redact_url(url: &Url) -> String {
    let mut redacted = url.clone();
    let pairs: Vec<(String, String)> = redacted
        .query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();
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
    let uri = args
        .get(1)
        .filter(|u| u.starts_with("fi.joonastuomi.CosmicExtConstellations://"))
        .cloned();

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
        dbus.name_has_owner(ipc::DBUS_NAME.try_into().unwrap())
            .await
            .unwrap_or(false)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown_paragraph() {
        let text = "This is a simple paragraph.";
        let events = parse_markdown(text);
        assert_eq!(
            events,
            vec![
                PreviewEvent::Text("This is a simple paragraph.".to_string()),
                PreviewEvent::EndBlock
            ]
        );
    }

    #[test]
    fn test_parse_markdown_heading() {
        let text = "# Heading 1\nSome text.";
        let events = parse_markdown(text);
        assert_eq!(
            events,
            vec![
                PreviewEvent::StartHeading,
                PreviewEvent::Text("Heading 1".to_string()),
                PreviewEvent::EndBlock,
                PreviewEvent::Text("Some text.".to_string()),
                PreviewEvent::EndBlock,
            ]
        );
    }

    #[test]
    fn test_parse_markdown_code() {
        let text = "Here is `some code` inline.";
        let events = parse_markdown(text);
        assert_eq!(
            events,
            vec![
                PreviewEvent::Text("Here is ".to_string()),
                PreviewEvent::Code("some code".to_string()),
                PreviewEvent::Text(" inline.".to_string()),
                PreviewEvent::EndBlock,
            ]
        );
    }

    #[test]
    fn test_parse_markdown_breaks() {
        let text = "Line 1\nLine 2  \nLine 3";
        let events = parse_markdown(text);
        assert_eq!(
            events,
            vec![
                PreviewEvent::Text("Line 1".to_string()),
                PreviewEvent::Break,
                PreviewEvent::Text("Line 2".to_string()),
                PreviewEvent::Break,
                PreviewEvent::Text("Line 3".to_string()),
                PreviewEvent::EndBlock,
            ]
        );
    }

    #[test]
    fn test_parse_markdown_ignored_formatting() {
        // Italics and bold should just emit Text events without wrapping them in special formatting events.
        let text = "Some **bold** and *italic* text.";
        let events = parse_markdown(text);
        assert_eq!(
            events,
            vec![
                PreviewEvent::Text("Some ".to_string()),
                PreviewEvent::Text("bold".to_string()),
                PreviewEvent::Text(" and ".to_string()),
                PreviewEvent::Text("italic".to_string()),
                PreviewEvent::Text(" text.".to_string()),
                PreviewEvent::EndBlock,
            ]
        );
    }

    #[tokio::test]
    async fn test_get_room_data_not_found() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let engine = matrix::MatrixEngine::new(tmp_dir.path().to_path_buf())
            .await
            .unwrap();

        let room_id = matrix_sdk::ruma::RoomId::parse("!nonexistent:example.com").unwrap();

        let result = get_room_data(&engine, &room_id).await;

        assert!(result.is_none());
    }
}
