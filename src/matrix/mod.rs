use anyhow::{Context, Result};
use eyeball_im::VectorDiff;
use matrix_sdk::authentication::matrix::MatrixSession;
use matrix_sdk::media::MediaFormat;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::ruma::events::room::MediaSource;
use matrix_sdk::ruma::events::space::child::SpaceChildEventContent;
use matrix_sdk::ruma::events::space::parent::SpaceParentEventContent;
use matrix_sdk::ruma::events::{AnySyncMessageLikeEvent, AnySyncTimelineEvent, SyncStateEvent};
use matrix_sdk::{
    ruma::{room::RoomType, OwnedDeviceId, OwnedRoomId, RoomId, UserId},
    Client, Room, SessionChange, SessionTokens,
};
pub use matrix_sdk_ui::room_list_service::{RoomListDynamicEntriesController, RoomListService};
use matrix_sdk_ui::sync_service::SyncService;
pub use matrix_sdk_ui::timeline::{RoomExt, Timeline, TimelineItem, VirtualTimelineItem};
use oo7::Keyring;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{error, info};
use url::Url;

const BACKOFF_INITIAL: u64 = 2;
const BACKOFF_MAX: u64 = 60;
const BACKOFF_RESET_THRESHOLD: u64 = 30;
const OIDC_CALLBACK_URL: &str = "fi.joonastuomi.CosmicExtConstellations://callback";
const OIDC_CLIENT_ID: &str = "fi.joonastuomi.CosmicExtConstellations";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncStatus {
    Disconnected,
    Syncing,
    Connected,
    Error(String),
    MissingSlidingSyncSupport,
}

#[derive(Error, Debug, Clone)]
pub enum SyncError {
    #[error("Sliding Sync (MSC4186) is not supported by the homeserver")]
    MissingSlidingSyncSupport,
    #[error("Matrix error: {0}")]
    Matrix(String),
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("Error: {0}")]
    Anyhow(String),
    #[error("{0}")]
    Generic(String),
}

impl From<matrix_sdk::Error> for SyncError {
    fn from(e: matrix_sdk::Error) -> Self {
        Self::Matrix(e.to_string())
    }
}

impl From<matrix_sdk::HttpError> for SyncError {
    fn from(e: matrix_sdk::HttpError) -> Self {
        Self::Http(e.to_string())
    }
}

impl From<anyhow::Error> for SyncError {
    fn from(e: anyhow::Error) -> Self {
        Self::Anyhow(e.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomData {
    pub id: String,
    pub name: Option<String>,
    pub last_message: Option<String>,
    pub unread_count: u32,
    pub avatar_url: Option<String>,
    pub room_type: Option<RoomType>,
    pub is_space: bool,
    pub parent_space_id: Option<String>,
}

pub type RoomListDiff = VectorDiff<RoomData>;
pub type TimelineDiff<T> = VectorDiff<Arc<T>>;

#[derive(Debug, Default, Clone)]
pub struct SpaceHierarchy {
    /// Maps a space ID to its children (rooms or sub-spaces)
    pub children: HashMap<OwnedRoomId, Vec<OwnedRoomId>>,
    /// Maps a room/space ID to its parent spaces
    pub parents: HashMap<OwnedRoomId, Vec<OwnedRoomId>>,
    /// Set of all known space IDs
    pub known_spaces: HashSet<OwnedRoomId>,
}

impl SpaceHierarchy {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_space(&mut self, space_id: OwnedRoomId) {
        self.known_spaces.insert(space_id);
    }

    pub fn is_known_space(&self, room_id: &RoomId) -> bool {
        self.known_spaces.contains(room_id)
    }

    pub fn add_child(&mut self, space_id: OwnedRoomId, child_id: OwnedRoomId) {
        self.add_space(space_id.clone());
        let children = self.children.entry(space_id.clone()).or_default();
        if !children.contains(&child_id) {
            children.push(child_id.clone());
        }

        let parents = self.parents.entry(child_id).or_default();
        if !parents.contains(&space_id) {
            parents.push(space_id);
        }
    }

    pub fn remove_child(&mut self, space_id: &RoomId, child_id: &RoomId) {
        if let Some(children) = self.children.get_mut(space_id) {
            children.retain(|id| id != child_id);
        }
        if let Some(parents) = self.parents.get_mut(child_id) {
            parents.retain(|id| id != space_id);
        }
    }

    pub fn is_in_space(&self, room_id: &RoomId, space_id: &RoomId) -> bool {
        let mut visited = HashSet::new();
        self.is_in_space_recursive(room_id, space_id, &mut visited)
    }

    // ⚡ Bolt Optimization:
    // Use `HashSet<&RoomId>` to prevent string allocations (`.to_owned()`) during recursive traversal
    fn is_in_space_recursive<'a>(
        &'a self,
        current_id: &'a RoomId,
        target_space_id: &RoomId,
        visited: &mut HashSet<&'a RoomId>,
    ) -> bool {
        if current_id == target_space_id {
            return true;
        }

        if !visited.insert(current_id) {
            return false;
        }

        if let Some(parents) = self.parents.get(current_id) {
            for parent in parents {
                if self.is_in_space_recursive(parent, target_space_id, visited) {
                    return true;
                }
            }
        }

        false
    }
}

#[derive(Debug, Clone)]
pub enum MatrixEvent {
    SyncStatusChanged(SyncStatus),
    SyncIndicatorChanged(bool),
    RoomDiff(RoomListDiff),
    TimelineDiff(TimelineDiff<TimelineItem>),
    TimelineReset,
    ReactionAdded {
        room_id: String,
        event_id: String,
        reaction: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct SessionData {
    homeserver: String,
    user_id: String,
    access_token: String,
    refresh_token: Option<String>,
    id_token: Option<String>,
    device_id: String,
    #[serde(default)]
    is_oidc: bool,
}

#[derive(Clone, Debug)]
pub struct MatrixEngine {
    inner: Arc<RwLock<MatrixEngineInner>>,
}

struct MatrixEngineInner {
    client: Client,
    sync_service: Option<Arc<SyncService>>,
    room_list_service: Option<Arc<RoomListService>>,
    room_list_controller: Option<Arc<RoomListDynamicEntriesController>>,
    timelines: HashMap<OwnedRoomId, Arc<Timeline>>,
    data_dir: PathBuf,
    sync_handle: Option<tokio::task::JoinHandle<()>>,
    space_hierarchy: SpaceHierarchy,
    oidc_client: Option<Client>,
    session_change_handle: Option<tokio::task::JoinHandle<()>>,
}

impl std::fmt::Debug for MatrixEngineInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MatrixEngineInner")
            .field("client", &self.client)
            .field(
                "sync_service",
                &self.sync_service.as_ref().map(|_| "SyncService"),
            )
            .field(
                "room_list_service",
                &self.room_list_service.as_ref().map(|_| "RoomListService"),
            )
            .field(
                "room_list_controller",
                &self
                    .room_list_controller
                    .as_ref()
                    .map(|_| "RoomListDynamicEntriesController"),
            )
            .field("timelines", &self.timelines.keys())
            .field("data_dir", &self.data_dir)
            .field(
                "sync_handle",
                &self.sync_handle.as_ref().map(|_| "JoinHandle"),
            )
            .field("space_hierarchy", &self.space_hierarchy)
            .field("oidc_client", &self.oidc_client.as_ref().map(|_| "Client"))
            .field(
                "session_change_handle",
                &self.session_change_handle.as_ref().map(|_| "JoinHandle"),
            )
            .finish()
    }
}

struct Backoff {
    current: u64,
    max: u64,
}

impl Backoff {
    fn new(initial: u64, max: u64) -> Self {
        Self {
            current: initial,
            max,
        }
    }

    fn next(&mut self) -> u64 {
        let next = self.current;
        self.current = (self.current * 2).min(self.max);
        next
    }
}

impl MatrixEngine {
    pub async fn new(data_dir: PathBuf) -> Result<Self> {
        let client = Self::setup_client(data_dir.clone(), "https://matrix.org").await?;

        client
            .oauth()
            .restore_registered_client(matrix_sdk::authentication::oauth::ClientId::new(
                OIDC_CLIENT_ID.to_string(),
            ));

        let inner = MatrixEngineInner {
            client: client.clone(),
            sync_service: None,
            room_list_service: None,
            room_list_controller: None,
            timelines: HashMap::new(),
            data_dir,
            sync_handle: None,
            space_hierarchy: SpaceHierarchy::new(),
            oidc_client: None,
            session_change_handle: None,
        };

        let engine = Self {
            inner: Arc::new(RwLock::new(inner)),
        };
        engine.setup_event_handlers(&client);
        engine.spawn_session_change_handler(client).await;
        Ok(engine)
    }

    async fn save_session_to_keyring(session_data: &SessionData) -> Result<()> {
        let keyring = Keyring::new().await?;
        let mut attributes = HashMap::new();
        attributes.insert("app_id", "fi.joonastuomi.CosmicExtConstellations");
        attributes.insert("type", "matrix-session");

        let secret = serde_json::to_vec(session_data)?;

        keyring
            .create_item("Constellations Matrix Session", &attributes, &secret, true)
            .await?;
        Ok(())
    }

    async fn spawn_session_change_handler(&self, client: Client) {
        let mut subscriber = client.subscribe_to_session_changes();
        let homeserver = client.homeserver().to_string();

        let handle = tokio::spawn(async move {
            loop {
                match subscriber.recv().await {
                    Ok(change) => match change {
                        SessionChange::TokensRefreshed => {
                            info!("Session tokens refreshed, updating keyring...");

                            if let Some(session) = client.oauth().user_session() {
                                let session_data = SessionData {
                                    homeserver: homeserver.clone(),
                                    user_id: session.meta.user_id.to_string(),
                                    access_token: session.tokens.access_token.to_string(),
                                    refresh_token: session.tokens.refresh_token.clone(),
                                    id_token: None,
                                    device_id: session.meta.device_id.to_string(),
                                    is_oidc: true,
                                };

                                if let Err(e) = Self::save_session_to_keyring(&session_data).await {
                                    error!("Failed to update session in keyring: {}", e);
                                } else {
                                    info!("Successfully updated session in keyring.");
                                }
                            } else if let Some(session) = client.matrix_auth().session() {
                                let session_data = SessionData {
                                    homeserver: homeserver.clone(),
                                    user_id: session.meta.user_id.to_string(),
                                    access_token: session.tokens.access_token.to_string(),
                                    refresh_token: session.tokens.refresh_token.clone(),
                                    id_token: None,
                                    device_id: session.meta.device_id.to_string(),
                                    is_oidc: false,
                                };

                                if let Err(e) = Self::save_session_to_keyring(&session_data).await {
                                    error!("Failed to update session in keyring: {}", e);
                                } else {
                                    info!("Successfully updated session in keyring.");
                                }
                            } else {
                                error!("Session tokens refreshed but client has no session!");
                            }
                        }
                        SessionChange::UnknownToken { .. } => {
                            error!("Session token is no longer valid!");
                        }
                    },
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        error!("Session change subscriber lagged by {} messages", n);
                        continue;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        info!("Session change subscriber closed.");
                        break;
                    }
                }
            }
        });

        let mut inner = self.inner.write().await;
        if let Some(old_handle) = inner.session_change_handle.take() {
            old_handle.abort();
        }
        inner.session_change_handle = Some(handle);
        drop(inner);
    }

    fn setup_event_handlers(&self, client: &Client) {
        let inner_clone = self.inner.clone();
        client.add_event_handler(
            move |event: SyncStateEvent<SpaceChildEventContent>, room: Room| {
                let inner = inner_clone.clone();
                async move {
                    let space_id = room.room_id().to_owned();
                    let child_id = match RoomId::parse(event.state_key()) {
                        Ok(id) => id,
                        Err(_) => return,
                    };

                    let mut inner_write = inner.write().await;
                    match event {
                        SyncStateEvent::Original(ev) => {
                            if ev.content.via.is_empty() {
                                inner_write
                                    .space_hierarchy
                                    .remove_child(&space_id, &child_id);
                                info!(
                                    "Space hierarchy updated: {} removed from {}",
                                    child_id, space_id
                                );
                            } else {
                                inner_write
                                    .space_hierarchy
                                    .add_child(space_id.clone(), child_id.clone());
                                info!(
                                    "Space hierarchy updated: {} is child of {}",
                                    child_id, space_id
                                );
                            }
                        }
                        SyncStateEvent::Redacted(_) => {
                            inner_write
                                .space_hierarchy
                                .remove_child(&space_id, &child_id);
                            info!(
                                "Space hierarchy updated: {} removed from {} (redacted)",
                                child_id, space_id
                            );
                        }
                    }
                }
            },
        );

        let inner_clone = self.inner.clone();
        client.add_event_handler(
            move |event: SyncStateEvent<SpaceParentEventContent>, room: Room| {
                let inner = inner_clone.clone();
                async move {
                    let child_id = room.room_id().to_owned();
                    let parent_id = match RoomId::parse(event.state_key()) {
                        Ok(id) => id,
                        Err(_) => return,
                    };

                    let mut inner_write = inner.write().await;
                    match event {
                        SyncStateEvent::Original(ev) => {
                            if ev.content.via.is_empty() {
                                inner_write
                                    .space_hierarchy
                                    .remove_child(&parent_id, &child_id);
                                info!(
                                    "Space hierarchy updated: {} removed as parent of {}",
                                    parent_id, child_id
                                );
                            } else {
                                inner_write
                                    .space_hierarchy
                                    .add_child(parent_id.clone(), child_id.clone());
                                info!(
                                    "Space hierarchy updated: {} is parent of {}",
                                    parent_id, child_id
                                );
                            }
                        }
                        SyncStateEvent::Redacted(_) => {
                            inner_write
                                .space_hierarchy
                                .remove_child(&parent_id, &child_id);
                            info!(
                                "Space hierarchy updated: {} removed as parent of {} (redacted)",
                                parent_id, child_id
                            );
                        }
                    }
                }
            },
        );
    }

    pub async fn login(&self, homeserver: &str, username: &str, password: &str) -> Result<()> {
        let homeserver_url = if homeserver.starts_with("https://")
            || homeserver.starts_with("http://localhost")
            || homeserver.starts_with("http://127.0.0.1")
            || homeserver.starts_with("http://[::1]")
        {
            homeserver.to_string()
        } else {
            let stripped = homeserver.strip_prefix("http://").unwrap_or(homeserver);
            format!("https://{}", stripped)
        };

        let client = {
            let mut inner = self.inner.write().await;
            if let Some(handle) = inner.sync_handle.take() {
                handle.abort();
            }
            if let Some(handle) = inner.session_change_handle.take() {
                handle.abort();
            }
            let data_dir = inner.data_dir.clone();
            let new_client = Self::setup_client(data_dir, &homeserver_url).await?;
            inner.client = new_client.clone();
            new_client
        };

        client
            .matrix_auth()
            .login_username(username, password)
            .initial_device_display_name("Constellations Matrix Client")
            .send()
            .await
            .context("Failed to login")?;

        let sync_service: Arc<SyncService> =
            Arc::new(SyncService::builder(client.clone()).build().await?);
        let room_list_service = sync_service.room_list_service();

        // Save session to oo7
        if let Some(session) = client.matrix_auth().session() {
            let session_data = SessionData {
                homeserver: homeserver_url,
                user_id: session.meta.user_id.to_string(),
                access_token: session.tokens.access_token.to_string(),
                refresh_token: session.tokens.refresh_token.clone(),
                id_token: None,
                device_id: session.meta.device_id.to_string(),
                is_oidc: false,
            };

            Self::save_session_to_keyring(&session_data).await?;
        }

        self.setup_event_handlers(&client);

        let mut inner = self.inner.write().await;
        inner.client = client.clone();
        inner.sync_service = Some(sync_service);
        inner.room_list_service = Some(room_list_service);

        drop(inner);
        self.spawn_session_change_handler(client).await;

        Ok(())
    }

    pub async fn restore_session(&self) -> Result<bool> {
        let keyring = Keyring::new().await?;
        let mut attributes = HashMap::new();
        attributes.insert("app_id", "fi.joonastuomi.CosmicExtConstellations");
        attributes.insert("type", "matrix-session");

        let items = keyring.search_items(&attributes).await?;

        if let Some(item) = items.first() {
            let secret = item.secret().await?;
            let session_data: SessionData = serde_json::from_slice(&secret)?;

            let data_dir = self.inner.read().await.data_dir.clone();
            let client = Self::setup_client(data_dir, &session_data.homeserver).await?;

            if session_data.is_oidc {
                client.oauth().restore_registered_client(
                    matrix_sdk::authentication::oauth::ClientId::new(OIDC_CLIENT_ID.to_string()),
                );
                client
                    .oauth()
                    .restore_session(
                        matrix_sdk::authentication::oauth::OAuthSession {
                            client_id: matrix_sdk::authentication::oauth::ClientId::new(
                                OIDC_CLIENT_ID.to_string(),
                            ),
                            user: matrix_sdk::authentication::oauth::UserSession {
                                meta: matrix_sdk::SessionMeta {
                                    user_id: UserId::parse(session_data.user_id.clone())?,
                                    device_id: OwnedDeviceId::from(session_data.device_id),
                                },
                                tokens: SessionTokens {
                                    access_token: session_data.access_token,
                                    refresh_token: session_data.refresh_token,
                                },
                            },
                        },
                        matrix_sdk::store::RoomLoadSettings::default(),
                    )
                    .await?;
            } else {
                let matrix_session = MatrixSession {
                    meta: matrix_sdk::SessionMeta {
                        user_id: UserId::parse(session_data.user_id.clone())?,
                        device_id: OwnedDeviceId::from(session_data.device_id),
                    },
                    tokens: SessionTokens {
                        access_token: session_data.access_token,
                        refresh_token: session_data.refresh_token,
                    },
                };
                client
                    .matrix_auth()
                    .restore_session(
                        matrix_session,
                        matrix_sdk::store::RoomLoadSettings::default(),
                    )
                    .await?;
            }

            let sync_service: Arc<SyncService> =
                Arc::new(SyncService::builder(client.clone()).build().await?);
            let room_list_service = sync_service.room_list_service();

            self.setup_event_handlers(&client);

            let mut inner = self.inner.write().await;
            inner.client = client.clone();
            inner.sync_service = Some(sync_service);
            inner.room_list_service = Some(room_list_service);

            drop(inner);
            self.spawn_session_change_handler(client).await;

            return Ok(true);
        }

        Ok(false)
    }

    pub async fn logout(&self) -> Result<()> {
        let keyring = Keyring::new().await?;
        
        let mut session_attributes = HashMap::new();
        session_attributes.insert("app_id", "fi.joonastuomi.CosmicExtConstellations");
        session_attributes.insert("type", "matrix-session");

        if let Ok(items) = keyring.search_items(&session_attributes).await {
            for item in items {
                let _ = item.delete().await;
            }
        }
        
        let mut pass_attributes = HashMap::new();
        pass_attributes.insert("app_id", "fi.joonastuomi.CosmicExtConstellations");
        pass_attributes.insert("type", "store-passphrase");

        if let Ok(items) = keyring.search_items(&pass_attributes).await {
            for item in items {
                let _ = item.delete().await;
            }
        }

        let mut inner = self.inner.write().await;
        if let Some(handle) = inner.sync_handle.take() {
            handle.abort();
        }
        if let Some(sync_service) = inner.sync_service.take() {
            let _ = sync_service.stop().await;
        }
        inner.room_list_service = None;
        inner.room_list_controller = None;
        inner.timelines.clear();
        inner.space_hierarchy = SpaceHierarchy::new();

        // Try logging out properly from Matrix
        let _ = inner.client.matrix_auth().logout().await;

        let store_path = inner.data_dir.join("matrix-store");
        let _ = std::fs::remove_dir_all(&store_path);

        Ok(())
    }

    pub async fn client(&self) -> Client {
        self.inner.read().await.client.clone()
    }

    pub async fn sync_service(&self) -> Option<Arc<SyncService>> {
        self.inner.read().await.sync_service.clone()
    }

    pub async fn room_list_service(&self) -> Option<Arc<RoomListService>> {
        self.inner.read().await.room_list_service.clone()
    }

    pub async fn set_room_list_controller(
        &self,
        controller: Arc<RoomListDynamicEntriesController>,
    ) {
        let mut inner = self.inner.write().await;
        inner.room_list_controller = Some(controller);
    }

    pub async fn update_room_list_filter(&self, selected_space: Option<OwnedRoomId>) -> Result<()> {
        let inner = self.inner.read().await;
        if let Some(controller) = &inner.room_list_controller {
            use matrix_sdk_ui::room_list_service::filters;

            let filter: Box<dyn matrix_sdk_ui::room_list_service::filters::Filter + Send + Sync> =
                if let Some(space_id) = selected_space {
                    let hierarchy = inner.space_hierarchy.clone();
                    let space_id_clone = space_id.clone();
                    // Custom filter that checks if the room is in the selected space OR is a space itself
                    // This ensures the SpaceSwitcher always has access to all spaces.
                    Box::new(filters::new_filter_any(vec![Box::new(
                        move |item: &matrix_sdk_ui::room_list_service::RoomListItem| {
                            hierarchy.is_in_space(item.room_id(), &space_id_clone)
                                || hierarchy.is_known_space(item.room_id())
                        },
                    )]))
                } else {
                    // No space selected, show all rooms
                    Box::new(filters::new_filter_all(vec![]))
                };

            controller.set_filter(filter);
        }
        Ok(())
    }

    pub async fn fetch_room_data(&self, room: &matrix_sdk::Room) -> Result<RoomData> {
        let id = room.room_id().to_string();
        let name = match room.name() {
            Some(n) => Some(n.to_string()),
            None => room.cached_display_name().map(|n| n.to_string()),
        };

        let unread_count = room.unread_notification_counts().notification_count as u32;
        let avatar_url = room.avatar_url().map(|u| u.to_string());

        let last_message = if let Some(latest_event) = room.latest_event() {
            if let Ok(event) = latest_event.event().raw().deserialize() {
                match event {
                    AnySyncTimelineEvent::MessageLike(AnySyncMessageLikeEvent::RoomMessage(e)) => {
                        e.as_original().map(|o| {
                            let mut msg = o.content.body().to_string();
                            if msg.len() > 30 {
                                msg.truncate(26);
                                msg.push_str("...");
                            }
                            msg
                        })
                    }
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        };

        let room_type = room.room_type();
        let is_space = room_type == Some(RoomType::Space);

        if is_space {
            let mut inner = self.inner.write().await;
            inner.space_hierarchy.add_space(room.room_id().to_owned());
        }

        let parent_space_id = {
            let inner = self.inner.read().await;
            inner
                .space_hierarchy
                .parents
                .get(room.room_id())
                .and_then(|parents| parents.first())
                .map(|id| id.to_string())
        };

        Ok(RoomData {
            id,
            name,
            last_message,
            unread_count,
            avatar_url,
            room_type,
            is_space,
            parent_space_id,
        })
    }

    pub async fn start_sync(&self) -> Result<(), SyncError> {
        let client = self.client().await;
        let request =
            matrix_sdk::ruma::api::client::discovery::get_supported_versions::Request::new();
        let versions = client.send(request).await?;
        let supports_sliding_sync = versions
            .unstable_features
            .contains_key("org.matrix.msc4186")
            || versions.versions.iter().any(|v| v == "v1.11");

        if !supports_sliding_sync {
            return Err(SyncError::MissingSlidingSyncSupport);
        }

        let mut inner = self.inner.write().await;

        if let Some(handle) = inner.sync_handle.take() {
            handle.abort();
            if let Some(sync_service) = &inner.sync_service {
                let _ = sync_service.stop().await;
            }
        }

        if let Some(sync_service) = &inner.sync_service {
            let sync_service = sync_service.clone();
            let handle = tokio::spawn(async move {
                let mut backoff = Backoff::new(BACKOFF_INITIAL, BACKOFF_MAX);
                loop {
                    let current_backoff = backoff.current;
                    info!(
                        "Starting Matrix sync service (current backoff: {}s)...",
                        current_backoff
                    );
                    let start_time = std::time::Instant::now();

                    // The start() future completes when the service is stopped or fails.
                    sync_service.start().await;

                    let elapsed = start_time.elapsed();
                    let state = sync_service.state().get();

                    if elapsed.as_secs() > BACKOFF_RESET_THRESHOLD {
                        info!(
                            "Matrix sync service ran for {:?}, resetting backoff.",
                            elapsed
                        );
                        backoff = Backoff::new(BACKOFF_INITIAL, BACKOFF_MAX);
                    }

                    let next_delay = backoff.next();
                    error!("Matrix sync service stopped after {:?}. State: {:?}. Retrying in {} seconds...", elapsed, state, next_delay);
                    tokio::time::sleep(std::time::Duration::from_secs(next_delay)).await;
                }
            });
            inner.sync_handle = Some(handle);
        }

        Ok(())
    }

    pub async fn timeline(&self, room_id: &str) -> Result<Arc<Timeline>> {
        let room_id = RoomId::parse(room_id)?;

        {
            let inner = self.inner.read().await;
            if let Some(timeline) = inner.timelines.get(&room_id) {
                return Ok(timeline.clone());
            }
        }

        let rls = self
            .room_list_service()
            .await
            .context("RoomListService not initialized")?;
        let room = rls
            .room(&room_id)
            .map_err(|e| anyhow::anyhow!("Failed to get room: {}", e))?;
        let timeline = Arc::new(room.timeline_builder().build().await?);

        let mut inner = self.inner.write().await;
        inner.timelines.insert(room_id.to_owned(), timeline.clone());

        Ok(timeline)
    }

    pub async fn paginate_backwards(&self, room_id: &str, limit: u16) -> Result<()> {
        let timeline = self.timeline(room_id).await?;
        timeline.paginate_backwards(limit).await?;
        Ok(())
    }

    pub async fn set_room_name(&self, room_id: &str, name: String) -> Result<()> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;
        room.set_name(name).await?;
        Ok(())
    }

    pub async fn set_room_topic(&self, room_id: &str, topic: String) -> Result<()> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;
        room.set_room_topic(&topic).await?;
        Ok(())
    }

    pub async fn upload_room_avatar(&self, room_id: &str, data: Vec<u8>, mime: &str) -> Result<()> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;
        
        let content_type = mime.parse::<mime::Mime>()?;
        room.upload_avatar(&content_type, data, None).await?;
        Ok(())
    }

    pub async fn leave_room(&self, room_id: &str) -> Result<()> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;
        room.leave().await?;
        Ok(())
    }

    pub async fn forget_room(&self, room_id: &str) -> Result<()> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;
        room.forget().await?;
        Ok(())
    }

    pub async fn get_room_power_levels(&self, room_id: &str) -> Result<(i64, HashMap<matrix_sdk::ruma::OwnedUserId, i64>)> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;
        let power_levels = room.power_levels().await?;
        
        let users = room.users_with_power_levels().await;
        // Also add users who have the default power level but are members
        // To avoid listing thousands of users in large rooms, maybe we only list members if the room is small?
        // Actually, let's just use what's in the power levels event first.
        // If the user wants to promote someone else, they can search for them.
        Ok((power_levels.users_default.into(), users))
    }

    pub async fn update_user_power_level(&self, room_id: &str, user_id: &str, level: i64) -> Result<()> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let user_id_parsed = matrix_sdk::ruma::UserId::parse(user_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;
        
        let int_level = matrix_sdk::ruma::Int::new(level).ok_or_else(|| anyhow::anyhow!("Invalid power level"))?;
        room.update_power_levels(vec![(&user_id_parsed, int_level)]).await?;
        Ok(())
    }

    pub async fn update_room_power_level_settings(
        &self, 
        room_id: &str, 
        ban: Option<i64>,
        invite: Option<i64>,
        kick: Option<i64>,
        redact: Option<i64>,
    ) -> Result<()> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;

        let mut changes = matrix_sdk::room::power_levels::RoomPowerLevelChanges::new();
        changes.ban = ban;
        changes.invite = invite;
        changes.kick = kick;
        changes.redact = redact;

        room.apply_power_level_changes(changes).await?;
        Ok(())
    }

    pub async fn invite_user(&self, room_id: &str, user_id: &str) -> Result<()> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let user_id_parsed = matrix_sdk::ruma::UserId::parse(user_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;
        room.invite_user_by_id(&user_id_parsed).await?;
        Ok(())
    }

    pub async fn kick_user(&self, room_id: &str, user_id: &str, reason: Option<String>) -> Result<()> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let user_id_parsed = matrix_sdk::ruma::UserId::parse(user_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;
        room.kick_user(&user_id_parsed, reason.as_deref()).await?;
        Ok(())
    }

    pub async fn ban_user(&self, room_id: &str, user_id: &str, reason: Option<String>) -> Result<()> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let user_id_parsed = matrix_sdk::ruma::UserId::parse(user_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;
        room.ban_user(&user_id_parsed, reason.as_deref()).await?;
        Ok(())
    }

    pub async fn send_message(
        &self,
        room_id: &str,
        body: String,
        html_body: Option<String>,
    ) -> Result<()> {
        let room_id = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id).context("Room not found")?;

        let content = if let Some(html) = html_body {
            RoomMessageEventContent::text_html(body, html)
        } else {
            RoomMessageEventContent::text_plain(body)
        };

        room.send(content).await?;
        Ok(())
    }

    pub async fn fetch_media(&self, source: MediaSource) -> Result<Vec<u8>> {
        let client = self.client().await;
        let request = matrix_sdk::media::MediaRequestParameters {
            source,
            format: MediaFormat::File,
        };
        let content = client.media().get_media_content(&request, true).await?;
        Ok(content)
    }

    pub async fn create_room(&self, name: &str) -> Result<OwnedRoomId> {
        let client = self.client().await;
        let mut request = matrix_sdk::ruma::api::client::room::create_room::v3::Request::new();
        request.name = Some(name.to_string());
        let room = client.create_room(request).await?;
        Ok(room.room_id().to_owned())
    }

    pub async fn is_in_space(&self, room_id: &RoomId, space_id: &RoomId) -> bool {
        let inner = self.inner.read().await;
        inner.space_hierarchy.is_in_space(room_id, space_id)
    }

    pub fn is_in_space_sync(&self, room_id: &RoomId, space_id: &RoomId) -> bool {
        if let Ok(inner) = self.inner.try_read() {
            inner.space_hierarchy.is_in_space(room_id, space_id)
        } else {
            false
        }
    }

    pub async fn login_oidc(&self, homeserver: &str) -> Result<Url> {
        let homeserver_url = if homeserver.starts_with("https://")
            || homeserver.starts_with("http://localhost")
            || homeserver.starts_with("http://127.0.0.1")
            || homeserver.starts_with("http://[::1]")
        {
            homeserver.to_string()
        } else {
            let stripped = homeserver.strip_prefix("http://").unwrap_or(homeserver);
            format!("https://{}", stripped)
        };

        let client = {
            let mut inner = self.inner.write().await;
            if let Some(handle) = inner.sync_handle.take() {
                handle.abort();
            }
            if let Some(handle) = inner.session_change_handle.take() {
                handle.abort();
            }
            let data_dir = inner.data_dir.clone();
            let new_client = Self::setup_client(data_dir, &homeserver_url).await?;
            inner.client = new_client.clone();
            new_client
        };

        client
            .oauth()
            .restore_registered_client(matrix_sdk::authentication::oauth::ClientId::new(
                OIDC_CLIENT_ID.to_string(),
            ));

        let redirect_uri = Url::parse(OIDC_CALLBACK_URL)?;
        let login_url = client
            .oauth()
            .login(redirect_uri, None, None, None)
            .build()
            .await?
            .url;

        let mut inner = self.inner.write().await;
        inner.oidc_client = Some(client);

        Ok(login_url)
    }

    pub async fn complete_oidc_login(&self, callback_url: Url) -> Result<()> {
        let client = {
            let mut inner = self.inner.write().await;
            inner
                .oidc_client
                .take()
                .context("No OIDC login in progress")?
        };

        client
            .oauth()
            .finish_login(callback_url.into())
            .await
            .context("Failed to complete OIDC login")?;

        let sync_service: Arc<SyncService> =
            Arc::new(SyncService::builder(client.clone()).build().await?);
        let room_list_service = sync_service.room_list_service();

        self.setup_event_handlers(&client);

        // Save session to oo7
        if let Some(session) = client.oauth().user_session() {
            let session_data = SessionData {
                homeserver: client.homeserver().to_string(),
                user_id: session.meta.user_id.to_string(),
                access_token: session.tokens.access_token.to_string(),
                refresh_token: session.tokens.refresh_token.clone(),
                id_token: None,
                device_id: session.meta.device_id.to_string(),
                is_oidc: true,
            };

            Self::save_session_to_keyring(&session_data).await?;
        }

        let mut inner = self.inner.write().await;
        inner.client = client.clone();
        inner.sync_service = Some(sync_service);
        inner.room_list_service = Some(room_list_service);

        drop(inner);
        self.spawn_session_change_handler(client).await;

        Ok(())
    }

    async fn get_or_create_store_passphrase() -> Result<String> {
        let keyring = Keyring::new().await?;
        let mut attributes = HashMap::new();
        attributes.insert("app_id", "fi.joonastuomi.CosmicExtConstellations");
        attributes.insert("type", "store-passphrase");

        let items = keyring.search_items(&attributes).await?;

        if let Some(item) = items.first() {
            let secret = item.secret().await?;
            if let Ok(passphrase) = String::from_utf8(secret.to_vec()) {
                return Ok(passphrase);
            }
        }

        // Securely generate passphrase using standard library functionality (URandom) to avoid adding dependencies
        let mut buf = [0u8; 32];
        let mut f = std::fs::File::open("/dev/urandom")
            .context("Failed to open /dev/urandom for secure random generation")?;

        use std::io::Read;
        f.read_exact(&mut buf)
            .context("Failed to securely read bytes from /dev/urandom")?;

        let passphrase: String = buf.iter().map(|b| format!("{:02x}", b)).collect();

        keyring
            .create_item(
                "Constellations Store Passphrase",
                &attributes,
                passphrase.as_bytes(),
                true,
            )
            .await?;

        Ok(passphrase)
    }

    async fn setup_client(data_dir: PathBuf, homeserver_url: &str) -> Result<Client> {
        let store_path = data_dir.join("matrix-store");

        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir)?;
        }

        let passphrase = Self::get_or_create_store_passphrase().await?;

        let build_client = |path: &PathBuf, pass: &str| {
            Client::builder()
                .homeserver_url(homeserver_url)
                .sqlite_store(path, Some(pass))
                .handle_refresh_tokens()
        };

        let client = match build_client(&store_path, &passphrase).build().await {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("Failed to initialize stores (possibly corrupted cipher): {}. Recreating store.", e);
                let _ = std::fs::remove_dir_all(&store_path);
                build_client(&store_path, &passphrase).build().await?
            }
        };

        Ok(client)
    }
}

pub fn markdown_to_html(markdown: &str) -> String {
    let mut html_body = String::new();
    let parser = pulldown_cmark::Parser::new(markdown);
    pulldown_cmark::html::push_html(&mut html_body, parser);
    html_body
}

#[cfg(test)]
mod tests;
