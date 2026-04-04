use anyhow::{Context, Result};
use thiserror::Error;
use matrix_sdk::{
    config::StoreConfig,
    ruma::{UserId, OwnedDeviceId, RoomId, OwnedRoomId},
    Client,
    matrix_auth::MatrixSession,
};
use matrix_sdk::media::{MediaRequest, MediaFormat};
use matrix_sdk::ruma::events::room::MediaSource;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::ruma::events::{AnySyncTimelineEvent, AnySyncMessageLikeEvent};
use matrix_sdk_sqlite::SqliteStateStore;
pub use matrix_sdk_ui::timeline::{Timeline, TimelineItem, PaginationOptions, VirtualTimelineItem};
use matrix_sdk_ui::RoomListService;
use matrix_sdk_ui::sync_service::SyncService;
use eyeball_im::VectorDiff;
use oo7::Keyring;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{info, error};

const BACKOFF_INITIAL: u64 = 2;
const BACKOFF_MAX: u64 = 60;
const BACKOFF_RESET_THRESHOLD: u64 = 30;

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
}

pub type RoomListDiff = VectorDiff<RoomData>;
pub type TimelineDiff<T> = VectorDiff<Arc<T>>;

#[derive(Debug, Clone)]
pub enum MatrixEvent {
    SyncStatusChanged(SyncStatus),
    RoomDiff(RoomListDiff),
    TimelineDiff(TimelineDiff<TimelineItem>),
    TimelineReset,
    ReactionAdded { room_id: String, event_id: String, reaction: String },
}

#[derive(Serialize, Deserialize, Debug)]
struct SessionData {
    homeserver: String,
    user_id: String,
    access_token: String,
    refresh_token: Option<String>,
    device_id: String,
}

#[derive(Clone, Debug)]
pub struct MatrixEngine {
    inner: Arc<RwLock<MatrixEngineInner>>,
}

struct MatrixEngineInner {
    client: Client,
    sync_service: Option<Arc<SyncService>>,
    room_list_service: Option<Arc<RoomListService>>,
    timelines: HashMap<OwnedRoomId, Arc<Timeline>>,
    data_dir: PathBuf,
    sync_handle: Option<tokio::task::JoinHandle<()>>,
}

impl std::fmt::Debug for MatrixEngineInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MatrixEngineInner")
            .field("client", &self.client)
            .field("sync_service", &self.sync_service.as_ref().map(|_| "SyncService"))
            .field("room_list_service", &self.room_list_service.as_ref().map(|_| "RoomListService"))
            .field("timelines", &self.timelines.keys())
            .field("data_dir", &self.data_dir)
            .field("sync_handle", &self.sync_handle.as_ref().map(|_| "JoinHandle"))
            .finish()
    }
}

struct Backoff {
    current: u64,
    max: u64,
}

impl Backoff {
    fn new(initial: u64, max: u64) -> Self {
        Self { current: initial, max }
    }

    fn next(&mut self) -> u64 {
        let next = self.current;
        self.current = (self.current * 2).min(self.max);
        next
    }
}

impl MatrixEngine {
    pub async fn new(data_dir: PathBuf) -> Result<Self> {
        let store_path = data_dir.join("matrix-store.db");
        
        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir)?;
        }

        let sqlite_store = SqliteStateStore::open(&store_path, None).await?;
        let store_config = StoreConfig::default().state_store(sqlite_store);

        let client = Client::builder()
            .homeserver_url("https://matrix.org")
            .store_config(store_config)
            .build()
            .await?;

        let inner = MatrixEngineInner {
            client,
            sync_service: None,
            room_list_service: None,
            timelines: HashMap::new(),
            data_dir,
            sync_handle: None,
        };

        Ok(Self { inner: Arc::new(RwLock::new(inner)) })
    }

    pub async fn login(&self, homeserver: &str, username: &str, password: &str) -> Result<()> {
        let homeserver_url = if homeserver.starts_with("http") {
            homeserver.to_string()
        } else {
            format!("https://{}", homeserver)
        };

        let data_dir = self.inner.read().await.data_dir.clone();
        let store_path = data_dir.join("matrix-store.db");
        let sqlite_store = SqliteStateStore::open(&store_path, None).await?;
        let store_config = StoreConfig::default().state_store(sqlite_store);

        let client = Client::builder()
            .homeserver_url(&homeserver_url)
            .store_config(store_config)
            .build()
            .await?;

        client
            .matrix_auth()
            .login_username(username, password)
            .initial_device_display_name("Claw Matrix Client")
            .send()
            .await
            .context("Failed to login")?;

        let sync_service: Arc<SyncService> = Arc::new(SyncService::builder(client.clone()).build().await?);
        let room_list_service = sync_service.room_list_service();
        
        // Save session to oo7
        if let Some(session) = client.session() {
            let meta = session.meta();
            let access_token = session.access_token();

            let session_data = SessionData {
                homeserver: homeserver_url,
                user_id: meta.user_id.to_string(),
                access_token: access_token.to_string(),
                refresh_token: session.get_refresh_token().map(|t| t.to_string()),
                device_id: meta.device_id.to_string(),
            };

            let keyring = Keyring::new().await?;
            let mut attributes = HashMap::new();
            attributes.insert("app_id", "com.system76.Claw");
            attributes.insert("type", "matrix-session");
            
            let secret = serde_json::to_vec(&session_data)?;

            keyring
                .create_item("Claw Matrix Session", &attributes, &secret, true)
                .await?;
        }

        let mut inner = self.inner.write().await;
        inner.client = client;
        inner.sync_service = Some(sync_service);
        inner.room_list_service = Some(room_list_service);

        Ok(())
    }

    pub async fn restore_session(&self) -> Result<bool> {
        let keyring = Keyring::new().await?;
        let mut attributes = HashMap::new();
        attributes.insert("app_id", "com.system76.Claw");
        attributes.insert("type", "matrix-session");
        
        let items = keyring.search_items(&attributes).await?;

        if let Some(item) = items.first() {
            let secret = item.secret().await?;
            let session_data: SessionData = serde_json::from_slice(&secret)?;

            let matrix_session = MatrixSession {
                meta: matrix_sdk::SessionMeta {
                    user_id: UserId::parse(session_data.user_id.clone())?,
                    device_id: OwnedDeviceId::from(session_data.device_id),
                },
                tokens: matrix_sdk::matrix_auth::MatrixSessionTokens {
                    access_token: session_data.access_token,
                    refresh_token: session_data.refresh_token,
                },
            };

            let data_dir = self.inner.read().await.data_dir.clone();
            let store_path = data_dir.join("matrix-store.db");
            let sqlite_store = SqliteStateStore::open(&store_path, None).await?;
            let store_config = StoreConfig::default().state_store(sqlite_store);

            let client = Client::builder()
                .homeserver_url(&session_data.homeserver)
                .store_config(store_config)
                .build()
                .await?;

            client.matrix_auth().restore_session(matrix_session).await?;
            
            let sync_service: Arc<SyncService> = Arc::new(SyncService::builder(client.clone()).build().await?);
            let room_list_service = sync_service.room_list_service();
            
            let mut inner = self.inner.write().await;
            inner.client = client;
            inner.sync_service = Some(sync_service);
            inner.room_list_service = Some(room_list_service);
            
            return Ok(true);
        }

        Ok(false)
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

    pub async fn fetch_room_data(&self, room: &matrix_sdk::Room) -> Result<RoomData> {
        let id = room.room_id().to_string();
        let name = room.display_name().await.ok().map(|n| n.to_string());
        
        let unread_count = room.unread_notification_counts().notification_count as u32;
        let avatar_url = room.avatar_url().map(|u| u.to_string());
        
        let last_message = if let Some(latest_event) = room.latest_event() {
            if let Ok(event) = latest_event.event().event.deserialize() {
                match event {
                    AnySyncTimelineEvent::MessageLike(AnySyncMessageLikeEvent::RoomMessage(e)) => {
                        e.as_original().map(|o| o.content.body().to_string())
                    }
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        };

        Ok(RoomData {
            id,
            name,
            last_message,
            unread_count,
            avatar_url,
        })
    }

    pub async fn start_sync(&self) -> Result<(), SyncError> {
        let client = self.client().await;
        let request = matrix_sdk::ruma::api::client::discovery::get_supported_versions::Request::new();
        let versions = client.send(request, None).await?;
        let supports_sliding_sync = versions.unstable_features.contains_key("org.matrix.msc4186") || 
                                   versions.versions.iter().any(|v| v == "v1.11");
        
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
                    info!("Starting Matrix sync service (current backoff: {}s)...", current_backoff);
                    let start_time = std::time::Instant::now();
                    
                    // The start() future completes when the service is stopped or fails.
                    sync_service.start().await;
                    
                    let elapsed = start_time.elapsed();
                    let state = sync_service.state().get();
                    
                    if elapsed.as_secs() > BACKOFF_RESET_THRESHOLD {
                        info!("Matrix sync service ran for {:?}, resetting backoff.", elapsed);
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

        let rls = self.room_list_service().await.context("RoomListService not initialized")?;
        let room = rls.room(&room_id).await
            .map_err(|e| anyhow::anyhow!("Failed to get room: {}", e))?;
        let timeline = room.timeline().await;
        
        let mut inner = self.inner.write().await;
        inner.timelines.insert(room_id.to_owned(), timeline.clone());
        
        Ok(timeline)
    }

    pub async fn paginate_backwards(&self, room_id: &str, limit: u16) -> Result<()> {
        let timeline = self.timeline(room_id).await?;
        timeline.paginate_backwards(PaginationOptions::simple_request(limit)).await?;
        Ok(())
    }

    pub async fn send_message(&self, room_id: &str, body: String, html_body: Option<String>) -> Result<()> {
        let room_id = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id)
            .context("Room not found")?;

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
        let content = client.media().get_media_content(&MediaRequest {
            source,
            format: MediaFormat::File,
        }, true).await?;
        Ok(content)
    }

    pub async fn create_room(&self, name: &str) -> Result<OwnedRoomId> {
        let client = self.client().await;
        let mut request = matrix_sdk::ruma::api::client::room::create_room::v3::Request::new();
        request.name = Some(name.to_string());
        let room = client.create_room(request).await?;
        Ok(room.room_id().to_owned())
    }

    pub async fn login_oidc(&self) -> Result<url::Url> {
        let client = self.client().await;
        // In matrix-sdk 0.7, OIDC is handled via the oidc() method on matrix_auth()
        // but it requires specific features and setup. 
        // Given the "stub or placeholder" instruction for complexity:
        let _ = client;
        Err(anyhow::anyhow!("OIDC login is not yet implemented"))
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
